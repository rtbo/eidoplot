use std::sync::Arc;

use crate::drawing::axis::{Axis, AxisScale, Bounds, Side};
use crate::drawing::legend::{Legend, LegendBuilder};
use crate::drawing::scale::CoordMapXy;
use crate::drawing::series::{self, Series, SeriesExt};
use crate::drawing::{Ctx, Error};
use crate::ir::plot::PlotLine;
use crate::style::defaults;
use crate::style::series::Palette;
use crate::style::theme::{self, Theme};
use crate::{Style, data, geom, ir, missing_params, render};

#[derive(Debug, Clone)]
pub struct Plots {
    plots: Vec<Option<Plot>>,
}

#[derive(Debug, Clone)]
struct Plot {
    rect: geom::Rect,
    fill: Option<theme::Fill>,
    border: Option<ir::plot::Border>,
    // None when there is no series (empty plot)
    axes: Option<Axes>,
    series: Vec<Series>,
    legend: Option<(geom::Point, Legend)>,
    lines: Vec<PlotLine>,
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
                    if a.id() == Some(id.as_str()) || a.title_text() == Some(id.as_str()) {
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
                        if axis.id() == Some(id) || axis.title().map(|t| t.text()) == Some(id) {
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
                * (missing_params::AXIS_MARGIN + missing_params::AXIS_SPINE_WIDTH);
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

                if let Some(ir_plot) = ir_plots.plot(row, col) {
                    let outer_rect = geom::Rect::from_xywh(x, y, width, height);
                    let plot_rect = geom::Rect::from_xywh(
                        x + left_widths[col as usize],
                        y + top_heights[row as usize],
                        subplot_rect_width,
                        subplot_rect_height,
                    );

                    let PlotData { series, legend, .. } = data.unwrap();

                    let legend = legend.map(|leg| {
                        let top_left = legend_top_left(
                            ir_plot.legend().unwrap(),
                            leg.size(),
                            &plot_rect,
                            &outer_rect,
                        );
                        (top_left, leg)
                    });

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
                    let plot = Plot {
                        rect: plot_rect,
                        fill: ir_plot.fill().cloned(),
                        border: ir_plot.border().cloned(),
                        axes,
                        series,
                        legend,
                        lines: ir_plot.lines().to_vec(),
                    };
                    plots[plt_idx as usize] = Some(plot);
                }
                x += width + ir_plots.space();
            }

            y += height + ir_plots.space();
        }

        let mut plots = Plots { plots };

        plots.update_series_data(self.data_source())?;

