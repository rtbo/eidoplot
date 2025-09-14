use eidoplot_text as text;

use crate::drawing::legend::LegendBuilder;
use crate::drawing::plot::Plot;
use crate::drawing::{Ctx, Error, SurfWrapper, plot};
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

        let mut rect = geom::Rect::from_ps(geom::Point::ORIGIN, fig.size()).pad(fig.padding());

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
        let mut builder = LegendBuilder::from_ir(
            legend.legend(),
            legend.pos().prefers_vertical(),
            rect.width(),
            ctx.fontdb().clone(),
        );

        let mut idx = 0;
        for plot in fig.plots().iter() {
            plot::for_each_series(plot, |s| {
                if let Some(entry) = s.legend_entry() {
                    builder.add_entry(idx, entry)?;
                    idx += 1;
                }
                Ok(())
            })?;
        }

        let Some(leg) = builder.layout() else {
            return Ok(());
        };

        let sz = leg.size();
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
        self.draw_legend(ctx, &leg, &top_left)?;
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
            ir::figure::Plots::Plot(ir_plot) => {
                let plot = ctx.setup_plot(ir_plot, rect)?;
                self.draw_plot(ctx, ir_plot, &plot)
            }
            ir::figure::Plots::Subplots(subplots) => {
                // collect plots in a grid
                let mut plots = Vec::with_capacity(subplots.plots().len());
                let (rows, cols) = (subplots.rows(), subplots.cols());
                let space = subplots.space();
                let w = (rect.width() - space * (subplots.cols() - 1) as f32) / cols as f32;
                let h = (rect.height() - space * (rows - 1) as f32) / rows as f32;
                let mut y = rect.y();
                for r in 0..rows {
                    let mut x = rect.x();
                    for c in 0..cols {
                        let cols = cols as u32;
                        let idx = (r * cols + c) as usize;
                        let ir_plot = &subplots.plots()[idx];
                        let rect = geom::Rect::from_xywh(x, y, w, h);
                        plots.push(ctx.setup_plot(ir_plot, &rect)?);
                        x += w + space;
                    }
                    y += h + space;
                }
                let mut grid = PlotGrid{plots, rows, cols};
                for c in 0..cols {
                    grid.align_axes_for_col(c);
                }
                for r in 0..rows {
                    grid.align_axes_for_row(r);
                }
                for r in 0..rows {
                    for c in 0..cols {
                        let idx = (r * cols + c) as usize;
                        let plot = &grid.plots[idx];
                        let ir_plot = &subplots.plots()[idx];
                        self.draw_plot(ctx, ir_plot, plot)?;
                    }
                }
                Ok(())
            }
        }
    }
}

struct PlotGrid {
    plots: Vec<Plot>,
    rows: u32,
    cols: u32,
}

impl PlotGrid {
    fn align_axes_for_col(&mut self, col: u32) {
        let mut left_right: Option<(f32, f32)> = None;
        for r in 0..self.rows {
            let idx = (r * self.cols + col) as usize;
            let pr = self.plots[idx].rect();
            if let Some((left, right)) = left_right.as_mut() {
                *left = left.max(pr.left());
                *right = right.min(pr.right());
            } else {
                left_right = Some((pr.left(), pr.right()));
            }
        }
        let Some((left, right)) = left_right else { return };
        for r in 0..self.rows {
            let idx = (r * self.cols + col) as usize;
            let plot = &mut self.plots[idx];
            plot.rect_mut().set_left(left);
            plot.rect_mut().set_right(right);
        }
    }

    fn align_axes_for_row(&mut self, row: u32) {
        let mut top_bottom: Option<(f32, f32)> = None;
        for c in 0..self.cols {
            let idx = (row * self.cols + c) as usize;
            let pr = self.plots[idx].rect();
            if let Some((top, bottom)) = top_bottom.as_mut() {
                *top = top.max(pr.top());
                *bottom = bottom.min(pr.bottom());
            } else {
                top_bottom = Some((pr.top(), pr.bottom()));
            }
        }
        let Some((top, bottom)) = top_bottom else { return };
        for c in 0..self.cols {
            let idx = (row * self.cols + c) as usize;
            let plot = &mut self.plots[idx];
            plot.rect_mut().set_top(top);
            plot.rect_mut().set_bottom(bottom);
        }
    }
}
