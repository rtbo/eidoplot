use core::slice;
use std::sync::Arc;

use crate::drawing::axis::{self, Axis, AxisScale};
use crate::drawing::legend::{Legend, LegendBuilder};
use crate::drawing::series::{Series, SeriesExt};
use crate::drawing::{Ctx, Error, SurfWrapper, scale};
use crate::render::{self, Surface};
use crate::style::defaults;
use crate::{data, geom, ir, missing_params};

mod grid_idx;
use grid_idx::GridIdx;

#[derive(Debug, Clone)]
pub struct Plots {
    plots: Vec<Plot>,
}

#[derive(Debug, Clone)]
struct Plot {
    plot_rect: geom::Rect,
    outer_rect: geom::Rect,
    // None when there is no series (empty plot)
    axes: Option<Axes>,
    series: Vec<Series>,
    legend: Option<Legend>,
}

#[derive(Debug, Clone)]
struct Axes {
    left: Axis,
    bottom: Axis,
}

/// Plot itermediate data during setup phase
#[derive(Debug, Clone)]
struct PlotData {
    series: Vec<Series>,
    legend: Option<Legend>,
    insets: geom::Padding,
}

pub trait IrPlotsExt {
    fn len(&self) -> usize {
        self.plots().len()
    }
    fn plots(&self) -> &[ir::Plot];
    fn cols(&self) -> usize;
    fn rows(&self) -> usize;
    fn space(&self) -> f32;
    fn share_x(&self) -> bool;
    fn share_y(&self) -> bool;
}

impl IrPlotsExt for ir::figure::Plots {
    fn plots(&self) -> &[ir::Plot] {
        match self {
            ir::figure::Plots::Plot(ir_plot) => slice::from_ref(ir_plot),
            ir::figure::Plots::Subplots(ir_subplots) => ir_subplots.plots(),
        }
    }

    fn cols(&self) -> usize {
        match self {
            ir::figure::Plots::Plot(_) => 1,
            ir::figure::Plots::Subplots(ir_subplots) => ir_subplots.cols() as usize,
        }
    }
    fn rows(&self) -> usize {
        match self {
            ir::figure::Plots::Plot(_) => 1,
            ir::figure::Plots::Subplots(ir_subplots) => ir_subplots.rows() as usize,
        }
    }
    fn space(&self) -> f32 {
        match self {
            ir::figure::Plots::Plot(_) => 0.0,
            ir::figure::Plots::Subplots(ir_subplots) => ir_subplots.space(),
        }
    }
    fn share_x(&self) -> bool {
        match self {
            ir::figure::Plots::Plot(_) => false,
            ir::figure::Plots::Subplots(ir_subplots) => ir_subplots.share_x(),
        }
    }
    fn share_y(&self) -> bool {
        match self {
            ir::figure::Plots::Plot(_) => false,
            ir::figure::Plots::Subplots(ir_subplots) => ir_subplots.share_y(),
        }
    }
}

