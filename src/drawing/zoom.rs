//! Module for handling zoom operations and views in figures.
use std::sync::Arc;

use crate::drawing::scale::CoordMap;
use crate::drawing::{fig_x_to_plot_x, fig_y_to_plot_y};
use crate::des::PlotIdx;
use crate::{data, fontdb, geom};

/// A mask to indicate which axes are affected by a zoom operation.
#[derive(Debug, Clone, Copy)]
pub struct AxisMask(u32);

impl Default for AxisMask {
    fn default() -> Self {
        AxisMask(0xffff_ffff)
    }
}

impl AxisMask {
    /// Create a mask with no axes selected.
    pub fn none() -> Self {
        AxisMask(0)
    }

    /// Create a mask with all axes selected.
    pub fn all() -> Self {
        AxisMask(0xffff_ffff)
    }

    /// Check if the mask contains the given axis index.
    pub fn contains(&self, axis_idx: u32) -> bool {
        (self.0 & (1 << axis_idx)) != 0
    }

    /// Insert the given axis index into the mask.
    pub fn insert(&mut self, axis_idx: u32) {
        self.0 |= 1 << axis_idx;
    }

    /// Remove the given axis index from the mask.
    pub fn remove(&mut self, axis_idx: u32) {
        self.0 &= !(1 << axis_idx);
    }
}

/// A zoom operation to be applied to a figure plot.
/// The zoom is defined by a rectangle in figure coordinates,
/// and masks indicating which axes are affected.
///
/// In order to zoom-in, the rectangle should be smaller than the current view.
/// To zoom-out, the rectangle should be larger than the current view.
#[derive(Debug, Clone, Copy)]
pub struct Zoom {
    rect: geom::Rect,
    x_axis_mask: AxisMask,
    y_axis_mask: AxisMask,
}

impl Zoom {
    /// Create a new zoom operation with the given rectangle.
    pub fn new(rect: geom::Rect) -> Self {
        Zoom {
            rect,
            x_axis_mask: AxisMask::default(),
            y_axis_mask: AxisMask::default(),
        }
    }

    /// Set the mask of x axes affected by the zoom.
    pub fn x_axis_mask(mut self, mask: AxisMask) -> Self {
        self.x_axis_mask = mask;
        self
    }

    /// Set the mask of y axes affected by the zoom.
    pub fn y_axis_mask(mut self, mask: AxisMask) -> Self {
        self.y_axis_mask = mask;
        self
    }
}

/// A view of a plot within a figure, capturing the current state of its axes.
#[derive(Debug, Clone)]
pub struct PlotView {
    idx: PlotIdx,
    rect: geom::Rect,
    x_infos: Vec<Arc<dyn CoordMap>>,
    y_infos: Vec<Arc<dyn CoordMap>>,
}

impl PlotView {
    /// Get the index of the plot this view corresponds to.
    pub fn idx(&self) -> PlotIdx {
        self.idx
    }

    /// Get the rectangle of the plot in figure units.
    pub fn rect(&self) -> geom::Rect {
        self.rect
    }

    /// Apply a zoom operation to this plot view, returning a new plot view.
    pub fn apply_zoom(&self, zoom: &Zoom) -> PlotView {
        let x_infos = self
            .x_infos
            .iter()
            .enumerate()
            .map(|(i, info)| {
                if zoom.x_axis_mask.contains(i as u32) {
                    info.create_view(
                        fig_x_to_plot_x(&self.rect, zoom.rect.left()),
                        fig_x_to_plot_x(&self.rect, zoom.rect.right()),
                    )
                } else {
                    info.clone()
                }
            })
            .collect();

        let y_infos = self
            .y_infos
            .iter()
            .enumerate()
            .map(|(i, info)| {
                if zoom.y_axis_mask.contains(i as u32) {
                    info.create_view(
                        fig_y_to_plot_y(&self.rect, zoom.rect.bottom()),
                        fig_y_to_plot_y(&self.rect, zoom.rect.top()),
                    )
                } else {
                    info.clone()
                }
            })
            .collect();

        PlotView {
            idx: self.idx,
            rect: self.rect,
            x_infos,
            y_infos,
        }
    }
}

