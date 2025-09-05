use scale::CoordMapXy;

use crate::drawing::legend::Legend;
use crate::drawing::series::{Series, SeriesExt};
use crate::drawing::{Ctx, Error, SurfWrapper, axis, scale};
use crate::render::{self, Surface as _};
use crate::style::{self, Theme, defaults};
use crate::{data, geom, ir, missing_params};

fn plot_insets(plot: &ir::Plot) -> geom::Padding {
    match plot.insets() {
        Some(&ir::plot::Insets::Fixed(x, y)) => geom::Padding::Center { v: y, h: x },
        Some(ir::plot::Insets::Auto) => auto_insets(plot),
        None => geom::Padding::Even(0.0),
    }
}

fn auto_insets(plot: &ir::Plot) -> geom::Padding {
    for s in plot.series() {
        match s {
            ir::Series::Histogram(..) => return defaults::PLOT_VER_BARS_AUTO_INSETS,
            ir::Series::Bars(..) => return defaults::PLOT_VER_BARS_AUTO_INSETS,
            ir::Series::BarsGroup(bg) if bg.orientation().is_vertical() => {
                return defaults::PLOT_VER_BARS_AUTO_INSETS;
            }
            ir::Series::BarsGroup(bg) if bg.orientation().is_horizontal() => {
                return defaults::PLOT_HOR_BARS_AUTO_INSETS;
            }
            _ => (),
        }
    }
    defaults::PLOT_XY_AUTO_INSETS
}

pub fn for_each_series<F>(plot: &ir::Plot, mut f: F) -> Result<(), Error>
where
    F: FnMut(&dyn SeriesExt) -> Result<(), Error>,
{
    for s in plot.series() {
        match &s {
            ir::Series::Line(line) => f(line)?,
            ir::Series::Scatter(scatter) => f(scatter)?,
            ir::Series::Histogram(hist) => f(hist)?,
            ir::Series::Bars(bars) => f(bars)?,
            ir::Series::BarsGroup(bars_group) => {
                for bs in bars_group.series() {
                    f(bs)?
                }
            }
        }
    }
    Ok(())
}

impl<D, T> Ctx<'_, D, T> {
    fn setup_plot_axes(
        &self,
        plot: &ir::Plot,
        ab: (&axis::Bounds, &axis::Bounds),
        rect: &geom::Rect,
    ) -> Result<axis::Axes, Error> {
        let insets = plot_insets(plot);

        // bootstrapping the axes by setting the vertical axis with estimated height
        // from the font size of the horizontal axis (for which text width doesn't matter)

        let estimated_height = rect.height() - self.estimate_hor_axis_height(plot.x_axis());
        let mut left_axis = self.setup_axis(
            plot.y_axis(),
            ab.1,
            axis::Side::Left,
            estimated_height,
            &insets,
        )?;

        let width = rect.width() - left_axis.size_across();
        let bottom_axis =
            self.setup_axis(plot.x_axis(), ab.0, axis::Side::Bottom, width, &insets)?;

        // now we correct the vertical axis according the real height of the horizontal axis
        let height = rect.height() - bottom_axis.size_across();
        left_axis.set_size_along(height);

        Ok(axis::Axes {
            left: left_axis,
            bottom: bottom_axis,
        })
    }

    fn estimate_hor_axis_height(&self, x_axis: &ir::Axis) -> f32 {
        let mut height = 0.0;
        if let Some(ticks) = x_axis.ticks() {
            height +=
                missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN + ticks.font().size;
        }
        if let Some(label) = x_axis.title() {
            height += 2.0 * missing_params::AXIS_TITLE_MARGIN + label.font().size;
        }
        height
    }
}

impl<D, T> Ctx<'_, D, T>
where
    D: data::Source,
{
    fn setup_plot_series(&self, plot: &ir::Plot) -> Result<Vec<Series>, Error> {
        plot.series()
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
            if let Some(legend) = plot.legend() {
                if !legend.pos().is_inside() {
                    self.draw_plot_outer_legend(ctx, plot, legend, &mut rect)?;
                }
            }
            rect
        };

        let series = ctx.setup_plot_series(plot)?;
        if series.is_empty() {
            self.draw_plot_background(ctx, plot, &rect)?;
            self.draw_plot_border(ctx, plot.border(), &rect)?;
            return Ok(());
        }

        let (x_bounds, y_bounds) = Series::unite_bounds(&series)?.ok_or(Error::UnboundedAxis)?;

        let axes = ctx.setup_plot_axes(plot, (&x_bounds, &y_bounds), &rect)?;

        let rect = rect.pad(&axes.total_plot_padding());

        self.draw_plot_background(ctx, plot, &rect)?;
        self.draw_axes_grids(ctx, &axes, &rect)?;
        self.draw_plot_series(ctx, plot.series(), &series, &rect, &axes)?;
        self.draw_axes(ctx, &axes, &rect)?;
        self.draw_plot_border(ctx, plot.border(), &rect)?;

        if let Some(legend) = plot.legend() {
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

        let mut idx = 0;
        for_each_series(plot, |s| {
            if let Some(entry) = s.legend_entry() {
                dlegend.add_entry(idx, entry)?;
                idx += 1;
            }
            Ok(())
        })?;

        let Some(sz) = dlegend.layout() else {
            return Ok(());
        };
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

        let mut idx = 0;
        for_each_series(plot, |s| {
            if let Some(entry) = s.legend_entry() {
                dlegend.add_entry(idx, entry)?;
                idx += 1;
            }
            Ok(())
        })?;

        let Some(sz) = dlegend.layout() else {
            return Ok(());
        };

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
        if let Some(fill) = plot.fill() {
            self.draw_rect(&render::Rect {
                rect: *rect,
                fill: Some(fill.as_paint(ctx.theme())),
                stroke: None,
                transform: None,
            })?;
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
        axes: &axis::Axes,
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
            x: axes.bottom.coord_map(),
            y: axes.left.coord_map(),
        };

        for (ir_series, series) in ir_series.iter().zip(series.iter()) {
            self.draw_series_plot(ctx, ir_series, series, rect, &cm)?;
        }
        self.pop_clip()?;
        Ok(())
    }
}
