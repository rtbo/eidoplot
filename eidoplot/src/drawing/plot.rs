use std::sync::Arc;

use crate::drawing::axis::{Axis, AxisScale, Bounds, Side};
use crate::drawing::legend::{Legend, LegendBuilder};
use crate::drawing::scale::CoordMapXy;
use crate::drawing::series::{self, Series, SeriesExt};
use crate::drawing::{Ctx, Error, SurfWrapper};
use crate::render::{self, Surface};
use crate::style::defaults;
use crate::{data, geom, ir, missing_params};

#[derive(Debug, Clone)]
pub struct Plots {
    plots: Vec<Option<Plot>>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    X,
    Y,
}

#[derive(Debug, Clone)]
struct Axes {
    x: Vec<Axis>,
    y: Vec<Axis>,
}

impl Axes {
    fn or_find(
        &self,
        or: Orientation,
        ax_ref: Option<&ir::axis::Ref>,
    ) -> Result<Option<&Axis>, Error> {
        let axes = match or {
            Orientation::X => &self.x,
            Orientation::Y => &self.y,
        };

        for (ax_idx, a) in axes.iter().enumerate() {
            match ax_ref {
                None => {
                    if ax_idx == 0 {
                        return Ok(Some(a));
                    }
                }
                Some(ir::axis::Ref::Idx(idx)) => {
                    if ax_idx == *idx {
                        return Ok(Some(a));
                    }
                }
                Some(ir::axis::Ref::Id(id)) => {
                    if a.id() == Some(id.as_str()) {
                        return Ok(Some(a));
                    }
                }
                Some(ir::axis::Ref::Title(title)) => {
                    if a.title_text() == Some(title.as_str()) {
                        return Ok(Some(a));
                    }
                }
                Some(ax_ref) => return Err(Error::IllegalAxisRef(ax_ref.clone())),
            }
        }
        Ok(None)
    }
}

/// Plot itermediate data during setup phase
#[derive(Debug, Clone)]
struct PlotData {
    series: Vec<Series>,
    legend: Option<Legend>,
    insets: geom::Padding,
}

trait IrPlotExt {
    fn x_axes(&self) -> &[ir::Axis];
    fn y_axes(&self) -> &[ir::Axis];
    fn or_axes(&self, or: Orientation) -> &[ir::Axis] {
        match or {
            Orientation::X => self.x_axes(),
            Orientation::Y => self.y_axes(),
        }
    }
}

impl IrPlotExt for ir::Plot {
    fn x_axes(&self) -> &[ir::Axis] {
        self.x_axes()
    }
    fn y_axes(&self) -> &[ir::Axis] {
        self.y_axes()
    }
}

trait IrPlotsExt {
    fn cols(&self) -> u32;
    fn plot(&self, row: u32, col: u32) -> Option<&ir::Plot>;
    fn plots(&self) -> impl Iterator<Item = Option<&ir::Plot>> + '_;

    /// get the plot and its index at once
    fn idx_plt(&self, row: u32, col: u32) -> Option<(usize, &ir::Plot)> {
        self.plot(row, col)
            .map(|p| ((row * self.cols() + col) as usize, p))
    }

    fn or_axes_len(&self, or: Orientation) -> usize {
        self.plots()
            .filter_map(|p| p)
            .map(|p| p.or_axes(or).len())
            .sum()
    }

    fn or_find_axis(
        &self,
        or: Orientation,
        ax_ref: &ir::axis::Ref,
        plt_idx: usize,
    ) -> Option<(usize, &ir::Axis)> {
        let mut fig_ax_idx = 0;
        for (pi, plot) in self.plots().enumerate() {
            let Some(plot) = plot else { continue };

            for (ai, axis) in plot.or_axes(or).iter().enumerate() {
                match ax_ref {
                    ir::axis::Ref::Idx(idx) => {
                        if *idx == ai && plt_idx == pi {
                            return Some((fig_ax_idx, axis));
                        }
                    }
                    ir::axis::Ref::FigIdx(idx) => {
                        if *idx == fig_ax_idx {
                            return Some((fig_ax_idx, axis));
                        }
                    }
                    ir::axis::Ref::Id(id) => {
                        if axis.id() == Some(id) {
                            return Some((fig_ax_idx, axis));
                        }
                    }
                    ir::axis::Ref::Title(id) => {
                        if axis.title().map(|t| t.text()) == Some(id) {
                            return Some((fig_ax_idx, axis));
                        }
                    }
                }
                fig_ax_idx += 1;
            }
        }
        None
    }
}

