use eidoplot_text::{self as text, font};
use scale::{CoordMap, CoordMapXy};
use text::TextLayout;
use tiny_skia_path::Transform;

use crate::drawing::legend::Legend;
use crate::drawing::series::{Series, series_has_legend};
use crate::drawing::{Ctx, Error, SurfWrapper, axis, scale, ticks};
use crate::render::{self, Surface as _};
use crate::style::{self, defaults, theme, Color as _, Theme};
use crate::{data, geom, ir, missing_params};

#[derive(Debug, Clone)]
struct Ticks {
    locs: Vec<data::OwnedSample>,
    lbls: Vec<TextLayout>,
    annot: Option<String>,
    font: ir::axis::TicksFont,
    color: theme::Color,
    grid: Option<theme::Line>,
}

impl Ticks {
    fn lbl_width(&self) -> f32 {
        self.lbls
            .iter()
            .map(|l| l.bbox().width())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }
}

#[derive(Debug, Clone)]
struct MinorTicks {
    locs: Vec<f64>,
    color: theme::Color,
    grid: Option<theme::Line>,
}

#[derive(Debug)]
struct Axis {
    ortho_sz: f32,
    coord_map: Box<dyn CoordMap>,
    ticks: Option<Ticks>,
    minor_ticks: Option<MinorTicks>,
    label: Option<TextLayout>,
}

impl CoordMap for Axis {
    fn map_coord_num(&self, num: f64) -> f32 {
        self.coord_map.map_coord_num(num)
    }

    fn map_coord_cat(&self, cat: &str) -> f32 {
        self.coord_map.map_coord_cat(cat)
    }

    fn axis_bounds(&self) -> axis::BoundsRef<'_> {
        self.coord_map.axis_bounds()
    }
}

struct Axes {
    x: Axis,
    y: Axis,
}

impl Axes {
    fn x_height(&self) -> f32 {
        self.x.ortho_sz
    }
    fn y_width(&self) -> f32 {
        self.y.ortho_sz
    }
}

fn plot_insets(plot: &ir::Plot) -> geom::Padding {
    match plot.insets {
        Some(ir::plot::Insets::Fixed(x, y)) => geom::Padding::Center { v: y, h: x },
        Some(ir::plot::Insets::Auto) => auto_insets(plot),
        None => geom::Padding::Even(0.0),
    }
}

fn auto_insets(plot: &ir::Plot) -> geom::Padding {
    for s in plot.series.iter() {
        match &s.plot {
            ir::series::SeriesPlot::Histogram(..) => return defaults::PLOT_HIST_AUTO_INSETS,
            _ => (),
        }
    }
    defaults::PLOT_XY_AUTO_INSETS
}

