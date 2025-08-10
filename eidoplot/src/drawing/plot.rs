use crate::data;
use crate::drawing::legend::Legend;
use crate::drawing::series::{series_has_legend, Series};
use crate::drawing::{Ctx, scale, ticks};
use crate::geom;
use crate::ir;
use crate::missing_params;
use crate::render::{self, Surface};
use crate::style::{self, defaults};

use scale::{CoordMap, CoordMapXy};

struct Ticks {
    locs: Vec<f64>,
    lbls: Vec<String>,
    annot: Option<String>,
    font: style::Font,
    color: style::Color,
    grid: Option<style::Line>,
}

struct Axis {
    ortho_sz: f32,
    coord_map: Box<dyn CoordMap>,
    ticks: Option<Ticks>,
    label: Option<(String, style::Font)>,
}

impl CoordMap for Axis {
    fn map_coord(&self, v: f64) -> f32 {
        self.coord_map.map_coord(v)
    }
    fn view_bounds(&self) -> data::ViewBounds {
        self.coord_map.view_bounds()
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

fn plot_insets(plot: &ir::Plot) -> (f32, f32) {
    match plot.insets {
        Some(ir::plot::Insets::Fixed(x, y)) => (x, y),
        Some(ir::plot::Insets::Auto) => auto_insets(plot),
        None => (0.0, 0.0),
    }
}

fn auto_insets(plot: &ir::Plot) -> (f32, f32) {
    for s in plot.series.iter() {
        match &s.plot {
            ir::plot::SeriesPlot::Histogram(..) => return defaults::PLOT_HIST_AUTO_INSETS,
            _ => (),
        }
    }
    defaults::PLOT_XY_AUTO_INSETS
}

fn setup_ticks(ticks: &ir::axis::Ticks, vb: data::ViewBounds) -> Ticks {
    let mut locs = ticks::locate(ticks.locator(), vb);
    locs.retain(|l| vb.contains(*l));
    let lbl_formatter = ticks::label_formatter(ticks, vb);
    let lbls = locs
        .iter()
        .copied()
        .map(|l| lbl_formatter.format_label(l))
        .collect();
    let annot = lbl_formatter.axis_annotation().map(String::from);
    Ticks {
        locs,
        lbls,
        annot,
        font: ticks.font().clone(),
        color: ticks.color(),
        grid: ticks.grid().copied(),
    }
}

impl<'a, S> Ctx<'a, S>
where
    S: render::Surface,
{
    pub fn draw_plot(&mut self, plot: &ir::Plot, rect: &geom::Rect) -> Result<(), S::Error> {
        let rect = {
            let mut rect = rect.pad(&missing_params::PLOT_PADDING);

            // draw outer legend and adjust rect
            if let Some(legend) = &plot.legend {
                if !legend.pos().is_inside() {
                    self.draw_plot_outer_legend(plot, legend, &mut rect)?;
                }
            }
            rect
        };

        let series = self.setup_plot_series(plot);
        let vb = Series::unite_view_bounds(&series);
        let axes = self.setup_plot_axes(plot, vb, &rect);

        let rect = rect
            .shifted_left_side(axes.y_width())
            .shifted_bottom_side(-axes.x_height());

        self.draw_plot_background(plot, &rect)?;
        self.draw_grid(&axes, &rect)?;
        self.draw_plot_series(&series, &rect, &axes)?;
        self.draw_x_axis(&axes.x, &rect)?;
        self.draw_y_axis(&axes.y, &rect)?;
        self.draw_plot_border(plot.border.as_ref(), &rect)?;

        Ok(())
    }

    fn setup_plot_axes(&mut self, plot: &ir::Plot, vb: (data::ViewBounds, data::ViewBounds), rect: &geom::Rect) -> Axes {
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

        let y_cm = scale::map_scale_coord(plot.y_axis.scale(), rect.height(), vb.1, insets.1);
        let y_axis = self.setup_y_axis(&plot.y_axis, y_cm);
        let rect = rect.shifted_left_side(y_axis.ortho_sz);

        let x_cm = scale::map_scale_coord(plot.x_axis.scale(), rect.width(), vb.0, insets.0);
        let x_axis = self.setup_x_axis(&plot.x_axis, x_cm, x_height);

        Axes {
            x: x_axis,
            y: y_axis,
        }
    }

    fn calculate_x_axis_height(&self, x_axis: &ir::Axis) -> f32 {
        let mut height = 0.0;
        if let Some(ticks) = x_axis.ticks() {
            height +=
                missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN + ticks.font().size();
        }
        if let Some(label) = x_axis.label() {
            let fs = label
                .font()
                .map(|f| f.size())
                .unwrap_or(defaults::AXIS_LABEL_FONT_SIZE);
            height += 2.0 * missing_params::AXIS_LABEL_MARGIN + fs;
        }
        height
    }

    // TODO: When pxl draws on its own rather than using resvg,
    // this function should return the calculated shapes and cache them in the render::Text
    // and send them to the surface for reuse
    fn calculate_y_axis_width(&self, y_axis: &ir::Axis, y_ticks: Option<&Ticks>) -> f32 {
        let mut width = 0.0;
        if let Some(label) = y_axis.label() {
            let fs = label
                .font()
                .map(|f| f.size())
                .unwrap_or(defaults::AXIS_LABEL_FONT_SIZE);
            width += 2.0 * missing_params::AXIS_LABEL_MARGIN + fs;
        }
        if let Some(ticks) = y_ticks {
            let max_w = self
                .fontdb()
                .max_labels_width(ticks.lbls.iter(), &ticks.font);
            width += missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN + max_w;
        }
        width
    }

    fn setup_y_axis(&mut self, y_axis: &ir::Axis, coord_map: Box<dyn CoordMap>) -> Axis {
        let ticks = y_axis
            .ticks()
            .map(|t| setup_ticks(t, coord_map.view_bounds()));

        let y_width = self.calculate_y_axis_width(y_axis, ticks.as_ref());

        let label = y_axis.label().map(|l| {
            (
                l.text().to_string(),
                l.font().cloned().unwrap_or_else(|| {
                    style::Font::new(
                        defaults::AXIS_LABEL_FONT_FAMILY.into(),
                        defaults::AXIS_LABEL_FONT_SIZE,
                    )
                }),
            )
        });

        Axis {
            ortho_sz: y_width,
            coord_map,
            ticks,
            label,
        }
    }

    fn setup_x_axis(
        &mut self,
        x_axis: &ir::Axis,
        coord_map: Box<dyn CoordMap>,
        x_height: f32,
    ) -> Axis {
        let ticks = x_axis
            .ticks()
            .map(|t| setup_ticks(t, coord_map.view_bounds()));

        let label = x_axis.label().map(|l| {
            (
                l.text().to_string(),
                l.font().cloned().unwrap_or_else(|| {
                    style::Font::new(
                        defaults::AXIS_LABEL_FONT_FAMILY.into(),
                        defaults::AXIS_LABEL_FONT_SIZE,
                    )
                }),
            )
        });

        Axis {
            ortho_sz: x_height,
            coord_map,
            ticks,
            label,
        }
    }

    fn setup_plot_series<'b>(&mut self, plot: &'b ir::Plot) -> Vec<Series<'b>> {
        plot.series
            .iter()
            .map(|s| Series::from_ir(s))
            .collect()
    }
}

