use std::sync::Arc;

use crate::drawing::legend::{self, LegendBuilder};
use crate::drawing::{Ctx, Error, plot};
use crate::style::Theme;
use crate::text::font;
use crate::{data, geom, ir, missing_params, render, text};

#[derive(Debug)]
pub struct Figure {
    fig: ir::Figure,
    title: Option<(geom::Transform, text::RichText)>,
    legend: Option<(geom::Point, legend::Legend)>,
    plots: plot::Plots,
}

impl Figure {
    pub fn size(&self) -> geom::Size {
        self.fig.size()
    }

    pub fn prepare<D>(
        ir: ir::Figure,
        theme: Theme,
        fontdb: Option<Arc<font::Database>>,
        data_source: &D,
    ) -> Result<Self, Error>
    where
        D: data::Source,
    {
        let fontdb = fontdb.unwrap_or_else(|| Arc::new(crate::bundled_font_db()));
        let theme = Arc::new(theme);
        let ctx = Ctx::new(data_source, theme, fontdb);
        ctx.setup_figure(&ir)
    }

    pub fn update_series_data<D>(&mut self, data_source: &D) -> Result<(), Error>
    where
        D: data::Source,
    {
        self.plots
            .update_series_data(self.fig.plots(), data_source)?;
        Ok(())
    }

    pub fn draw<S>(&self, surface: &mut S, theme: &Theme) -> Result<(), Error>
    where
        S: render::Surface,
    {
        surface.prepare(self.fig.size())?;

        if let Some(fill) = self.fig.fill() {
            surface.fill(fill.as_paint(theme))?;
        }

        if let Some((transform, title)) = &self.title {
            let text = render::RichText {
                text: title,
                transform: *transform,
            };
            surface.draw_rich_text(&text)?;
        }

        if let Some((pos, legend)) = &self.legend {
            legend.draw(surface, theme, pos)?;
        }

        self.plots.draw(surface, theme, self.fig.plots())?;

        Ok(())
    }
}

impl<D> Ctx<'_, D>
where
    D: data::Source,
{
    fn setup_figure(&self, fig: &ir::Figure) -> Result<Figure, Error> {
        let mut rect =
            geom::Rect::from_ps(geom::Point { x: 0.0, y: 0.0 }, fig.size()).pad(fig.padding());

        let mut title = None;
        if let Some(fig_title) = fig.title() {
            let layout = text::rich::Layout::Horizontal(
                text::rich::Align::Center,
                text::line::VerAlign::Hanging.into(),
                Default::default(),
            );
            let rich = fig_title.to_rich_text(layout, self.fontdb(), self.theme())?;

            let anchor_x = rect.center_x();
            let anchor_y = rect.top();
            let transform = geom::Transform::from_translate(anchor_x, anchor_y);

            rect = rect
                .shifted_top_side(rich.visual_bbox().height() + missing_params::FIG_TITLE_MARGIN);

            title = Some((transform, rich));
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
            fig: fig.clone(),
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
            self.fontdb().clone(),
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