impl<D> Ctx<'_, D>
where
    D: data::Source,
{
    /// Setup a collection of plots, given an IR representation of the plots
    /// and a bounding rectangle.
    pub fn setup_plots<P>(&self, ir_plots: &P, rect: &geom::Rect) -> Result<Plots, Error>
    where
        P: IrPlotsExt,
    {
        // We build all needed characteristics by the plots one after another.
        // Each characteristic (axes, interspace etc.) is in vector, indexed in the
        // same order than the plots

        // PlotData contains all data that is not impacted by the size of axes
        let plot_data = self.setup_plot_data(ir_plots, rect)?;

        // Estimate the space taken by all horizontal axes
        // Can be slightly wrong if font metrics height isn't exactly font size.
        // This will be fixed at end of the setup phase.
        let bottom_heights = self.calc_estimated_bottom_heights(ir_plots, &plot_data);
        let top_heights = self.calc_top_heights(ir_plots, &plot_data);
        let hor_space_height = bottom_heights.iter().sum::<f32>()
            + top_heights.iter().sum::<f32>()
            + ir_plots.space() * (ir_plots.rows() - 1) as f32;

        // Now we can determine length of vertical axes and set them all up
        let subplot_rect_height = (rect.height() - hor_space_height) / ir_plots.rows() as f32;
        let y_axes = self.setup_y_axes(ir_plots, &plot_data, subplot_rect_height)?;

        // Now we calculate the interspace between vertical axes
        let left_widths = self.calc_left_widths(ir_plots, &plot_data, &y_axes);
        let right_widths = self.calc_right_widths(ir_plots, &plot_data);
        let vert_space_width = left_widths.iter().sum::<f32>()
            + right_widths.iter().sum::<f32>()
            + ir_plots.space() * (ir_plots.cols() - 1) as f32;

        // Now we can determine width of horizontal axes and set them all up
        let subplot_rect_width = (rect.width() - vert_space_width) / ir_plots.cols() as f32;
        let x_axes = self.setup_x_axes(ir_plots, &plot_data, subplot_rect_width)?;

        // bottom heights were estimated, we can now calculate them and fix the y-axes heights
        let bottom_heights = self.calc_bottom_heights(ir_plots, &plot_data, &x_axes);
        let hor_space_height = bottom_heights.iter().sum::<f32>()
            + top_heights.iter().sum::<f32>()
            + ir_plots.space() * (ir_plots.rows() - 1) as f32;
        let subplot_rect_height = (rect.height() - hor_space_height) / ir_plots.rows() as f32;
        let y_axes = self.setup_y_axes(ir_plots, &plot_data, subplot_rect_height)?;

        // Everything is now ready to setup all plots
        let gi = GridIdx::from(ir_plots);
        let mut plots: Vec<Option<Plot>> = vec![None; gi.len()];
        let mut data = plot_data.into_iter();
        let mut x_axes = x_axes.into_iter();
        let mut y_axes = y_axes.into_iter();

        let mut y = rect.y();
        for row in 0..gi.rows() {
            let height = subplot_rect_height + top_heights[row] + bottom_heights[row];
            let mut x = rect.x();

            for col in 0..gi.cols() {
                let width = subplot_rect_width + left_widths[col] + right_widths[col];
                if let Some(plt_idx) = gi.plot_idx(row, col) {
                    let outer_rect = geom::Rect::from_xywh(x, y, width, height);
                    let plot_rect = geom::Rect::from_xywh(
                        x + left_widths[col],
                        y + top_heights[col],
                        subplot_rect_width,
                        subplot_rect_height,
                    );
                    let PlotData { series, legend, .. } = data.next().unwrap();
                    let x_axis = x_axes.next().unwrap();
                    let y_axis = y_axes.next().unwrap();
                    let axes = match (x_axis, y_axis) {
                        (Some(x_axis), Some(y_axis)) => Some(Axes {
                            bottom: x_axis,
                            left: y_axis,
                        }),
                        (None, None) => None,
                        _ => unreachable!(
                            "axis are None when there is no series, so should be both None or both Some"
                        ),
                    };
                    plots[plt_idx] = Some(Plot {
                        plot_rect,
                        outer_rect,
                        axes,
                        series,
                        legend,
                    });
                }
                x += width + ir_plots.space();
            }

            y += height + ir_plots.space();
        }

        let plots = plots.into_iter().map(|p| p.unwrap()).collect();
        Ok(Plots { plots })
    }

    fn setup_plot_data(
        &self,
        ir_plots: &impl IrPlotsExt,
        rect: &geom::Rect,
    ) -> Result<Vec<PlotData>, Error> {
        let mut plot_data = Vec::with_capacity(ir_plots.plots().len());
        for ir_plot in ir_plots.plots().iter() {
            let series = self.setup_plot_series(ir_plot)?;
            let cols = ir_plots.cols() as f32;
            let avail_width = (rect.width() - ir_plots.space() * (cols - 1.0)) / cols;
            let legend = self.setup_plot_legend(ir_plot, avail_width)?;
            let insets = plot_insets(ir_plot);
            plot_data.push(PlotData {
                series,
                legend,
                insets,
            });
        }
        Ok(plot_data)
    }

    fn setup_plot_series(&self, plot: &ir::Plot) -> Result<Vec<Series>, Error> {
        plot.series()
            .iter()
            .enumerate()
            .map(|(index, s)| Series::from_ir(index, s, self.data_source()))
            .collect()
    }

    fn setup_plot_legend(
        &self,
        ir_plot: &ir::Plot,
        avail_width: f32,
    ) -> Result<Option<Legend>, Error> {
        let Some(ir_leg) = ir_plot.legend() else {
            return Ok(None);
        };

        let mut builder = LegendBuilder::from_ir(
            ir_leg.legend(),
            ir_leg.pos().prefers_vertical(),
            avail_width,
            self.fontdb().clone(),
        );

        let mut idx = 0;
        for_each_series(ir_plot, |s| {
            if let Some(entry) = s.legend_entry() {
                builder.add_entry(idx, entry)?;
                idx += 1;
            }
            Ok(())
        })?;

        Ok(builder.layout())
    }

    fn calc_estimated_bottom_heights(
        &self,
        ir_plots: &impl IrPlotsExt,
        datas: &[PlotData],
    ) -> Vec<f32> {
        let idx = GridIdx::from(ir_plots);

        let mut heights = Vec::with_capacity(idx.rows());
        for row in 0..idx.rows() {
            let is_shared_row = ir_plots.share_x() && row < idx.rows() - 1;
            let mut max_height: f32 = 0.0;
            for col in 0..idx.cols() {
                if let Some(plot_idx) = idx.plot_idx(row, col) {
                    let ir_plot = &ir_plots.plots()[plot_idx];
                    let mut height = missing_params::PLOT_PADDING.bottom();
                    if is_shared_row {
                        height += self.estimate_bottom_shared_axis_height(&ir_plot.x_axis())
                    } else {
                        height += self.estimate_bottom_axis_height(&ir_plot.x_axis())
                    };
                    if let Some(ir_leg) = ir_plot.legend() {
                        let leg = datas[plot_idx].legend.as_ref().unwrap();
                        if let ir::plot::LegendPos::OutBottom = ir_leg.pos() {
                            height += leg.size().height() + ir_leg.margin();
                        }
                    }
                    max_height = max_height.max(height);
                }
            }
            heights.push(max_height);
        }
        heights
    }

    fn calc_bottom_heights(
        &self,
        ir_plots: &impl IrPlotsExt,
        datas: &[PlotData],
        x_axes: &[Option<Axis>],
    ) -> Vec<f32> {
        let gi = GridIdx::from(ir_plots);
        let mut heights = Vec::with_capacity(gi.len());

        for row in 0..gi.rows() {
            let mut max_height = f32::NAN;
            for col in 0..gi.cols() {
                if let Some(plot_idx) = gi.plot_idx(row, col) {
                    let ir_plot = &ir_plots.plots()[plot_idx];
                    let x_axis = &x_axes[plot_idx];

                    let mut height = missing_params::PLOT_PADDING.bottom();
                    height += x_axis.as_ref().map(|a| a.size_across()).unwrap_or(0.0);
                    if let Some(ir_leg) = ir_plot.legend() {
                        let leg = datas[plot_idx].legend.as_ref().unwrap();
                        if let ir::plot::LegendPos::OutBottom = ir_leg.pos() {
                            height += leg.size().height() + ir_leg.margin();
                        }
                    }

                    max_height = max_height.max(height);
                }
            }
            debug_assert!(max_height.is_finite());
            heights.push(max_height);
        }
        heights
    }

    fn calc_top_heights(&self, ir_plots: &impl IrPlotsExt, datas: &[PlotData]) -> Vec<f32> {
        let idx = GridIdx::from(ir_plots);

        let mut heights = Vec::with_capacity(idx.rows());
        for row in 0..idx.rows() {
            let mut max_height = 0.0;
            for col in 0..idx.cols() {
                if let Some(plot_idx) = idx.plot_idx(row, col) {
                    let ir_plot = &ir_plots.plots()[plot_idx];

                    let mut height = missing_params::PLOT_PADDING.top();
                    if let Some(ir_leg) = ir_plot.legend() {
                        let leg = datas[plot_idx].legend.as_ref().unwrap();
                        if let ir::plot::LegendPos::OutTop = ir_leg.pos() {
                            height += leg.size().height() + ir_leg.margin();
                        }
                    }
                    if height > max_height {
                        max_height = height;
                    }
                }
            }
            heights.push(max_height);
        }
        heights
    }

    fn calc_left_widths(
        &self,
        ir_plots: &impl IrPlotsExt,
        datas: &[PlotData],
        y_axes: &[Option<Axis>],
    ) -> Vec<f32> {
        let gi = GridIdx::from(ir_plots);
        let mut widths = Vec::with_capacity(gi.len());

        for col in 0..gi.cols() {
            let mut max_width = f32::NAN;
            for row in 0..gi.rows() {
                if let Some(plot_idx) = gi.plot_idx(row, col) {
                    let ir_plot = &ir_plots.plots()[plot_idx];
                    let y_axis = &y_axes[plot_idx];

                    let mut width = missing_params::PLOT_PADDING.left();
                    width += y_axis.as_ref().map(|a| a.size_across()).unwrap_or(0.0);
                    if let Some(ir_leg) = ir_plot.legend() {
                        let leg = datas[plot_idx].legend.as_ref().unwrap();
                        if let ir::plot::LegendPos::OutLeft = ir_leg.pos() {
                            width += leg.size().width() + ir_leg.margin();
                        }
                    }

                    max_width = max_width.max(width);
                }
            }
            debug_assert!(max_width.is_finite());
            widths.push(max_width);
        }
        widths
    }

    fn calc_right_widths(&self, ir_plots: &impl IrPlotsExt, datas: &[PlotData]) -> Vec<f32> {
        let gi = GridIdx::from(ir_plots);
        let mut widths = Vec::with_capacity(gi.len());

        for col in 0..gi.cols() {
            let mut max_width: f32 = 0.0;
            for row in 0..gi.rows() {
                if let Some(plot_idx) = gi.plot_idx(row, col) {
                    let ir_plot = &ir_plots.plots()[plot_idx];
                    let mut width = missing_params::PLOT_PADDING.right();
                    if let Some(ir_leg) = ir_plot.legend() {
                        let leg = datas[plot_idx].legend.as_ref().unwrap();
                        if let ir::plot::LegendPos::OutRight = ir_leg.pos() {
                            width += leg.size().width() + ir_leg.margin();
                        }
                    }
                    max_width = max_width.max(width);
                }
            }
            debug_assert!(max_width.is_finite());
            widths.push(max_width);
        }
        widths
    }

    fn setup_y_axes(
        &self,
        ir_plots: &impl IrPlotsExt,
        datas: &[PlotData],
        height: f32,
    ) -> Result<Vec<Option<Axis>>, Error> {
        if ir_plots.share_y() {
            return self.setup_shared_y_axes(ir_plots, datas, height);
        }

        let mut axes = Vec::with_capacity(ir_plots.len());
        for (ir_plot, plot) in ir_plots.plots().iter().zip(datas.iter()) {
            let ir_axis = ir_plot.y_axis();
            let bounds = Series::unite_y_bounds(&plot.series, None)?;
            let axis = bounds
                .map(|bounds| {
                    self.setup_axis(
                        ir_axis,
                        &bounds,
                        axis::Side::Left,
                        height,
                        &plot.insets,
                        None,
                    )
                })
                .transpose()?;
            axes.push(axis);
        }
        Ok(axes)
    }

    fn setup_shared_y_axes(
        &self,
        ir_plots: &impl IrPlotsExt,
        datas: &[PlotData],
        height: f32,
    ) -> Result<Vec<Option<Axis>>, Error> {
        let idx = GridIdx::from(ir_plots);

        let mut axes = vec![None; idx.len()];
        for row in 0..idx.rows() {
            // uniting bounds for the whole row
            let mut bounds = None;
            for plt_idx in idx.iter_plot_indices_within_row(row) {
                let series = &datas[plt_idx].series;
                bounds = Series::unite_y_bounds(series, bounds)?;
            }
            let bounds = bounds.unwrap();

            // building the first col (leftmost) axis of this row
            // and get its scale to share with the other cols
            let Some(plt_idx) = idx.plot_idx(row, 0) else {
                continue;
            };
            let ir_plot = &ir_plots.plots()[plt_idx];
            let ir_axis = ir_plot.y_axis();
            let axis = self.setup_axis(
                ir_axis,
                &bounds,
                axis::Side::Left,
                height,
                &datas[0].insets,
                None,
            )?;
            let shared_scale = axis.scale().clone();
            axes[plt_idx] = Some(axis);

            // building the other cols with shared scale
            for col in 1..idx.cols() {
                if let Some(plot_idx) = idx.plot_idx(row, col) {
                    let ir_plot = &ir_plots.plots()[plot_idx];
                    let plot_data = &datas[plot_idx];
                    let axis = self.setup_axis(
                        ir_plot.y_axis(),
                        &bounds,
                        axis::Side::Left,
                        height,
                        &plot_data.insets,
                        Some(shared_scale.clone()),
                    )?;
                    axes[plt_idx] = Some(axis);
                }
            }
        }

        Ok(axes)
    }

    fn setup_x_axes(
        &self,
        ir_plots: &impl IrPlotsExt,
        datas: &[PlotData],
        width: f32,
    ) -> Result<Vec<Option<Axis>>, Error> {
        if ir_plots.share_x() {
            return self.setup_shared_x_axes(ir_plots, datas, width);
        }

        let mut axes = Vec::with_capacity(ir_plots.len());
        for (ir_plot, plot) in ir_plots.plots().iter().zip(datas.iter()) {
            let ir_axis = ir_plot.x_axis();
            let bounds = Series::unite_x_bounds(&plot.series, None)?;
            let axis = bounds
                .map(|bounds| {
                    self.setup_axis(
                        ir_axis,
                        &bounds,
                        axis::Side::Bottom,
                        width,
                        &plot.insets,
                        None,
                    )
                })
                .transpose()?;
            axes.push(axis);
        }
        Ok(axes)
    }

    fn setup_shared_x_axes(
        &self,
        ir_plots: &impl IrPlotsExt,
        datas: &[PlotData],
        width: f32,
    ) -> Result<Vec<Option<Axis>>, Error> {
        let idx = GridIdx::from(ir_plots);

        let mut axes = vec![None; idx.len()];
        for col in 0..idx.cols() {
            // uniting bounds for the whole column
            let mut bounds = None;
            for plt_idx in idx.iter_plot_indices_within_col(col) {
                let series = &datas[plt_idx].series;
                bounds = Series::unite_x_bounds(series, bounds)?;
            }
            let bounds = bounds;

            // building the last row (bottommost) axis of this column
            // and get its scale to share with the other rows
            let mut shared_scale: Option<Arc<AxisScale>> = None;
            for row in (0..idx.rows()).rev() {
                let Some(plt_idx) = idx.plot_idx(row, 0) else {
                    continue;
                };
                let Some(bounds) = &bounds else {
                    axes[plt_idx] = None;
                    continue;
                };
                let ir_plot = &ir_plots.plots()[plt_idx];
                let plot_data = &datas[plt_idx];
                let axis = self.setup_axis(
                    ir_plot.x_axis(),
                    bounds,
                    axis::Side::Bottom,
                    width,
                    &plot_data.insets,
                    shared_scale.as_ref().cloned(),
                )?;
                if shared_scale.is_none() {
                    shared_scale = Some(axis.scale().clone());
                }
                axes[plt_idx] = Some(axis);
            }
        }

        Ok(axes)
    }
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

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    pub fn draw_plots<D>(
        &mut self,
        ctx: &Ctx<D>,
        ir_plots: &impl IrPlotsExt,
        plots: &Plots,
    ) -> Result<(), Error>
    where
        D: data::Source,
    {
        for (ir_plot, plot) in ir_plots.plots().iter().zip(plots.plots.iter()) {
            self.draw_plot(ctx, ir_plot, plot)?;
        }
        Ok(())
    }

    fn draw_plot<D>(&mut self, ctx: &Ctx<D>, ir_plot: &ir::Plot, plot: &Plot) -> Result<(), Error>
    where
        D: data::Source,
    {
        self.draw_plot_background(ctx, ir_plot, &plot.plot_rect)?;
        let Some(axes) = &plot.axes else {
            self.draw_plot_border(ctx, ir_plot.border(), &plot.plot_rect)?;
            return Ok(());
        };

        self.draw_plot_axes_grids(ctx, &axes, &plot.plot_rect)?;
        self.draw_plot_series(ctx, ir_plot.series(), &plot.series, &plot.plot_rect, &axes)?;
        self.draw_plot_axes(ctx, &axes, &plot.plot_rect)?;
        self.draw_plot_border(ctx, ir_plot.border(), &plot.plot_rect)?;

        if let (Some(leg), Some(ir_leg)) = (&plot.legend, ir_plot.legend()) {
            self.draw_plot_legend(ctx, &leg, ir_leg, &plot.plot_rect, &plot.outer_rect)?;
        }

        Ok(())
    }

    fn draw_plot_background<D>(
        &mut self,
        ctx: &Ctx<D>,
        ir_plot: &ir::Plot,
        rect: &geom::Rect,
    ) -> Result<(), render::Error> {
        if let Some(fill) = ir_plot.fill() {
            self.draw_rect(&render::Rect {
                rect: *rect,
                fill: Some(fill.as_paint(ctx.theme())),
                stroke: None,
                transform: None,
            })?;
        }
        Ok(())
    }

    fn draw_plot_border<D>(
        &mut self,
        ctx: &Ctx<D>,
        border: Option<&ir::plot::Border>,
        rect: &geom::Rect,
    ) -> Result<(), Error> {
        match border {
            None => Ok(()),
            Some(ir::plot::Border::Box(stroke)) => {
                self.draw_rect(&render::Rect {
                    rect: *rect,
                    fill: None,
                    stroke: Some(stroke.as_stroke(ctx.theme())),
                    transform: None,
                })?;
                Ok(())
            }
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
                self.draw_path(&path)?;
                Ok(())
            }
            Some(ir::plot::Border::AxisArrow { .. }) => {
                todo!("Draw axis arrow")
            }
        }
    }

    fn draw_plot_series<D>(
        &mut self,
        ctx: &Ctx<D>,
        ir_series: &[ir::Series],
        series: &[Series],
        rect: &geom::Rect,
        axes: &Axes,
    ) -> Result<(), Error>
    where
        D: data::Source,
    {
        self.push_clip(&render::Clip {
            path: &rect.to_path(),
            transform: None,
        })?;

        let cm = scale::CoordMapXy {
            x: axes.bottom.coord_map(),
            y: axes.left.coord_map(),
        };

        for (ir_series, series) in ir_series.iter().zip(series.iter()) {
            self.draw_series_plot(ctx, ir_series, series, rect, &cm)?;
        }
        self.pop_clip()?;
        Ok(())
    }

    fn draw_plot_axes_grids<D>(
        &mut self,
        ctx: &Ctx<D>,
        axes: &Axes,
        rect: &geom::Rect,
    ) -> Result<(), Error> {
        self.draw_axis_minor_grids(ctx, &axes.bottom, rect)?;
        self.draw_axis_minor_grids(ctx, &axes.left, rect)?;
        self.draw_axis_major_grids(ctx, &axes.bottom, rect)?;
        self.draw_axis_major_grids(ctx, &axes.left, rect)?;
        Ok(())
    }

    fn draw_plot_axes<D>(
        &mut self,
        ctx: &Ctx<D>,
        axes: &Axes,
        plot_rect: &geom::Rect,
    ) -> Result<(), Error> {
        self.draw_axis(ctx, &axes.bottom, plot_rect)?;
        self.draw_axis(ctx, &axes.left, plot_rect)?;
        Ok(())
    }

    fn draw_plot_legend<D>(
        &mut self,
        ctx: &Ctx<D>,
        leg: &Legend,
        ir_leg: &ir::PlotLegend,
        plot_rect: &geom::Rect,
        outer_rect: &geom::Rect,
    ) -> Result<(), Error> {
        let top_left = legend_top_left(ir_leg, leg.size(), plot_rect, outer_rect);
        self.draw_legend(ctx, &leg, &top_left)?;
        Ok(())
    }
}