        Ok(plots)
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
            .map(|(index, s)| Series::prepare(index, s, self.data_source()))
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
            self.fontdb(),
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
                    if let (Some(ir_leg), Some(leg)) = (ir_plot.legend(), data.legend.as_ref()) {
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

                if let (Some(ir_leg), Some(leg)) = (ir_plot.legend(), data.legend.as_ref()) {
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

                    if let (Some(ir_leg), Some(leg)) = (ir_plot.legend(), data.legend.as_ref()) {
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

            // track whether the main and opposite axes are directly attached to the plot area
            let mut main_off_plot = false;
            let mut opposite_off_plot = false;

            for (ax_idx, ir_ax) in ir_axes.iter().enumerate() {
                if ir_ax.scale().is_shared() {
                    continue;
                }

                // `ir_ax` owns its scale.
                // We have to collect data bounds of all the series that refer to it.
                // `matcher` will match the series that refer to `ir_ax` with Series::x/y_axis.
                // If Series::x/y_axis returns None, it refers implicitly to ax_idx == 0.

                // We also have to collect data bounds of series that refer to a shared axis
                // referring explicitly to `ir_ax`. This is done in the inner loop with `ir_ax2`.

                let matcher = series::AxisMatcher {
                    plt_idx,
                    ax_idx,
                    id: ir_ax.id(),
                    title: ir_ax.title().map(|t| t.text()),
                };
                let mut bounds = None;

                for (plt_idx2, ir_plot2) in ir_plots.iter().enumerate() {
                    let Some(ir_plot2) = ir_plot2 else { continue };
                    let data = datas[plt_idx2].as_ref().unwrap();
                    let series = &data.series;
                    bounds = Series::unite_bounds(or, series, bounds, &matcher, plt_idx2)?;

                    for (ax_idx2, ir_ax2) in ir_plot2.or_axes(or).iter().enumerate() {
                        if let ir::axis::Scale::Shared(ax_ref2) = ir_ax2.scale() {
                            if matcher.matches_ref(Some(ax_ref2), plt_idx2)? {
                                let matcher = series::AxisMatcher {
                                    plt_idx: plt_idx2,
                                    ax_idx: ax_idx2,
                                    id: ir_ax2.id(),
                                    title: ir_ax2.title().map(|t| t.text()),
                                };
                                bounds =
                                    Series::unite_bounds(or, series, bounds, &matcher, plt_idx2)?;
                            }
                        }
                    }
                }

                let Some(bounds) = bounds else { continue };

                let off_plot = match ir_ax.side() {
                    ir::axis::Side::Main => &mut main_off_plot,
                    ir::axis::Side::Opposite => &mut opposite_off_plot,
                };
                let off_plot_area = *off_plot;
                *off_plot = true;

                // spine is drawn by axis:
                //  - when it is off plot area
                //  - when it is in plot area, but not a boxed plot
                let spine = match (off_plot_area, ir_plot.border()) {
                    (true, _) => ir_plot.border().cloned(),
                    (false, Some(ir::plot::Border::Box(_))) => None,
                    (false, Some(border)) => Some(border.clone()),
                    (false, None) => None,
                };

                let ax = self.setup_axis(
                    ir_ax,
                    &bounds,
                    Side::from_or_ir_side(or, ir_ax.side()),
                    size_along,
                    &datas[plt_idx].as_ref().unwrap().insets,
                    None,
                    spine,
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

            // track whether the main and opposite axes are directly attached to the plot area
            let mut main_off_plot = false;
            let mut opposite_off_plot = false;

            for (ax_idx, ir_ax) in ir_axes.iter().enumerate() {
                let ir::axis::Scale::Shared(ax_ref) = ir_ax.scale() else {
                    continue;
                };
                let (fig_ax_idx, _) = ir_plots
                    .or_find_axis(or, ax_ref, plt_idx)
                    .ok_or_else(|| Error::UnknownAxisRef(ax_ref.clone()))?;

                let info = ax_infos[fig_ax_idx]
                    .as_ref()
                    .ok_or_else(|| Error::IllegalAxisRef(ax_ref.clone()))?;

                let off_plot = match ir_ax.side() {
                    ir::axis::Side::Main => &mut main_off_plot,
                    ir::axis::Side::Opposite => &mut opposite_off_plot,
                };
                let off_plot_area = *off_plot;
                *off_plot = true;

                // spine is drawn by axis:
                //  - when it is off plot area
                //  - when it is in plot area, but not a boxed plot
                let spine = match (off_plot_area, ir_plot.border()) {
                    (true, _) => ir_plot.border().cloned(),
                    (false, Some(ir::plot::Border::Box(_))) => None,
                    (false, Some(border)) => Some(border.clone()),
                    (false, None) => None,
                };

                let axis = self.setup_axis(
                    ir_ax,
                    &info.0,
                    Side::from_or_ir_side(or, ir_ax.side()),
                    size_along,
                    &datas[plt_idx].as_ref().unwrap().insets,
                    Some(info.1.clone()),
                    spine,
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

impl Plots {
    pub fn update_series_data<D>(
        &mut self,
        data_source: &D,
    ) -> Result<(), Error>
    where
        D: data::Source,
    {
        for plot in self.plots.iter_mut() {
            if let Some(plot) = plot.as_mut() {
                plot.update_series_data(data_source)?;
            }
        }
        Ok(())
    }

    pub fn draw<S, T, P>(
        &self,
        surface: &mut S,
        style: &Style<T, P>,
    ) -> Result<(), Error>
    where
        S: render::Surface,
        T: Theme,
        P: Palette,
    {
        for plot in self.plots.iter() {
            if let Some(plot) = plot {
                plot.draw(surface, style)?;
            }
        }
        Ok(())
    }
}

impl Plot {
    fn update_series_data<D>(&mut self, data_source: &D) -> Result<(), Error>
    where
        D: data::Source,
    {
        let Some(axes) = &self.axes else {
            return Ok(());
        };

        for series in self.series.iter_mut() {
            let (x_ax_ref, y_ax_ref) = series.axes();
            let x = axes.or_find(Orientation::X, x_ax_ref)?;
            let y = axes.or_find(Orientation::Y, y_ax_ref)?;
            let (Some(x), Some(y)) = (x, y) else {
                unreachable!("Series without axis");
            };
            let cm = CoordMapXy {
                x: x.coord_map(),
                y: y.coord_map(),
            };

            series.update_data(data_source, &self.rect, &cm)?;
        }
        Ok(())
    }

    fn draw<S, T, P>(
        &self,
        surface: &mut S,
        style: &Style<T, P>,
    ) -> Result<(), Error>
    where
        S: render::Surface,
        T: Theme,
        P: Palette,
    {
        self.draw_background(surface, style)?;
        let Some(axes) = &self.axes else {
            self.draw_border_box(surface, style)?;
            return Ok(());
        };

        axes.draw_grids(surface, style, &self.rect)?;

        self.draw_lines(surface, style, axes, false)?;
        self.draw_series(surface, style)?;
        self.draw_lines(surface, style, axes, true)?;

        axes.draw(surface, style, &self.rect)?;
        self.draw_border_box(surface, style)?;

        if let Some((top_left, leg)) = self.legend.as_ref() {
            leg.draw(surface, style, top_left)?;
        }

        Ok(())
    }

    fn draw_background<S, T, P>(
        &self,
        surface: &mut S,
        style: &Style<T, P>,
    ) -> Result<(), Error>
    where
        S: render::Surface,
        T: Theme,
    {
        if let Some(fill) = &self.fill {
            surface.draw_rect(&render::Rect {
                rect: self.rect,
                fill: Some(fill.as_paint(style)),
                stroke: None,
                transform: None,
            })?;
        }
        Ok(())
    }

    fn draw_border_box<S, T, P>(
        &self,
        surface: &mut S,
        style: &Style<T, P>,
    ) -> Result<(), Error>
    where
        S: render::Surface,
        T: Theme,
    {
        // border is drawn by plot only when it is a box
        // otherwise, axes draw the border as spines
        let rect = self.rect;
        match self.border.as_ref() {
            None => Ok(()),
            Some(ir::plot::Border::Box(stroke)) => {
                surface.draw_rect(&render::Rect {
                    rect,
                    fill: None,
                    stroke: Some(stroke.as_stroke(style)),
                    transform: None,
                })?;
                Ok(())
            }
            Some(_) => Ok(()),
        }
    }

    fn draw_series<S, T, P>(
        &self,
        surface: &mut S,
        style: &Style<T, P>,
    ) -> Result<(), Error>
    where
        S: render::Surface,
        P: Palette,
    {
        let rect = self.rect;
        let series = &self.series;

        let clip = render::Clip {
            rect: &rect,
            transform: None,
        };
        surface.push_clip(&clip)?;

        for series in series.iter(){
            series.draw(surface, style)?;
        }
        surface.pop_clip()?;
        Ok(())
    }

    fn draw_lines<S, T, P>(
        &self,
        surface: &mut S,
        style: &Style<T, P>,
        axes: &Axes,
        above: bool,
    ) -> Result<(), Error>
    where
        S: render::Surface,
        T: Theme,
    {
        for line in self.lines.iter() {
            if line.above == above {
                let x_axis = axes
                    .or_find(Orientation::X, line.x_axis.as_ref())?
                    .ok_or_else(|| Error::UnknownAxisRef(line.x_axis.as_ref().unwrap().clone()))?;
                let y_axis = axes
                    .or_find(Orientation::Y, line.y_axis.as_ref())?
                    .ok_or_else(|| Error::UnknownAxisRef(line.y_axis.as_ref().unwrap().clone()))?;

                self.draw_line(surface, style, line, x_axis, y_axis, &self.rect)?;
            }
        }
        Ok(())
    }

    fn draw_line<S, T, P>(
        &self,
        surface: &mut S,
        style: &Style<T, P>,
        line: &PlotLine,
        x_axis: &Axis,
        y_axis: &Axis,
        plot_rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        S: render::Surface,
        T: Theme,
    {
        let (x, y) = (line.x, line.y);
        let (p1, p2) = match line.direction {
            ir::plot::Direction::Horizontal => {
                let y = y_axis.coord_map().map_coord_num(y);
                let p1 = geom::Point {
                    x: plot_rect.left(),
                    y,
                };
                let p2 = geom::Point {
                    x: plot_rect.right(),
                    y,
                };
                (p1, p2)
            }
            ir::plot::Direction::Vertical => {
                let x = x_axis.coord_map().map_coord_num(x);
                let p1 = geom::Point {
                    x,
                    y: plot_rect.top(),
                };
                let p2 = geom::Point {
                    x,
                    y: plot_rect.bottom(),
                };
                (p1, p2)
            }
            ir::plot::Direction::Slope(slope) => {
                // FIXME: raise error if either X or Y is logarithmic
                let x1 = x_axis.coord_map().map_coord_num(x);
                let y1 = y_axis.coord_map().map_coord_num(y);
                let x2 = x1 + 100.0;
                let y2 = y1 + 100.0 * slope;
                let p1 = geom::Point { x: x1, y: y1 };
                let p2 = geom::Point { x: x2, y: y2 };
                (p1, p2)
            }
            ir::plot::Direction::SecondPoint(x2, y2) => {
                let x1 = x_axis.coord_map().map_coord_num(x);
                let y1 = y_axis.coord_map().map_coord_num(y);
                let x2 = x_axis.coord_map().map_coord_num(x2);
                let y2 = y_axis.coord_map().map_coord_num(y2);
                let p1 = geom::Point { x: x1, y: y1 };
                let p2 = geom::Point { x: x2, y: y2 };
                (p1, p2)
            }
        };

        let p1 = geom::Point {
            x: p1.x + plot_rect.left(),
            y: plot_rect.bottom() - p1.y,
        };
        let p2 = geom::Point {
            x: p2.x + plot_rect.left(),
            y: plot_rect.bottom() - p2.y,
        };

        let points = plot_rect_intersections(plot_rect, &p1, &p2);
        if let Some([p1, p2]) = points {
            let mut path = geom::PathBuilder::with_capacity(2, 2);
            path.move_to(p1.x, p1.y);
            path.line_to(p2.x, p2.y);
            let path = path.finish().expect("Should be a valid path");
            let path = render::Path {
                path: &path,
                fill: None,
                stroke: Some(line.line.as_stroke(style)),
                transform: None,
            };
            surface.draw_path(&path)?;
        }

        Ok(())
    }
}

impl Axes {
    fn draw_grids<S, T, P>(
        &self,
        surface: &mut S,
        style: &Style<T, P>,
        rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        S: render::Surface,
        T: Theme,
    {
        for axis in self.x.iter() {
            axis.draw_minor_grids(surface, style, rect)?;
        }
        for axis in self.y.iter() {
            axis.draw_minor_grids(surface, style, rect)?;
        }
        for axis in self.x.iter() {
            axis.draw_major_grids(surface, style, rect)?;
        }
        for axis in self.y.iter() {
            axis.draw_major_grids(surface, style, rect)?;
        }
        Ok(())
    }

    fn draw<S, T, P>(
        &self,
        surface: &mut S,
        style: &Style<T, P>,
        plot_rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        S: render::Surface,
        T: Theme,
    {
        self.draw_side(surface, style, &self.x, Side::Top, plot_rect)?;
        self.draw_side(surface, style, &self.y, Side::Right, plot_rect)?;
        self.draw_side(surface, style, &self.x, Side::Bottom, plot_rect)?;
        self.draw_side(surface, style, &self.y, Side::Left, plot_rect)?;
        Ok(())
    }

    fn draw_side<S, T, P>(
        &self,
        surface: &mut S,
        style: &Style<T, P>,
        axes: &[Axis],
        side: Side,
        plot_rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        S: render::Surface,
        T: Theme,
    {
        let mut rect = *plot_rect;
        for axis in axes.iter() {
            if axis.side() == side {
                let shift = axis.draw(surface, style, &rect)?
                    + missing_params::AXIS_MARGIN
                    + missing_params::AXIS_SPINE_WIDTH;
                rect = match side {
                    Side::Top => rect.shifted_top_side(-shift),
                    Side::Right => rect.shifted_right_side(shift),
                    Side::Bottom => rect.shifted_bottom_side(shift),
                    Side::Left => rect.shifted_left_side(-shift),
                };
            }
        }
        Ok(())
    }
}

pub fn plot_rect_intersections(
    plot_rect: &geom::Rect,
    p1: &geom::Point,
    p2: &geom::Point,
) -> Option<[geom::Point; 2]> {
    let mut intersections: [Option<geom::Point>; 4] = [None; 4];

    // Parametric equation of the line: p1 + t * (p2 - p1)
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;

    // Function to calculate Y for given X (if dx != 0)
    let y_for_x = |x: f32| -> f32 {
        if dx == 0.0 {
            p1.y // vertical line
        } else {
            let t = (x - p1.x) / dx;
            p1.y + t * dy
        }
    };

    // Function to calculate X for given Y (if dx != 0)
    let x_for_y = |y: f32| -> f32 {
        if dy == 0.0 {
            p1.x // horizontal line
        } else {
            let t = (y - p1.y) / dy;
            p1.x + t * dx
        }
    };

    let mut idx = 0;

    // Intersection with vertical edges (left and right)
    if dx != 0.0 {
        for &x in &[plot_rect.x(), plot_rect.x() + plot_rect.width()] {
            let y = y_for_x(x);
            if y >= plot_rect.y() && y <= plot_rect.y() + plot_rect.height() {
                intersections[idx] = Some(geom::Point { x, y });
                idx += 1;
            }
        }
    }

    // Intersection with horizontal edges (top and bottom)
    if dy != 0.0 {
        for &y in &[plot_rect.y(), plot_rect.y() + plot_rect.height()] {
            let x = x_for_y(y);
            if x >= plot_rect.x() && x <= plot_rect.x() + plot_rect.width() {
                intersections[idx] = Some(geom::Point { x, y });
                idx += 1;
            }
        }
    }

    // We return result only if we have two points
    if idx == 2 {
        Some([intersections[0].unwrap(), intersections[1].unwrap()])
    } else {
        None
    }
}

fn legend_top_left(
    legend: &ir::PlotLegend,
    sz: geom::Size,
    plot_rect: &geom::Rect,
    outer_rect: &geom::Rect,
) -> geom::Point {
    match legend.pos() {
        ir::plot::LegendPos::OutTop => geom::Point {
            x: outer_rect.center_x() - sz.width() / 2.0,
            y: outer_rect.top(),
        },
        ir::plot::LegendPos::OutRight => geom::Point {
            x: outer_rect.right() - sz.width(),
            y: outer_rect.center_y() - sz.height() / 2.0,
        },
        ir::plot::LegendPos::OutBottom => geom::Point {
            x: outer_rect.center_x() - sz.width() / 2.0,
            y: outer_rect.bottom() - sz.height(),
        },
        ir::plot::LegendPos::OutLeft => geom::Point {
            x: outer_rect.left(),
            y: outer_rect.center_y() - sz.height() / 2.0,
        },
        ir::plot::LegendPos::InTop => geom::Point {
            x: plot_rect.center_x() - sz.width() / 2.0,
            y: plot_rect.top() + legend.margin(),
        },
        ir::plot::LegendPos::InTopRight => geom::Point {
            x: plot_rect.right() - sz.width() - legend.margin(),
            y: plot_rect.top() + legend.margin(),
        },
        ir::plot::LegendPos::InRight => geom::Point {
            x: plot_rect.right() - sz.width() - legend.margin(),
            y: plot_rect.center_y() - sz.height() / 2.0,
        },
        ir::plot::LegendPos::InBottomRight => geom::Point {
            x: plot_rect.right() - sz.width() - legend.margin(),
            y: plot_rect.bottom() - sz.height() - legend.margin(),
        },
        ir::plot::LegendPos::InBottom => geom::Point {
            x: plot_rect.center_x() - sz.width() / 2.0,
            y: plot_rect.bottom() - sz.height() - legend.margin(),
        },
        ir::plot::LegendPos::InBottomLeft => geom::Point {
            x: plot_rect.left() + legend.margin(),
            y: plot_rect.bottom() - sz.height() - legend.margin(),
        },
        ir::plot::LegendPos::InLeft => geom::Point {
            x: plot_rect.left() + legend.margin(),
            y: plot_rect.center_y() - sz.height() / 2.0,
        },
        ir::plot::LegendPos::InTopLeft => geom::Point {
            x: plot_rect.left() + legend.margin(),
            y: plot_rect.top() + legend.margin(),
        },
    }
}
