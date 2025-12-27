use crate::drawing::legend::{self, LegendBuilder};
use crate::drawing::{Ctx, Error, plot};
use crate::style::theme::{self, Theme};
use crate::{Style, data, geom, ir, missing_params, render, style, text};

/// A figure that has been prepared for drawing. See [`Figure::prepare`].
/// It contains all the necessary data and layout information.
///
/// The texts have been shaped, laid out and transformed to paths.
/// Therefore, the fonts are no longer needed at draw time.
///
/// The colors, strokes and fills will be resolved at draw time using the given theme.
#[derive(Debug)]
pub struct Figure {
    pub(super) size: geom::Size,
    pub(super) fill: Option<theme::Fill>,
    pub(super) title: Option<(geom::Transform, super::Text)>,
    pub(super) legend: Option<(geom::Point, legend::Legend)>,
    pub(super) plots: plot::Plots,
}

impl Clone for Figure {
    fn clone(&self) -> Self {
        Self {
            size: self.size,
            fill: self.fill.clone(),
            title: self.title.clone(),
            legend: self.legend.clone(),
            plots: self.plots.clone(),
        }
    }
}

impl Figure {
    /// The size of the figure in figure units
    pub fn size(&self) -> geom::Size {
        self.size
    }

    ///
    pub fn plot_indices(&self) -> impl Iterator<Item = ir::PlotIdx> + '_ {
        self.plots.iter_indices()
    }

    pub(super) fn _title_area(&self) -> Option<geom::Rect> {
        self.title.as_ref().and_then(|(transform, text)| {
            text.bbox.as_ref().map(|bbox| bbox.transform(transform))
        })
    }

    pub(super) fn _legend_area(&self) -> Option<geom::Rect> {
        self.legend.as_ref().map(|(pos, legend)| {
            geom::Rect::from_ps(*pos, legend.size())
        })
    }

    /// Update the data for all series in the figure from the given data source.
    /// This allows reusing the same prepared figure with different data and to perform
    /// efficient redraws in real-time applications.
    /// Note that axis bounds are not recomputed, only the series data is updated,
    /// within the same axes bounds.
    pub fn update_series_data<D>(&mut self, data_source: &D) -> Result<(), Error>
    where
        D: data::Source,
    {
        self.plots.update_series_data(data_source)?;
        Ok(())
    }
}

impl<D> Ctx<'_, D>
where
    D: data::Source,
{
    pub fn setup_figure(&self, fig: &ir::Figure) -> Result<Figure, Error> {
        let mut rect =
            geom::Rect::from_ps(geom::Point { x: 0.0, y: 0.0 }, fig.size()).pad(fig.padding());

        let mut title = None;
        if let Some(fig_title) = fig.title() {
            let layout = text::rich::Layout::Horizontal(
                text::rich::Align::Center,
                text::line::VerAlign::Hanging.into(),
                Default::default(),
            );
            let rich = fig_title.to_rich_text(layout, self.fontdb())?;
            let paths = super::Text::from_rich_text(&rich, self.fontdb())?;

            let anchor_x = rect.center_x();
            let anchor_y = rect.top();
            let transform = geom::Transform::from_translate(anchor_x, anchor_y);

            rect = rect.shifted_top_side(
                rich.visual_bbox().map_or(0.0, |bbox| bbox.height())
                    + missing_params::FIG_TITLE_MARGIN,
            );

            title = Some((transform, paths));
        }

        let mut legend = None;
        if let Some(fig_legend) = fig.legend() {
            let leg = self.prepare_legend(fig, fig_legend, &mut rect)?;
            if let Some((pos, leg)) = leg {
                legend = Some((pos, leg));
            }
        }

        let plots = self.setup_plots(fig.plots(), &rect)?;

        Ok(Figure {
            size: fig.size(),
            fill: fig.fill().clone(),
            title,
            legend,
            plots,
        })
    }

    fn prepare_legend(
        &self,
        fig: &ir::Figure,
        legend: &ir::FigLegend,
        rect: &mut geom::Rect,
    ) -> Result<Option<(geom::Point, legend::Legend)>, Error> {
        let mut builder = LegendBuilder::from_ir(
            legend.legend(),
            legend.pos().prefers_vertical(),
            rect.width(),
            self.fontdb(),
        );

        let mut idx = 0;
        for plot in fig.plots().iter().filter_map(|p| p) {
            plot::for_each_series(plot, |s| {
                if let Some(entry) = s.legend_entry() {
                    builder.add_entry(idx, entry)?;
                    idx += 1;
                }
                Ok(())
            })?;
        }

        let Some(leg) = builder.layout() else {
            return Ok(None);
        };

        let sz = leg.size();
        let top_left = match legend.pos() {
            ir::figure::LegendPos::Top => {
                let tl = geom::Point {
                    x: rect.center_x() - sz.width() / 2.0,
                    y: rect.top(),
                };
                rect.shift_top_side(sz.height() + legend.margin());
                tl
            }
            ir::figure::LegendPos::Right => {
                rect.shift_right_side(-sz.width() - legend.margin());
                geom::Point {
                    x: rect.right() + legend.margin(),
                    y: rect.center_y() - sz.height() / 2.0,
                }
            }
            ir::figure::LegendPos::Bottom => {
                rect.shift_bottom_side(-sz.height() - legend.margin());
                geom::Point {
                    x: rect.center_x() - sz.width() / 2.0,
                    y: rect.bottom() + legend.margin(),
                }
            }
            ir::figure::LegendPos::Left => {
                let tl = geom::Point {
                    x: rect.left(),
                    y: rect.center_y() - sz.height() / 2.0,
                };
                rect.shift_left_side(sz.width() + legend.margin());
                tl
            }
        };
        Ok(Some((top_left, leg)))
    }
}

impl Figure {
    /// Draw the figure on the given rendering surface, using the given theme
    /// The surface content will be replaced by the figure drawing.
    pub fn draw<S, T, P>(&self, surface: &mut S, style: &Style<T, P>) -> Result<(), Error>
    where
        S: render::Surface,
        T: Theme,
        P: style::series::Palette,
    {
        surface.prepare(self.size)?;

        if let Some(fill) = &self.fill {
            surface.fill(fill.as_paint(style))?;
        }

        if let Some((transform, title)) = &self.title {
            title.draw(surface, style, Some(transform))?;
        }

        if let Some((pos, legend)) = &self.legend {
            legend.draw(surface, style, pos)?;
        }

        self.plots.draw(surface, style)?;

        Ok(())
    }
}
