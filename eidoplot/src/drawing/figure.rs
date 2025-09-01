use eidoplot_text as text;

use crate::drawing::legend::Legend;
use crate::drawing::plot;
use crate::drawing::{Ctx, Error, SurfWrapper};
use crate::render::{self, Surface as _};
use crate::{data, geom, ir, missing_params, style};

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    pub fn draw_toplevel_figure<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        fig: &ir::Figure,
    ) -> Result<(), Error>
    where
        D: data::Source,
        T: style::Theme,
    {
        self.prepare(fig.size())?;
        if let Some(fill) = fig.fill() {
            self.fill(fill.as_paint(ctx.theme()))?;
        }

        let mut rect = geom::Rect::from_ps(geom::Point::ORIGIN, fig.size());
        let layout = fig.layout().cloned().unwrap_or_default();
        if let Some(padding) = layout.padding() {
            rect = rect.pad(padding);
        }

        if let Some(title) = fig.title() {
            let title_rect = geom::Rect::from_xywh(
                rect.x(),
                rect.y(),
                rect.width(),
                title.font().size + 2.0 * missing_params::FIG_TITLE_MARGIN,
            );
            let text = render::Text {
                text: title.text(),
                font: title.font().font(),
                font_size: title.font().size,
                fill: ctx.theme().foreground().into(),
                options: text::layout::Options {
                    hor_align: text::layout::HorAlign::Center,
                    ver_align: text::layout::VerAlign::Center,
                    ..Default::default()
                },
                transform: Some(&title_rect.center().translation()),
            };
            self.draw_text(&text)?;
            rect = rect.shifted_top_side(title_rect.height());
        }

        if let Some(legend) = fig.legend() {
            self.draw_figure_legend(ctx, fig, legend, &mut rect)?;
        }

        self.draw_figure_plots(ctx, fig.plots(), &rect)?;

        Ok(())
    }

    fn draw_figure_legend<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        fig: &ir::Figure,
        legend: &ir::FigLegend,
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
        for plot in fig.plots().iter() {
            plot::for_each_series(plot, |s| {
                if let Some(entry) = s.legend_entry() {
                    dlegend.add_entry(idx, entry)?;
                    idx += 1;
                }
                Ok(())
            })?;
        }

        let Some(sz) = dlegend.layout() else {
            return Ok(());
        };
        
        let top_left = match legend.pos() {
            ir::figure::LegendPos::Top => {
                let tl = geom::Point::new(rect.center_x() - sz.width() / 2.0, rect.top());
                rect.shift_top_side(sz.height() + legend.margin());
                tl
            }
            ir::figure::LegendPos::Right => {
                rect.shift_right_side(-sz.width() - legend.margin());
                geom::Point::new(
                    rect.right() + legend.margin(),
                    rect.center_y() - sz.height() / 2.0,
                )
            }
            ir::figure::LegendPos::Bottom => {
                rect.shift_bottom_side(-sz.height() - legend.margin());
                geom::Point::new(
                    rect.center_x() - sz.width() / 2.0,
                    rect.bottom() + legend.margin(),
                )
            }
            ir::figure::LegendPos::Left => {
                let tl = geom::Point::new(rect.left(), rect.center_y() - sz.height() / 2.0);
                rect.shift_left_side(sz.width() + legend.margin());
                tl
            }
        };
        self.draw_legend(ctx, &dlegend, &top_left)?;
        Ok(())
    }

    fn draw_figure_plots<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        plots: &ir::figure::Plots,
        rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        D: data::Source,
        T: style::Theme,
    {
        match plots {
            ir::figure::Plots::Plot(plot) => Ok(self.draw_plot(ctx, plot, rect)?),
            ir::figure::Plots::Subplots(subplots) => {
                let w = (rect.width() - subplots.space * (subplots.cols - 1) as f32)
                    / subplots.cols as f32;
                let h = (rect.height() - subplots.space * (subplots.rows - 1) as f32)
                    / subplots.rows as f32;
                let mut y = rect.y();
                for c in 0..subplots.cols {
                    let mut x = rect.x();
                    for r in 0..subplots.rows {
                        let cols = subplots.cols as u32;
                        let idx = (r * cols + c) as usize;
                        let plot = &subplots.plots[idx];
                        self.draw_plot(ctx, plot, &geom::Rect::from_xywh(x, y, w, h))?;
                        x += w + subplots.space;
                    }
                    y += h + subplots.space;
                }
                Ok(())
            }
        }
    }
}