impl<'a, S> Ctx<'a, S>
where
    S: render::Surface,
{
    fn draw_plot_outer_legend(
        &mut self,
        plot: &ir::Plot,
        legend: &ir::Legend,
        rect: &mut geom::Rect,
    ) -> Result<(), S::Error> {
        let mut dlegend = Legend::from_ir(legend, rect.width(), self.fontdb().clone());
        for s in plot.series.iter() {
            if series_has_legend(s) {
                dlegend.add_entry(s);
            }
        }
        let sz = dlegend.layout();
        let top_left = match legend.pos() {
            ir::legend::Pos::OutTop => {
                rect.shift_top_side(sz.height());
                geom::Point::new(rect.center_x() - sz.width() / 2.0, rect.top() - sz.height())
            }
            ir::legend::Pos::OutRight => {
                rect.shift_right_side(-sz.width());
                geom::Point::new(rect.right(), rect.center_y() - sz.height() / 2.0)
            }
            ir::legend::Pos::OutBottom => {
                rect.shift_bottom_side(-sz.height());
                geom::Point::new(rect.center_x() - sz.width() / 2.0, rect.bottom())
            }
            ir::legend::Pos::OutLeft => {
                rect.shift_left_side(sz.width());
                geom::Point::new(
                    rect.left() - sz.width(),
                    rect.center_y() - sz.height() / 2.0,
                )
            }
            _ => unreachable!(),
        };
        self.draw_legend(&dlegend, &top_left)
    }

    fn draw_plot_background(&mut self, plot: &ir::Plot, rect: &geom::Rect) -> Result<(), S::Error> {
        if let Some(fill) = plot.fill.as_ref() {
            self.draw_rect(&render::Rect {
                rect: *rect,
                fill: Some(fill.clone()),
                stroke: None,
                transform: None,
            })?;
        }
        Ok(())
    }

    fn draw_grid(&mut self, axes: &Axes, rect: &geom::Rect) -> Result<(), S::Error> {
        if let Some(x_ticks) = axes.x.ticks.as_ref() {
            if let Some(x_grid) = x_ticks.grid {
                let mut pathb = geom::PathBuilder::with_capacity(2, 2);
                for t in x_ticks.locs.iter().copied() {
                    let x = axes.x.map_coord(t) + rect.left();
                    pathb.move_to(x, rect.top());
                    pathb.line_to(x, rect.bottom());
                    let path = pathb.finish().expect("Should be a valid path");
                    let rpath = render::Path {
                        path: &path,
                        fill: None,
                        stroke: Some(x_grid.clone()),
                        transform: None,
                    };
                    self.draw_path(&rpath)?;
                    pathb = path.clear();
                }
            }
        }
        if let Some(y_ticks) = axes.y.ticks.as_ref() {
            if let Some(y_grid) = y_ticks.grid {
                let mut pathb = geom::PathBuilder::with_capacity(2, 2);
                for t in y_ticks.locs.iter().copied() {
                    let y = rect.bottom() - axes.y.map_coord(t);
                    pathb.move_to(rect.left(), y);
                    pathb.line_to(rect.right(), y);
                    let path = pathb.finish().expect("Should be a valid path");
                    let pathr = render::Path {
                        path: &path,
                        fill: None,
                        stroke: Some(y_grid.clone()),
                        transform: None,
                    };
                    self.draw_path(&pathr)?;
                    pathb = path.clear();
                }
            }
        }
        Ok(())
    }

    fn draw_plot_border(
        &mut self,
        border: Option<&ir::plot::Border>,
        rect: &geom::Rect,
    ) -> Result<(), S::Error> {
        match border {
            None => Ok(()),
            Some(ir::plot::Border::Box(stroke)) => self.draw_rect(&render::Rect {
                rect: *rect,
                fill: None,
                stroke: Some(stroke.clone()),
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
                    stroke: Some(stroke.clone()),
                    transform: None,
                };
                self.draw_path(&path)
            }
            Some(ir::plot::Border::AxisArrow { .. }) => {
                todo!("Draw axis arrow")
            }
        }
    }

    fn draw_plot_series(
        &mut self,
        series: &[Series],
        rect: &geom::Rect,
        axes: &Axes,
    ) -> Result<(), S::Error> {
        self.push_clip(&render::Clip {
            path: &rect.to_path(),
            transform: None,
        })?;

        let cm = CoordMapXy {
            x: &axes.x,
            y: &axes.y,
        };

        for series in series {
            self.draw_series_plot(series, rect, &cm)?;
        }
        self.pop_clip()?;
        Ok(())
    }
}