/// A view of a figure, capturing the current state of all its plots.
#[derive(Debug, Clone)]
pub struct FigureView {
    plot_views: Vec<Option<PlotView>>,
}

impl super::PreparedFigure {
    /// Get the current view of the figure.
    pub fn view(&self) -> FigureView {
        let mut plot_views = Vec::with_capacity(self.plots.len());

        for idx in self.plots.iter_indices() {
            plot_views.push(self.plot_view(idx));
        }

        FigureView { plot_views }
    }

    /// Get the current view of a given plot in the figure.
    pub fn plot_view(&self, idx: PlotIdx) -> Option<PlotView> {
        let Some(plot) = self.plots.plot(idx) else {
            return None;
        };
        let Some(axes) = plot.axes() else {
            return None;
        };

        let x_infos = axes.x().iter().map(|axis| axis.coord_map()).collect();
        let y_infos = axes.y().iter().map(|axis| axis.coord_map()).collect();

        Some(PlotView {
            idx,
            rect: *plot.rect(),
            x_infos,
            y_infos,
        })
    }

    /// Apply the given view to the figure.
    pub fn apply_view<D>(
        &mut self,
        view: &FigureView,
        data_source: &D,
        fontdb: Option<&fontdb::Database>,
    ) -> Result<(), super::Error>
    where
        D: data::Source + ?Sized,
    {
        for view in &view.plot_views {
            if let Some(plot_view) = view {
                self.apply_plot_view(plot_view.clone(), data_source, fontdb)?;
            }
        }
        Ok(())
    }

    /// Apply a plot view to a given plot in the figure.
    /// This will reconstruct the axis scales and the series accordingly.
    ///
    /// Panics if the plot index is invalid or if the number of axes
    /// in the view does not match the number of axes in the plot.
    /// Also panics if the font database is not provided when needed
    /// and no bundled font feature is enabled.
    pub fn apply_plot_view<D>(
        &mut self,
        view: PlotView,
        data_source: &D,
        fontdb: Option<&fontdb::Database>,
    ) -> Result<(), super::Error>
    where
        D: data::Source + ?Sized,
    {
        let idx = view.idx();

        let Some(plot) = self.plots.plot_mut(idx) else {
            panic!("Invalid plot index: {:?}", idx);
        };
        let Some(axes) = plot.axes_mut() else {
            assert!(view.x_infos.is_empty() && view.y_infos.is_empty());
            return Ok(());
        };

        assert!(
            axes.x().len() == view.x_infos.len() && axes.y().len() == view.y_infos.len(),
            "Number of axes in view does not match number of axes in plot"
        );

        super::with_ctx(data_source, fontdb, |ctx| {
            for (x_ax, new_x_cm) in axes.x_mut().iter_mut().zip(view.x_infos.iter()) {
                ctx.axis_set_coord_map(x_ax, new_x_cm.clone())?;
            }
            for (y_ax, new_y_cm) in axes.y_mut().iter_mut().zip(view.y_infos.iter()) {
                ctx.axis_set_coord_map(y_ax, new_y_cm.clone())?;
            }
            Ok::<(), super::Error>(())
        })?;

        self.update_series_data(data_source)?;

        Ok(())
    }

    /// Convenience method to apply a zoom to a given plot in the figure.
    /// This method will retrieve the current plot view, apply the zoom to it,
    /// and then apply the updated plot view back to the figure.
    pub fn apply_zoom<D>(
        &mut self,
        idx: PlotIdx,
        zoom: &Zoom,
        data_source: &D,
        fontdb: Option<&fontdb::Database>,
    ) -> Result<(), super::Error>
    where
        D: data::Source + ?Sized,
    {
        let mut plot_view = self.plot_view(idx).expect("Invalid plot index for zoom");
        plot_view = plot_view.apply_zoom(zoom);
        self.apply_plot_view(plot_view, data_source, fontdb)?;
        Ok(())
    }
}