impl<D, T> Ctx<'_, D, T> {
    fn setup_plot_axes(
        &self,
        plot: &ir::Plot,
        ab: (&axis::Bounds, &axis::Bounds),
        rect: &geom::Rect,
    ) -> Result<Axes, Error> {
        let insets = plot_insets(plot);

        // x-axis height only depends on font size, so it can be computed right-away,
        // y-axis width depends on font width therefore we have to generate tick labels,
        // which somehow depends on the x-axis height (for available space)
        // so the layout is bootstrapped in the following order:
        // - x-axis height
        // - y-axis ticks and labels
        // - y_axis width
        // - x-axis ticks and labels

        let x_height = self.calculate_x_axis_height(&plot.x_axis);
        let rect = rect.shifted_bottom_side(-x_height);

        let y_cm = scale::map_scale_coord(
            plot.y_axis.scale(),
            rect.height(),
            ab.1,
            (insets.bottom(), insets.top()),
        );
        let y_axis = self.setup_y_axis(&plot.y_axis, y_cm)?;
        let rect = rect.shifted_left_side(y_axis.ortho_sz);

        let x_cm = scale::map_scale_coord(
            plot.x_axis.scale(),
            rect.width(),
            ab.0,
            (insets.left(), insets.right()),
        );
        let x_axis = self.setup_x_axis(&plot.x_axis, x_cm, x_height)?;

        Ok(Axes {
            x: x_axis,
            y: y_axis,
        })
    }

    fn calculate_x_axis_height(&self, x_axis: &ir::Axis) -> f32 {
        let mut height = 0.0;
        if let Some(ticks) = x_axis.ticks() {
            height +=
                missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN + ticks.font().size;
        }
        if let Some(label) = x_axis.label() {
            height += 2.0 * missing_params::AXIS_LABEL_MARGIN + label.font.size;
        }
        height
    }

    // TODO: When pxl draws on its own rather than using resvg,
    // this function should return the calculated shapes and cache them in the render::Text
    // and send them to the surface for reuse
    fn calculate_y_axis_width(&self, y_axis: &ir::Axis, y_ticks: Option<&Ticks>) -> f32 {
        let mut width = 0.0;
        if let Some(label) = y_axis.label() {
            width += 2.0 * missing_params::AXIS_LABEL_MARGIN + label.font.size;
        }
        if let Some(ticks) = y_ticks {
            width +=
                missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN + ticks.lbl_width();
        }
        width
    }

    fn setup_y_axis(&self, y_axis: &ir::Axis, coord_map: Box<dyn CoordMap>) -> Result<Axis, Error> {
        let ticks = y_axis
            .ticks()
            .map(|t| self.setup_y_ticks(t, coord_map.axis_bounds()))
            .transpose()?;

        let major_locs = ticks.as_ref().map(|t| t.locs.as_slice()).unwrap_or(&[]);

        let minor_ticks = if let Some(mt) = y_axis.minor_ticks() {
            let bounds = coord_map.axis_bounds();
            let num_bounds = bounds.as_num().ok_or_else(|| {
                Error::InconsistentAxisBounds("Can't use minor ticks with categories".into())
            })?;
            Some(self.setup_minor_ticks(mt, major_locs, num_bounds)?)
        } else {
            None
        };

        let y_width = self.calculate_y_axis_width(y_axis, ticks.as_ref());

        let opts = text::layout::Options {
            hor_align: text::layout::HorAlign::Center,
            ver_align: text::layout::LineVerAlign::Hanging.into(),
            ..Default::default()
        };
        let label = y_axis
            .label()
            .map(|l| {
                text::shape_and_layout_str(&l.text, &l.font.font, &self.fontdb, l.font.size, &opts)
            })
            .transpose()?;

        Ok(Axis {
            ortho_sz: y_width,
            coord_map,
            ticks,
            minor_ticks,
            label,
        })
    }

    fn setup_x_axis(
        &self,
        x_axis: &ir::Axis,
        coord_map: Box<dyn CoordMap>,
        x_height: f32,
    ) -> Result<Axis, Error> {
        let ticks = x_axis
            .ticks()
            .map(|t| self.setup_x_ticks(t, coord_map.axis_bounds()))
            .transpose()?;

        let major_locs = ticks.as_ref().map(|t| t.locs.as_slice()).unwrap_or(&[]);

        let minor_ticks = if let Some(mt) = x_axis.minor_ticks() {
            let bounds = coord_map.axis_bounds();
            let num_bounds = bounds.as_num().ok_or_else(|| {
                Error::InconsistentAxisBounds("Can't use minor ticks with categories".into())
            })?;
            Some(self.setup_minor_ticks(mt, major_locs, num_bounds)?)
        } else {
            None
        };

        let opts = text::layout::Options {
            hor_align: text::layout::HorAlign::Center,
            ver_align: text::layout::LineVerAlign::Hanging.into(),
            ..Default::default()
        };
        let label = x_axis
            .label()
            .map(|l| {
                text::shape_and_layout_str(&l.text, &l.font.font, &self.fontdb, l.font.size, &opts)
            })
            .transpose()?;

        Ok(Axis {
            ortho_sz: x_height,
            coord_map,
            ticks,
            minor_ticks,
            label,
        })
    }

    fn setup_x_ticks(&self, ticks: &ir::axis::Ticks, ab: axis::BoundsRef) -> Result<Ticks, Error> {
        let opts = text::layout::Options {
            hor_align: text::layout::HorAlign::Center,
            ver_align: text::layout::LineVerAlign::Hanging.into(),
            ..Default::default()
        };
        self.setup_ticks(ticks, ab, opts)
    }

    fn setup_y_ticks(&self, ticks: &ir::axis::Ticks, ab: axis::BoundsRef) -> Result<Ticks, Error> {
        let opts = text::layout::Options {
            hor_align: text::layout::HorAlign::Right,
            ver_align: text::layout::LineVerAlign::Middle.into(),
            ..Default::default()
        };
        self.setup_ticks(ticks, ab, opts)
    }

    fn setup_ticks(
        &self,
        ticks: &ir::axis::Ticks,
        ab: axis::BoundsRef,
        opts: text::layout::Options,
    ) -> Result<Ticks, Error> {
        let mut locs = ticks::locate(ticks.locator(), ab);
        if let Some(ab) = ab.as_num() {
            locs.retain(|l| ab.contains(l.as_num().unwrap()));
        }
        let lbl_formatter = ticks::label_formatter(ticks, ab);
        let font = ticks.font();
        let db: &font::Database = self.fontdb();
        let lbls: Result<Vec<TextLayout>, _> = locs
            .iter()
            .map(|s| lbl_formatter.format_label(s.as_sample()))
            .map(|l| text::shape_and_layout_str(&l, &font.font, db, font.size, &opts))
            .collect();
        let lbls = lbls?;
        let annot = lbl_formatter.axis_annotation().map(String::from);
        Ok(Ticks {
            locs,
            lbls,
            annot,
            font: ticks.font().clone(),
            color: ticks.color(),
            grid: ticks.grid().cloned(),
        })
    }

    fn setup_minor_ticks(
        &self,
        minor_ticks: &ir::axis::MinorTicks,
        major_locs: &[data::OwnedSample],
        ab: axis::NumBounds,
    ) -> Result<MinorTicks, Error> {
        let mut locs = ticks::locate_minor(minor_ticks.locator(), ab);
        locs.retain(|l| ab.contains(*l) && !ticks_locs_contain(major_locs, *l));
        Ok(MinorTicks {
            locs,
            color: minor_ticks.color(),
            grid: minor_ticks.grid().cloned(),
        })
    }
}