impl<'a, S> Ctx<'a, S>
where
    S: render::Surface,
{
    fn draw_x_axis(&mut self, x_axis: &Axis, rect: &geom::Rect) -> Result<(), S::Error> {
        let mut label_y = rect.bottom() + missing_params::AXIS_LABEL_MARGIN;
        if let Some(x_ticks) = x_axis.ticks.as_ref() {
            self.draw_x_ticks(&rect, x_ticks, x_axis)?;
            label_y +=
                missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN + x_ticks.font.size();
        }
        if let Some(label) = &x_axis.label {
            let anchor = render::TextAnchor {
                pos: geom::Point::new(rect.center_x(), label_y),
                align: render::TextAlign::Center,
                baseline: render::TextBaseline::Hanging,
            };
            let text = render::Text {
                text: label.0.as_str(),
                font: &label.1,
                anchor,
                fill: missing_params::AXIS_LABEL_COLOR.into(),
                transform: None,
            };
            self.draw_text(&text)?;
        }
        Ok(())
    }

    fn draw_x_ticks(
        &mut self,
        rect: &geom::Rect,
        x_ticks: &Ticks,
        x_cm: &dyn scale::CoordMap,
    ) -> Result<(), S::Error> {
        let transform = geom::Transform::from_translate(rect.left(), rect.bottom());
        self.draw_ticks_path(&x_ticks.locs, x_cm, &transform)?;

        let fill = x_ticks.color.into();

        for (xt, lbl) in x_ticks.locs.iter().copied().zip(x_ticks.lbls.iter()) {
            let x = x_cm.map_coord(xt);
            let x = rect.left() + x;
            let pos = geom::Point::new(
                x,
                rect.bottom() + missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN,
            );
            let anchor = render::TextAnchor {
                pos,
                align: render::TextAlign::Center,
                baseline: render::TextBaseline::Hanging,
            };
            let text = render::Text {
                text: lbl.as_str(),
                font: &x_ticks.font,
                anchor,
                fill,
                transform: None,
            };

            self.draw_text(&text)?;
        }

        if let Some(annot) = x_ticks.annot.as_ref() {
            let font = x_ticks
                .font
                .clone()
                .with_family(missing_params::AXIS_ANNOT_FONT_FAMILY.into());
            let pos = geom::Point::new(
                rect.right(),
                rect.bottom()
                    + missing_params::TICK_SIZE
                    + missing_params::TICK_LABEL_MARGIN
                    + font.size(),
            );
            let anchor = render::TextAnchor {
                pos,
                align: render::TextAlign::End,
                baseline: render::TextBaseline::Hanging,
            };
            let text = render::Text {
                text: annot.as_str(),
                font: &font,
                anchor,
                fill,
                transform: None,
            };
            self.draw_text(&text)?;
        }

        Ok(())
    }

    fn draw_y_axis(&mut self, y_axis: &Axis, rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        if let Some(y_ticks) = y_axis.ticks.as_ref() {
            self.draw_y_ticks(rect, y_ticks, y_axis)?;
        }
        if let Some(label) = y_axis.label.as_ref() {
            // we render at origin, but translate to correct position and rotate
            let anchor = render::TextAnchor {
                pos: geom::Point::ORIGIN,
                align: render::TextAlign::Center,
                baseline: render::TextBaseline::Hanging,
            };

            let tx = rect.left() - y_axis.ortho_sz + missing_params::AXIS_LABEL_MARGIN;
            let ty = rect.center_y();
            let transform = geom::Transform::from_translate(tx, ty).pre_rotate(90.0);
            let text = render::Text {
                text: label.0.as_str(),
                font: &label.1,
                anchor,
                fill: missing_params::AXIS_LABEL_COLOR.into(),
                transform: Some(&transform),
            };
            self.draw_text(&text)?;
        }
        Ok(())
    }

    fn draw_y_ticks(
        &mut self,
        rect: &geom::Rect,
        y_ticks: &Ticks,
        y_cm: &dyn CoordMap,
    ) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        let transform =
            geom::Transform::from_translate(rect.left(), rect.bottom()).pre_rotate(90.0);
        self.draw_ticks_path(&y_ticks.locs, y_cm, &transform)?;

        let fill = y_ticks.color.into();

        for (yt, lbl) in y_ticks.locs.iter().copied().zip(y_ticks.lbls.iter()) {
            let x = rect.left() - missing_params::TICK_SIZE - missing_params::TICK_LABEL_MARGIN;
            let y = rect.bottom() - y_cm.map_coord(yt);
            let pos = geom::Point::new(x, y);
            let anchor = render::TextAnchor {
                pos,
                align: render::TextAlign::End,
                baseline: render::TextBaseline::Center,
            };
            let text = render::Text {
                text: lbl.as_str(),
                font: &y_ticks.font,
                anchor,
                fill,
                transform: None,
            };
            self.draw_text(&text)?;
        }
        Ok(())
    }

    fn draw_ticks_path(
        &mut self,
        ticks: &[f64],
        cm: &dyn CoordMap,
        transform: &geom::Transform,
    ) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        let ticks_path = ticks_path(&ticks, cm, None);
        let ticks_path = render::Path {
            path: &ticks_path,
            fill: None,
            stroke: Some(missing_params::TICK_COLOR.into()),
            transform: Some(transform),
        };
        self.draw_path(&ticks_path)?;
        Ok(())
    }
}

/// Build the ticks path along X axis.
/// Y axis will use the same function and rotate 90°
fn ticks_path(
    ticks: &[f64],
    cm: &dyn scale::CoordMap,
    reuse_alloc: Option<geom::Path>,
) -> geom::Path {
    let sz = missing_params::TICK_SIZE;
    let mut path = reuse_alloc
        .map(|p| p.clear())
        .unwrap_or_else(geom::PathBuilder::new);
    let vb = cm.view_bounds();
    for tick in ticks {
        if !vb.contains(*tick) {
            continue;
        }
        let x = cm.map_coord(*tick);
        path.move_to(x, -sz);
        path.line_to(x, sz);
    }
    path.finish().expect("Should be a valid path")
}