fn legend_top_left(
    legend: &ir::PlotLegend,
    sz: geom::Size,
    plot_rect: &geom::Rect,
    outer_rect: &geom::Rect,
) -> geom::Point {
    match legend.pos() {
        ir::plot::LegendPos::OutTop => {
            geom::Point::new(outer_rect.center_x() - sz.width() / 2.0, outer_rect.top())
        }
        ir::plot::LegendPos::OutRight => geom::Point::new(
            outer_rect.right() - sz.width(),
            outer_rect.center_y() - sz.height() / 2.0,
        ),
        ir::plot::LegendPos::OutBottom => geom::Point::new(
            outer_rect.center_x() - sz.width() / 2.0,
            outer_rect.bottom() - sz.height(),
        ),
        ir::plot::LegendPos::OutLeft => {
            geom::Point::new(outer_rect.left(), outer_rect.center_y() - sz.height() / 2.0)
        }

        ir::plot::LegendPos::InTop => geom::Point::new(
            plot_rect.center_x() - sz.width() / 2.0,
            plot_rect.top() + legend.margin(),
        ),
        ir::plot::LegendPos::InTopRight => geom::Point::new(
            plot_rect.right() - sz.width() - legend.margin(),
            plot_rect.top() + legend.margin(),
        ),
        ir::plot::LegendPos::InRight => geom::Point::new(
            plot_rect.right() - sz.width() - legend.margin(),
            plot_rect.center_y() - sz.height() / 2.0,
        ),
        ir::plot::LegendPos::InBottomRight => geom::Point::new(
            plot_rect.right() - sz.width() - legend.margin(),
            plot_rect.bottom() - sz.height() - legend.margin(),
        ),
        ir::plot::LegendPos::InBottom => geom::Point::new(
            plot_rect.center_x() - sz.width() / 2.0,
            plot_rect.bottom() - sz.height() - legend.margin(),
        ),
        ir::plot::LegendPos::InBottomLeft => geom::Point::new(
            plot_rect.left() + legend.margin(),
            plot_rect.bottom() - sz.height() - legend.margin(),
        ),
        ir::plot::LegendPos::InLeft => geom::Point::new(
            plot_rect.left() + legend.margin(),
            plot_rect.center_y() - sz.height() / 2.0,
        ),
        ir::plot::LegendPos::InTopLeft => geom::Point::new(
            plot_rect.left() + legend.margin(),
            plot_rect.top() + legend.margin(),
        ),
    }
}