impl IrPlotsExt for ir::figure::Plots {
    fn cols(&self) -> u32 {
        self.cols()
    }
    fn plot(&self, row: u32, col: u32) -> Option<&ir::Plot> {
        self.plot(row, col)
    }
    fn plots(&self) -> impl Iterator<Item = Option<&ir::Plot>> + '_ {
        self.iter()
    }
}

/// Temporary struct to hold axes of a plot during setup.
/// Either X or Y axes are held, but not both.
/// None is held when there is no series matching the axis
#[derive(Debug, Clone)]
struct PlotAxes(Vec<Option<Axis>>);

impl PlotAxes {
    fn size_across(&self, side: ir::axis::Side) -> f32 {
        let mut sz = 0.0;
        let mut cnt = 0;
        for a in &self.0 {
            if let Some(a) = a {
                if a.side().to_ir_side() != side {
                    continue;
                }
                sz += a.size_across();
                cnt += 1;
            }
        }
        if cnt > 1 {
            sz += (cnt as f32 - 1.0)
                * (missing_params::AXIS_MARGIN + missing_params::AXIS_LINE_WIDTH);
        }
        sz
    }
}

impl<D> Ctx<'_, D>
where
    D: data::Source,
{
    /// Setup a collection of plots, given an IR representation of the plots
    /// and a bounding rectangle.
    pub fn setup_plots(
        &self,
        ir_plots: &ir::figure::Plots,
        rect: &geom::Rect,
    ) -> Result<Plots, Error> {
        // We build all needed characteristics by the plots one after another.
        // Each characteristic (axes, interspace etc.) is in vector, indexed in the
        // same order than the plots

        // PlotData contains all data that is not impacted by the size of axes
        let plot_data = self.setup_plot_data(ir_plots, rect)?;

        // Estimate the space taken by all horizontal axes
        // Can be slightly wrong if font metrics height isn't exactly font size.
        // This will be fixed at end of the setup phase.
        let bottom_heights =
            self.calc_estimated_x_heights(ir_plots, &plot_data, ir::axis::Side::Main);
        let top_heights =
            self.calc_estimated_x_heights(ir_plots, &plot_data, ir::axis::Side::Opposite);
        let hor_space_height = bottom_heights.iter().sum::<f32>()
            + top_heights.iter().sum::<f32>()
            + ir_plots.space() * (ir_plots.rows() - 1) as f32;

        // Now we can determine length of vertical axes and set them all up
        let subplot_rect_height = (rect.height() - hor_space_height) / ir_plots.rows() as f32;
        let y_axes =
            self.setup_orientation_axes(Orientation::Y, ir_plots, &plot_data, subplot_rect_height)?;

        // Now we calculate the interspace between vertical axes
        let left_widths = self.calc_y_widths(ir_plots, &plot_data, &y_axes, ir::axis::Side::Main);
        let right_widths =
            self.calc_y_widths(ir_plots, &plot_data, &y_axes, ir::axis::Side::Opposite);
        let vert_space_width = left_widths.iter().sum::<f32>()
            + right_widths.iter().sum::<f32>()
            + ir_plots.space() * (ir_plots.cols() - 1) as f32;

        // Now we can determine width of horizontal axes and set them all up
        let subplot_rect_width = (rect.width() - vert_space_width) / ir_plots.cols() as f32;
        let x_axes =
            self.setup_orientation_axes(Orientation::X, ir_plots, &plot_data, subplot_rect_width)?;

        // bottom heights were estimated, we can now calculate them accurately and rebuild the y-axes
        let bottom_heights =
            self.calc_x_heights(ir_plots, &plot_data, &x_axes, ir::axis::Side::Main);
        let top_heights =
            self.calc_x_heights(ir_plots, &plot_data, &x_axes, ir::axis::Side::Opposite);
        let hor_space_height = bottom_heights.iter().sum::<f32>()
            + top_heights.iter().sum::<f32>()
            + ir_plots.space() * (ir_plots.rows() - 1) as f32;
        let subplot_rect_height = (rect.height() - hor_space_height) / ir_plots.rows() as f32;
        let y_axes =
            self.setup_orientation_axes(Orientation::Y, ir_plots, &plot_data, subplot_rect_height)?;

        // Everything is now ready to setup all plots
        let mut plots: Vec<Option<Plot>> = vec![None; ir_plots.len()];
        let mut plot_data = plot_data.into_iter();
        let mut x_axes = x_axes.into_iter();
        let mut y_axes = y_axes.into_iter();

        let mut y = rect.y();
        for row in 0..ir_plots.rows() {
            let height =
                subplot_rect_height + top_heights[row as usize] + bottom_heights[row as usize];
            let mut x = rect.x();

            for col in 0..ir_plots.cols() {
                let width =
                    subplot_rect_width + left_widths[col as usize] + right_widths[col as usize];

                let data = plot_data.next().unwrap();
                let x_axes = x_axes.next().unwrap();
                let y_axes = y_axes.next().unwrap();

                if ir_plots.plot(row, col).is_some() {
                    let outer_rect = geom::Rect::from_xywh(x, y, width, height);
                    let plot_rect = geom::Rect::from_xywh(
                        x + left_widths[col as usize],
                        y + top_heights[col as usize],
                        subplot_rect_width,
                        subplot_rect_height,
                    );

                    let PlotData { series, legend, .. } = data.unwrap();

                    let axes = {
                        let x_ax = x_axes.unwrap();
                        let y_ax = y_axes.unwrap();
                        let x: Vec<Axis> = x_ax.0.into_iter().filter_map(|a| a).collect();
                        let y: Vec<Axis> = y_ax.0.into_iter().filter_map(|a| a).collect();

                        if x.is_empty() && y.is_empty() {
                            None
                        } else if x.is_empty() || y.is_empty() {
                            unreachable!(
                                "axis are None when there is no series, so should be both None or both Some"
                            )
                        } else {
                            Some(Axes { x, y })
                        }
                    };

                    let plt_idx = row * ir_plots.cols() + col;
                    plots[plt_idx as usize] = Some(Plot {
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

        Ok(Plots { plots })
    }

    fn setup_plot_data(
        &self,
        ir_plots: &ir::figure::Plots,
        rect: &geom::Rect,
    ) -> Result<Vec<Option<PlotData>>, Error> {
        let mut plot_data = vec![None; ir_plots.len()];
        for (idx, ir_plot) in ir_plots.iter().enumerate() {
            let Some(ir_plot) = ir_plot else { continue };
            let series = self.setup_plot_series(ir_plot)?;
            let cols = ir_plots.cols() as f32;
            let avail_width = (rect.width() - ir_plots.space() * (cols - 1.0)) / cols;
            let legend = self.setup_plot_legend(ir_plot, avail_width)?;
            let insets = plot_insets(ir_plot);
            plot_data[idx] = Some(PlotData {
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

    fn calc_estimated_x_heights(
        &self,
        ir_plots: &ir::figure::Plots,
        datas: &[Option<PlotData>],
        side: ir::axis::Side,
    ) -> Vec<f32> {
        let mut heights = Vec::with_capacity(ir_plots.rows() as usize);
        for row in 0..ir_plots.rows() {
            let mut max_height: f32 = 0.0;
            for col in 0..ir_plots.cols() {
                if let Some((plt_idx, ir_plot)) = ir_plots.idx_plt(row, col) {
                    let data = datas[plt_idx].as_ref().unwrap();

                    let mut height = x_plot_padding(side);
                    height += self.estimate_x_axes_height(ir_plot.x_axes(), side);
                    if let Some(ir_leg) = ir_plot.legend() {
                        let leg = data.legend.as_ref().unwrap();
                        if x_side_matches_out_legend_pos(side, ir_leg.pos()) {
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

    fn calc_x_heights(
        &self,
        ir_plots: &ir::figure::Plots,
        datas: &[Option<PlotData>],
        x_axes: &[Option<PlotAxes>],
        side: ir::axis::Side,
    ) -> Vec<f32> {
        let mut heights = Vec::with_capacity(ir_plots.rows() as usize);

        for row in 0..ir_plots.rows() {
            let mut max_height = f32::NAN;
            for col in 0..ir_plots.cols() {
                let Some((index, ir_plot)) = ir_plots.idx_plt(row, col) else {
                    continue;
                };

                let data = datas[index].as_ref().unwrap();
                let x_axes = x_axes[index].as_ref().unwrap();

                let mut height = x_plot_padding(side);
                height += x_axes.size_across(side);

                if let Some(ir_leg) = ir_plot.legend() {
                    let leg = data.legend.as_ref().unwrap();
                    if x_side_matches_out_legend_pos(side, ir_leg.pos()) {
                        height += leg.size().height() + ir_leg.margin();
                    }
                }

                max_height = max_height.max(height);
            }
            debug_assert!(max_height.is_finite());
            heights.push(max_height);
        }
        heights
    }

    fn calc_y_widths(
        &self,
        ir_plots: &ir::figure::Plots,
        datas: &[Option<PlotData>],
        y_axes: &[Option<PlotAxes>],
        side: ir::axis::Side,
    ) -> Vec<f32> {
        let mut widths = Vec::with_capacity(ir_plots.cols() as usize);

        for col in 0..ir_plots.cols() {
            let mut max_width = f32::NAN;
            for row in 0..ir_plots.rows() {
                if let Some((index, ir_plot)) = ir_plots.idx_plt(row, col) {
                    let data = datas[index].as_ref().unwrap();
                    let y_axis = y_axes[index].as_ref().unwrap();

                    let mut width = y_plot_padding(side);
                    width += y_axis.size_across(side);

                    if let Some(ir_leg) = ir_plot.legend() {
                        let leg = data.legend.as_ref().unwrap();
                        if y_side_matches_out_legend_pos(side, ir_leg.pos()) {
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

    fn setup_orientation_axes(
        &self,
        or: Orientation,
        ir_plots: &ir::figure::Plots,
        datas: &[Option<PlotData>],
        size_along: f32,
    ) -> Result<Vec<Option<PlotAxes>>, Error> {
        let mut plot_axes = vec![None; ir_plots.len()];

        // collecting all axes that own their scale.

        // ax_infos is Some only for the axis owning their scale
        let mut ax_infos: Vec<Option<(Bounds, Arc<AxisScale>)>> =
            vec![None; ir_plots.or_axes_len(or)];

        // index of the first axis of a plot, at figure level
        let mut fig_ax_idx0 = 0;

        for (plt_idx, ir_plot) in ir_plots.iter().enumerate() {
            let Some(ir_plot) = ir_plot else { continue };

            let ir_axes = ir_plot.or_axes(or);
            let mut axes = vec![None; ir_axes.len()];

            for (ax_idx, axis) in ir_axes.iter().enumerate() {
                if axis.scale().is_shared() {
                    continue;
                }

                // `axis` owns its scale.
                // We have to collect data bounds of all the series that refer to it.
                // `matcher` will match the series that refer to `axis` with Series::x/y_axis.
                // If Series::x/y_axis returns None, it refers implicitly to ax_idx == 0.

                // We also have to collect data bounds of series that refer to a shared axis
                // referring explicitly to `axis`. This is done in the inner loop with `axis2`.

                let matcher = series::AxisMatcher {
                    plt_idx,
                    ax_idx,
                    id: axis.id(),
                    title: axis.title().map(|t| t.text()),
                };
                let mut bounds = None;

                for (plt_idx2, ir_plot2) in ir_plots.iter().enumerate() {
                    let Some(ir_plot2) = ir_plot2 else { continue };
                    let data = datas[plt_idx2].as_ref().unwrap();
                    let series = &data.series;
                    bounds = Series::unite_bounds(or, series, bounds, &matcher, plt_idx2)?;

                    for (ax_idx2, axis2) in ir_plot2.or_axes(or).iter().enumerate() {
                        if let ir::axis::Scale::Shared(ax_ref2) = axis2.scale() {
                            if matcher.matches_ref(Some(ax_ref2), plt_idx2)? {
                                let matcher = series::AxisMatcher {
                                    plt_idx: plt_idx2,
                                    ax_idx: ax_idx2,
                                    id: axis2.id(),
                                    title: axis2.title().map(|t| t.text()),
                                };
                                bounds =
                                    Series::unite_bounds(or, series, bounds, &matcher, plt_idx2)?;
                            }
                        }
                    }
                }

                let Some(bounds) = bounds else { continue };

                let ax = self.setup_axis(
                    axis,
                    &bounds,
                    Side::from_or_ir_side(or, axis.side()),
                    size_along,
                    &datas[plt_idx].as_ref().unwrap().insets,
                    None,
                )?;
                ax_infos[fig_ax_idx0 + ax_idx] = Some((bounds, ax.scale().clone()));
                axes[ax_idx] = Some(ax);
            }

            fig_ax_idx0 += ir_axes.len();
            plot_axes[plt_idx] = Some(PlotAxes(axes));
        }

        // build the others with shared scale

        for (plt_idx, ir_plot) in ir_plots.iter().enumerate() {
            let Some(ir_plot) = ir_plot else {
                continue;
            };

            let ir_axes = ir_plot.or_axes(or);
            let axes = plot_axes[plt_idx].as_mut().unwrap();

            for (ax_idx, ir_axis) in ir_axes.iter().enumerate() {
                let ir::axis::Scale::Shared(ax_ref) = ir_axis.scale() else {
                    continue;
                };
                let (fig_ax_idx, _) = ir_plots
                    .or_find_axis(or, ax_ref, plt_idx)
                    .ok_or_else(|| Error::UnknownAxisRef(ax_ref.clone()))?;

                let info = ax_infos[fig_ax_idx]
                    .as_ref()
                    .ok_or_else(|| Error::IllegalAxisRef(ax_ref.clone()))?;

                let axis = self.setup_axis(
                    ir_axis,
                    &info.0,
                    Side::from_or_ir_side(or, ir_axis.side()),
                    size_along,
                    &datas[plt_idx].as_ref().unwrap().insets,
                    Some(info.1.clone()),
                )?;
                axes.0[ax_idx] = Some(axis);
            }
        }
        Ok(plot_axes)
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

fn x_plot_padding(side: ir::axis::Side) -> f32 {
    match side {
        ir::axis::Side::Main => missing_params::PLOT_PADDING.bottom(),
        ir::axis::Side::Opposite => missing_params::PLOT_PADDING.top(),
    }
}

fn y_plot_padding(side: ir::axis::Side) -> f32 {
    match side {
        ir::axis::Side::Main => missing_params::PLOT_PADDING.left(),
        ir::axis::Side::Opposite => missing_params::PLOT_PADDING.right(),
    }
}

fn x_side_matches_out_legend_pos(side: ir::axis::Side, legend_pos: ir::plot::LegendPos) -> bool {
    match (side, legend_pos) {
        (ir::axis::Side::Main, ir::plot::LegendPos::OutBottom) => true,
        (ir::axis::Side::Opposite, ir::plot::LegendPos::OutTop) => true,
        _ => false,
    }
}

fn y_side_matches_out_legend_pos(side: ir::axis::Side, legend_pos: ir::plot::LegendPos) -> bool {
    match (side, legend_pos) {
        (ir::axis::Side::Main, ir::plot::LegendPos::OutLeft) => true,
        (ir::axis::Side::Opposite, ir::plot::LegendPos::OutRight) => true,
        _ => false,
    }
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    pub fn draw_plots<D>(
        &mut self,
        ctx: &Ctx<D>,
        ir_plots: &ir::figure::Plots,
        plots: &Plots,
    ) -> Result<(), Error>
    where
        D: data::Source,
    {
        for (ir_plot, plot) in ir_plots.plots().zip(plots.plots.iter()) {
            if let (Some(ir_plot), Some(plot)) = (ir_plot, plot.as_ref()) {
                self.draw_plot(ctx, ir_plot, plot)?;
            }
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

        for (ir_series, series) in ir_series.iter().zip(series.iter()) {
            let (x_ax_ref, y_ax_ref) = ir_series.axes();
            let x = axes.or_find(Orientation::X, x_ax_ref)?;
            let y = axes.or_find(Orientation::Y, y_ax_ref)?;
            let (Some(x), Some(y)) = (x, y) else {
                unreachable!("Series without axis");
            };
            let cm = CoordMapXy {
                x: x.coord_map(),
                y: y.coord_map(),
            };
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
        for axis in axes.x.iter() {
            self.draw_axis_minor_grids(ctx, axis, rect)?;
        }
        for axis in axes.y.iter() {
            self.draw_axis_minor_grids(ctx, axis, rect)?;
        }
        for axis in axes.x.iter() {
            self.draw_axis_major_grids(ctx, axis, rect)?;
        }
        for axis in axes.y.iter() {
            self.draw_axis_major_grids(ctx, axis, rect)?;
        }
        Ok(())
    }

    fn draw_plot_axes<D>(
        &mut self,
        ctx: &Ctx<D>,
        axes: &Axes,
        plot_rect: &geom::Rect,
    ) -> Result<(), Error> {
        self.draw_plot_axes_side(ctx, &axes.x, Side::Top, plot_rect)?;
        self.draw_plot_axes_side(ctx, &axes.y, Side::Right, plot_rect)?;
        self.draw_plot_axes_side(ctx, &axes.x, Side::Bottom, plot_rect)?;
        self.draw_plot_axes_side(ctx, &axes.y, Side::Left, plot_rect)?;
        Ok(())
    }

    fn draw_plot_axes_side<D>(
        &mut self,
        ctx: &Ctx<D>,
        axes: &[Axis],
        side: Side,
        plot_rect: &geom::Rect,
    ) -> Result<(), Error> {
        let mut rect = *plot_rect;
        for axis in axes.iter() {
            if axis.side() == side {
                let shift = self.draw_axis(ctx, axis, &rect)?;
                rect = match side {
                    Side::Top => rect.shifted_top_side(-shift),
                    Side::Right => rect.shifted_right_side(shift),
                    Side::Bottom => rect.shifted_bottom_side(shift),
                    Side::Left => rect.shifted_bottom_side(-shift),
                };
            }
        }
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