fn ticks_locs_contain(locs: &[data::OwnedSample], t: f64) -> bool {
    locs.iter()
        .find(|&l| tick_loc_is_close(l.as_num().expect("Should be a number"), t))
        .is_some()
}

fn tick_loc_is_close(a: f64, b: f64) -> bool {
    let ratio = a / b;
    ratio.is_finite() && (ratio - 1.0).abs() < 1e-8
}

impl<D, T> Ctx<'_, D, T>
where
    D: data::Source,
{
    fn setup_plot_series(&self, plot: &ir::Plot) -> Result<Vec<Series>, Error> {
        plot.series
            .iter()
            .enumerate()
            .map(|(index, s)| Series::from_ir(index, s, self.data_source()))
            .collect()
    }
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    pub fn draw_plot<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        plot: &ir::Plot,
        rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        D: data::Source,
        T: style::Theme,
    {
        let rect = {
            let mut rect = rect.pad(&missing_params::PLOT_PADDING);

            // draw outer legend and adjust rect
            if let Some(legend) = &plot.legend {
                if !legend.pos().is_inside() {
                    self.draw_plot_outer_legend(ctx, plot, legend, &mut rect)?;
                }
            }
            rect
        };

        let series = ctx.setup_plot_series(plot)?;
        let (x_bounds, y_bounds) = Series::unite_bounds(&series)?.ok_or(Error::UnboundedAxis)?;

        let axes = ctx.setup_plot_axes(plot, (&x_bounds, &y_bounds), &rect)?;

        let rect = rect
            .shifted_left_side(axes.y_width())
            .shifted_bottom_side(-axes.x_height());

        self.draw_plot_background(ctx, plot, &rect)?;
        self.draw_grid(ctx, &axes, &rect)?;
        self.draw_plot_series(ctx, &plot.series, &series, &rect, &axes)?;
        self.draw_x_axis(ctx, &axes.x, &rect)?;
        self.draw_y_axis(ctx, &axes.y, &rect)?;
        self.draw_plot_border(ctx, plot.border.as_ref(), &rect)?;

        if let Some(legend) = &plot.legend {
            if legend.pos().is_inside() {
                self.draw_plot_inner_legend(ctx, plot, legend, &rect)?;
            }
        }

        Ok(())
    }

    fn draw_plot_outer_legend<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        plot: &ir::Plot,
        legend: &ir::PlotLegend,
        rect: &mut geom::Rect,
    ) -> Result<(), Error>
    where
        T: style::Theme,
    {
        let mut dlegend = Legend::from_ir(
            legend.legend(),
            legend.pos().prefers_vertical(),
            rect.width(),
            ctx.fontdb().clone(),
        );
        for (index, s) in plot.series.iter().enumerate() {
            if series_has_legend(s) {
                dlegend.add_entry(index, s)?;
            }
        }
        let sz = dlegend.layout();
        let top_left = match legend.pos() {
            ir::plot::LegendPos::OutTop => {
                let tl = geom::Point::new(rect.center_x() - sz.width() / 2.0, rect.top());
                rect.shift_top_side(sz.height() + legend.margin());
                tl
            }
            ir::plot::LegendPos::OutRight => {
                rect.shift_right_side(-sz.width() - legend.margin());
                geom::Point::new(
                    rect.right() + legend.margin(),
                    rect.center_y() - sz.height() / 2.0,
                )
            }
            ir::plot::LegendPos::OutBottom => {
                rect.shift_bottom_side(-sz.height() - legend.margin());
                geom::Point::new(
                    rect.center_x() - sz.width() / 2.0,
                    rect.bottom() + legend.margin(),
                )
            }
            ir::plot::LegendPos::OutLeft => {
                let tl = geom::Point::new(rect.left(), rect.center_y() - sz.height() / 2.0);
                rect.shift_left_side(sz.width() + legend.margin());
                tl
            }
            _ => unreachable!(),
        };
        self.draw_legend(ctx, &dlegend, &top_left)?;
        Ok(())
    }

    fn draw_plot_inner_legend<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        plot: &ir::Plot,
        legend: &ir::PlotLegend,
        rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        T: style::Theme,
    {
        let mut dlegend = Legend::from_ir(
            legend.legend(),
            legend.pos().prefers_vertical(),
            rect.width(),
            ctx.fontdb().clone(),
        );
        for (index, s) in plot.series.iter().enumerate() {
            if series_has_legend(s) {
                dlegend.add_entry(index, s)?;
            }
        }

        let sz = dlegend.layout();

        let top_left = match legend.pos() {
            ir::plot::LegendPos::InTop => geom::Point::new(
                rect.center_x() - sz.width() / 2.0,
                rect.top() + legend.margin(),
            ),
            ir::plot::LegendPos::InTopRight => geom::Point::new(
                rect.right() - sz.width() - legend.margin(),
                rect.top() + legend.margin(),
            ),
            ir::plot::LegendPos::InRight => geom::Point::new(
                rect.right() - sz.width() - legend.margin(),
                rect.center_y() - sz.height() / 2.0,
            ),
            ir::plot::LegendPos::InBottomRight => geom::Point::new(
                rect.right() - sz.width() - legend.margin(),
                rect.bottom() - sz.height() - legend.margin(),
            ),
            ir::plot::LegendPos::InBottom => geom::Point::new(
                rect.center_x() - sz.width() / 2.0,
                rect.bottom() - sz.height() - legend.margin(),
            ),
            ir::plot::LegendPos::InBottomLeft => geom::Point::new(
                rect.left() + legend.margin(),
                rect.bottom() - sz.height() - legend.margin(),
            ),
            ir::plot::LegendPos::InLeft => geom::Point::new(
                rect.left() + legend.margin(),
                rect.center_y() - sz.height() / 2.0,
            ),
            ir::plot::LegendPos::InTopLeft => {
                geom::Point::new(rect.left() + legend.margin(), rect.top() + legend.margin())
            }
            _ => unreachable!(),
        };
        self.draw_legend(ctx, &dlegend, &top_left)?;
        Ok(())
    }

    fn draw_plot_background<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        plot: &ir::Plot,
        rect: &geom::Rect,
    ) -> Result<(), render::Error>
    where
        T: Theme,
    {
        if let Some(fill) = plot.fill.as_ref() {
            self.draw_rect(&render::Rect {
                rect: *rect,
                fill: Some(fill.as_paint(ctx.theme())),
                stroke: None,
                transform: None,
            })?;
        }
        Ok(())
    }

    fn draw_grid<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        axes: &Axes,
        rect: &geom::Rect,
    ) -> Result<(), render::Error>
    where
        T: Theme,
    {
        if let Some(x_min_ticks) = axes.x.minor_ticks.as_ref() {
            if let Some(grid) = &x_min_ticks.grid {
                let mut pathb = geom::PathBuilder::with_capacity(
                    2 * x_min_ticks.locs.len(),
                    2 * x_min_ticks.locs.len(),
                );
                for t in x_min_ticks.locs.iter().copied() {
                    let x = axes.x.map_coord_num(t.into()) + rect.left();
                    pathb.move_to(x, rect.top());
                    pathb.line_to(x, rect.bottom());
                    let path = pathb.finish().expect("Should be a valid path");
                    let rpath = render::Path {
                        path: &path,
                        fill: None,
                        stroke: Some(grid.as_stroke(ctx.theme())),
                        transform: None,
                    };
                    self.draw_path(&rpath)?;
                    pathb = path.clear();
                }
            }
        }
        if let Some(x_ticks) = axes.x.ticks.as_ref() {
            if let Some(x_grid) = &x_ticks.grid {
                let mut pathb = geom::PathBuilder::with_capacity(
                    2 * x_ticks.locs.len(),
                    2 * x_ticks.locs.len(),
                );
                for t in x_ticks.locs.iter() {
                    let x = axes
                        .x
                        .map_coord(t.as_sample())
                        .expect("Should be a valid coord")
                        + rect.left();
                    pathb.move_to(x, rect.top());
                    pathb.line_to(x, rect.bottom());
                    let path = pathb.finish().expect("Should be a valid path");
                    let rpath = render::Path {
                        path: &path,
                        fill: None,
                        stroke: Some(x_grid.as_stroke(ctx.theme())),
                        transform: None,
                    };
                    self.draw_path(&rpath)?;
                    pathb = path.clear();
                }
            }
        }
        if let Some(y_min_ticks) = axes.y.minor_ticks.as_ref() {
            if let Some(grid) = &y_min_ticks.grid {
                let mut pathb = geom::PathBuilder::with_capacity(
                    2 * y_min_ticks.locs.len(),
                    2 * y_min_ticks.locs.len(),
                );
                for t in y_min_ticks.locs.iter().copied() {
                    let y = rect.bottom() - axes.y.map_coord_num(t);
                    pathb.move_to(rect.left(), y);
                    pathb.line_to(rect.right(), y);
                    let path = pathb.finish().expect("Should be a valid path");
                    let pathr = render::Path {
                        path: &path,
                        fill: None,
                        stroke: Some(grid.as_stroke(ctx.theme())),
                        transform: None,
                    };
                    self.draw_path(&pathr)?;
                    pathb = path.clear();
                }
            }
        }
        if let Some(y_ticks) = axes.y.ticks.as_ref() {
            if let Some(y_grid) = &y_ticks.grid {
                let mut pathb = geom::PathBuilder::with_capacity(
                    2 * y_ticks.locs.len(),
                    2 * y_ticks.locs.len(),
                );
                for t in y_ticks.locs.iter() {
                    let y = rect.bottom()
                        - axes
                            .y
                            .map_coord(t.as_sample())
                            .expect("Should be a valid coord");
                    pathb.move_to(rect.left(), y);
                    pathb.line_to(rect.right(), y);
                    let path = pathb.finish().expect("Should be a valid path");
                    let pathr = render::Path {
                        path: &path,
                        fill: None,
                        stroke: Some(y_grid.as_stroke(ctx.theme())),
                        transform: None,
                    };
                    self.draw_path(&pathr)?;
                    pathb = path.clear();
                }
            }
        }
        Ok(())
    }

    fn draw_plot_border<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        border: Option<&ir::plot::Border>,
        rect: &geom::Rect,
    ) -> Result<(), render::Error>
    where
        T: Theme,
    {
        match border {
            None => Ok(()),
            Some(ir::plot::Border::Box(stroke)) => self.draw_rect(&render::Rect {
                rect: *rect,
                fill: None,
                stroke: Some(stroke.as_stroke(ctx.theme())),
                transform: None,
            }),
            Some(ir::plot::Border::Axis(stroke)) => {
                let mut path = geom::PathBuilder::with_capacity(4, 4);
                path.move_to(rect.left(), rect.top());
                path.line_to(rect.left(), rect.bottom());
                path.line_to(rect.right(), rect.bottom());
                let path = path.finish().expect("Should be a valid path");
                let path = render::Path {
                    path: &path,
                    fill: None,
                    stroke: Some(stroke.as_stroke(ctx.theme())),
                    transform: None,
                };
                self.draw_path(&path)
            }
            Some(ir::plot::Border::AxisArrow { .. }) => {
                todo!("Draw axis arrow")
            }
        }
    }

    fn draw_plot_series<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        ir_series: &[ir::Series],
        series: &[Series],
        rect: &geom::Rect,
        axes: &Axes,
    ) -> Result<(), Error>
    where
        D: data::Source,
        T: Theme,
    {
        self.push_clip(&render::Clip {
            path: &rect.to_path(),
            transform: None,
        })?;

        let cm = CoordMapXy {
            x: &axes.x,
            y: &axes.y,
        };

        for (ir_series, series) in ir_series.iter().zip(series.iter()) {
            self.draw_series_plot(ctx, ir_series, series, rect, &cm)?;
        }
        self.pop_clip()?;
        Ok(())
    }

    fn draw_x_axis<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        x_axis: &Axis,
        rect: &geom::Rect,
    ) -> Result<(), render::Error>
    where
        T: Theme,
    {
        if let Some(x_min_ticks) = x_axis.minor_ticks.as_ref() {
            let transform = geom::Transform::from_translate(rect.left(), rect.bottom());
            let ticks = x_min_ticks
                .locs
                .iter()
                .copied()
                .map(|t| x_axis.map_coord_num(t));
            self.draw_ticks_path(
                ticks,
                missing_params::MINOR_TICK_SIZE,
                render::Stroke {
                    color: x_min_ticks.color.resolve(ctx.theme()),
                    width: 1.0,
                    pattern: Default::default(),
                },
                &transform,
            )?;
        }
        let mut label_y = rect.bottom() + missing_params::AXIS_LABEL_MARGIN;
        if let Some(x_ticks) = x_axis.ticks.as_ref() {
            self.draw_x_ticks(ctx, &rect, x_ticks, x_axis)?;
            label_y +=
                missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN + x_ticks.font.size;
        }
        if let Some(label) = &x_axis.label {
            let text = render::TextLayout {
                layout: label,
                fill: ctx.theme().foreground().into(),
                transform: Some(&Transform::from_translate(rect.center_x(), label_y)),
            };
            self.draw_text_layout(&text)?;
        }
        Ok(())
    }

    fn draw_x_ticks<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        rect: &geom::Rect,
        x_ticks: &Ticks,
        x_cm: &dyn scale::CoordMap,
    ) -> Result<(), render::Error>
    where
        T: style::Theme,
    {
        let transform = geom::Transform::from_translate(rect.left(), rect.bottom());
        let ticks = x_ticks.locs.iter().map(|t| {
            x_cm.map_coord(t.as_sample())
                .expect("Ticks should be valid coord")
        });
        self.draw_ticks_path(
            ticks,
            missing_params::TICK_SIZE,
            render::Stroke {
                color: x_ticks.color.resolve(ctx.theme()),
                width: 1.0,
                pattern: Default::default(),
            },
            &transform,
        )?;

        let fill = x_ticks.color.resolve(ctx.theme()).into();

        for (xt, lbl) in x_ticks.locs.iter().zip(x_ticks.lbls.iter()) {
            let x = rect.left()
                + x_cm
                    .map_coord(xt.as_sample())
                    .expect("Ticks should be valid coord");
            let y = rect.bottom() + missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN;
            let text = render::TextLayout {
                layout: lbl,
                fill,
                transform: Some(&Transform::from_translate(x, y)),
            };
            self.draw_text_layout(&text)?;
        }

        if let Some(annot) = x_ticks.annot.as_ref() {
            let font = x_ticks.font.font.clone().with_families(
                style::font::parse_font_families(missing_params::AXIS_ANNOT_FONT_FAMILY).unwrap(),
            );
            let pos = geom::Point::new(
                rect.right(),
                rect.bottom()
                    + missing_params::TICK_SIZE
                    + missing_params::TICK_LABEL_MARGIN
                    + x_ticks.font.size,
            );
            let options = text::layout::Options {
                hor_align: text::HorAlign::Right,
                ver_align: text::LineVerAlign::Hanging.into(),
                ..Default::default()
            };
            let text = render::Text {
                text: annot.as_str(),
                font: &font,
                font_size: x_ticks.font.size,
                fill,
                options,
                transform: Some(&pos.translation()),
            };
            self.draw_text(&text)?;
        }

        Ok(())
    }

    fn draw_y_axis<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        y_axis: &Axis,
        rect: &geom::Rect,
    ) -> Result<(), render::Error>
    where
        T: style::Theme,
    {
        if let Some(y_min_ticks) = y_axis.minor_ticks.as_ref() {
            let transform =
                geom::Transform::from_translate(rect.left(), rect.bottom()).pre_rotate(-90.0);
            let ticks = y_min_ticks
                .locs
                .iter()
                .copied()
                .map(|t| y_axis.map_coord_num(t));
            self.draw_ticks_path(
                ticks,
                missing_params::MINOR_TICK_SIZE,
                render::Stroke {
                    color: y_min_ticks.color.resolve(ctx.theme()),
                    width: 1.0,
                    pattern: Default::default(),
                },
                &transform,
            )?;
        }
        if let Some(y_ticks) = y_axis.ticks.as_ref() {
            self.draw_y_ticks(ctx, rect, y_ticks, y_axis)?;
        }
        if let Some(label) = y_axis.label.as_ref() {
            // we render at origin, but translate to correct position and rotate
            let tx = rect.left() - y_axis.ortho_sz + missing_params::AXIS_LABEL_MARGIN;
            let ty = rect.center_y();
            let transform = geom::Transform::from_translate(tx, ty).pre_rotate(-90.0);
            let text = render::TextLayout {
                layout: label,
                fill: ctx.theme().foreground().into(),
                transform: Some(&transform),
            };
            self.draw_text_layout(&text)?;
        }
        Ok(())
    }

    fn draw_y_ticks<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        rect: &geom::Rect,
        y_ticks: &Ticks,
        y_cm: &dyn CoordMap,
    ) -> Result<(), render::Error>
    where
        T: style::Theme,
    {
        let transform =
            geom::Transform::from_translate(rect.left(), rect.bottom()).pre_rotate(-90.0);
        let ticks = y_ticks.locs.iter().map(|t| {
            y_cm.map_coord(t.as_sample())
                .expect("Ticks should be valid coord")
        });
        self.draw_ticks_path(
            ticks,
            missing_params::TICK_SIZE,
            render::Stroke {
                color: y_ticks.color.resolve(ctx.theme()).into(),
                width: 1.0,
                pattern: Default::default(),
            },
            &transform,
        )?;

        let fill = y_ticks.color.resolve(ctx.theme()).into();

        for (yt, lbl) in y_ticks.locs.iter().zip(y_ticks.lbls.iter()) {
            let x = rect.left() - missing_params::TICK_SIZE - missing_params::TICK_LABEL_MARGIN;
            let y = rect.bottom()
                - y_cm
                    .map_coord(yt.as_sample())
                    .expect("Ticks should be valid coord");
            let pos = geom::Point::new(x, y);
            let text = render::TextLayout {
                layout: lbl,
                fill,
                transform: Some(&pos.translation()),
            };
            self.draw_text_layout(&text)?;
        }
        Ok(())
    }

    fn draw_ticks_path<T>(
        &mut self,
        ticks: T,
        size: f32,
        stroke: render::Stroke,
        transform: &geom::Transform,
    ) -> Result<(), render::Error>
    where
        T: IntoIterator<Item = f32>,
    {
        let ticks_path = ticks_path(ticks, size);
        let ticks_path = render::Path {
            path: &ticks_path,
            fill: None,
            stroke: Some(stroke),
            transform: Some(transform),
        };
        self.draw_path(&ticks_path)?;
        Ok(())
    }
}

/// Build the ticks path along X axis.
/// Y axis will use the same function and rotate 90°
fn ticks_path<T>(ticks: T, size: f32) -> geom::Path
where
    T: IntoIterator<Item = f32>,
{
    let mut path = geom::PathBuilder::new();
    for tick in ticks {
        path.move_to(tick, -size);
        path.line_to(tick, size);
    }
    path.finish().expect("Should be a valid path")
}
