use std::iter::FusedIterator;
use std::sync::Arc;

use eidoplot_text as text;

use crate::drawing::legend::LegendBuilder;
use crate::drawing::plot::Plot;
use crate::drawing::{Ctx, Error, SurfWrapper, axis, plot};
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
                let mut i = 0;
                let mut y = rect.y();
                'row_loop: for r in 0..rows {
                    let mut x = rect.x();
                    for c in 0..cols {
                        let cols = cols as u32;
                        let idx = grid_idx(r, c, cols);
                        debug_assert_eq!(i, idx);
                        let Some(ir_plot) = subplots.plots().get(idx) else {
                            break 'row_loop;
                        };
                        let rect = geom::Rect::from_xywh(x, y, w, h);
                        plots.push(ctx.setup_plot(ir_plot, &rect)?);
                        i += 1;
                        x += w + space;
                    }
                    y += h + space;
                }

                let mut grid = PlotGrid { plots, cols };
                for c in 0..cols {
                    grid.align_axes_for_col(c);
                }
                for r in 0..rows {
                    grid.align_axes_for_row(r);
                }
                if subplots.share_x() {
                    for c in 0..cols {
                        let Some(bounds) = grid.united_x_bounds_for_col(c)? else {
                            continue;
                        };
                        let lst_idx = grid.iter_col_indices(c).last().unwrap();
                        // axis must be reconstructed from scratch because changing bounds 
                        // involve reconstructing all the ticks as well
                        let axis = {
                            let ir_plot = &subplots.plots()[lst_idx];
                            let plot = &grid.plots[lst_idx];
                            let side = axis::Side::Bottom;
                            let size_along = plot.rect().width();
                            let insets = plot.axes_insets().unwrap();
                            ctx.setup_axis(ir_plot.x_axis(), &bounds, side, size_along, &insets)?
                        };
                        let axis = Arc::new(axis);

                        for idx in grid.iter_col_indices(c) {
                            let config = if idx == lst_idx {
                                axis::SharedConfig::Shared(axis.clone())
                            } else {
                                axis::SharedConfig::SharedGrid(axis.clone())
                            };
                            grid.plots[idx].set_shared_axis_config(config);
                        }
                    }
                }
                for r in 0..rows {
                    for c in 0..cols {
                        let idx = grid_idx(r, c, cols);
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

#[inline]
fn grid_idx(r: u32, c: u32, cols: u32) -> usize {
    (r * cols + c) as usize
}

struct PlotGrid {
    plots: Vec<Plot>,
    cols: u32,
}

impl PlotGrid {
    fn iter_row_indices(&self, row: u32) -> RowColIter {
        RowColIter::iter_cols_for_row(self.plots.len(), self.cols, row)
    }

    fn iter_col_indices(&self, col: u32) -> RowColIter {
        RowColIter::iter_rows_for_col(self.plots.len(), self.cols, col)
    }
}

struct RowColIter {
    len: usize,
    cols: u32,
    r: u32,
    c: u32,
    r_adv: u32,
    c_adv: u32,
}

impl RowColIter {
    fn iter_cols_for_row(len: usize, cols: u32, row: u32) -> Self {
        RowColIter {
            len,
            cols,
            r: row,
            c: 0,
            r_adv: 0,
            c_adv: 1,
        }
    }
    fn iter_rows_for_col(len: usize, cols: u32, col: u32) -> Self {
        RowColIter {
            len,
            cols,
            r: 0,
            c: col,
            r_adv: 1,
            c_adv: 0,
        }
    }
}

impl Iterator for RowColIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.c >= self.cols {
            return None;
        }
        let idx = grid_idx(self.r, self.c, self.cols);
        self.r += self.r_adv;
        self.c += self.c_adv;
        if idx < self.len { Some(idx) } else { None }
    }
}

impl FusedIterator for RowColIter {}

impl PlotGrid {
    fn align_axes_for_col(&mut self, col: u32) {
        let mut left_right: Option<(f32, f32)> = None;
        for idx in self.iter_col_indices(col) {
            let pr = self.plots[idx].rect();
            if let Some((left, right)) = left_right.as_mut() {
                *left = left.max(pr.left());
                *right = right.min(pr.right());
            } else {
                left_right = Some((pr.left(), pr.right()));
            }
        }
        let Some((left, right)) = left_right else {
            return;
        };
        for idx in self.iter_col_indices(col) {
            let plot = &mut self.plots[idx];
            plot.rect_mut().set_left(left);
            plot.rect_mut().set_right(right);
        }
    }

    fn align_axes_for_row(&mut self, row: u32) {
        let mut top_bottom: Option<(f32, f32)> = None;
        for idx in self.iter_row_indices(row) {
            let pr = self.plots[idx].rect();
            if let Some((top, bottom)) = top_bottom.as_mut() {
                *top = top.max(pr.top());
                *bottom = bottom.min(pr.bottom());
            } else {
                top_bottom = Some((pr.top(), pr.bottom()));
            }
        }
        let Some((top, bottom)) = top_bottom else {
            return;
        };
        for idx in self.iter_row_indices(row) {
            let plot = &mut self.plots[idx];
            plot.rect_mut().set_top(top);
            plot.rect_mut().set_bottom(bottom);
        }
    }

    fn united_x_bounds_for_col(&mut self, col: u32) -> Result<Option<axis::Bounds>, Error> {
        let mut bounds = None;
        for idx in self.iter_col_indices(col) {
            let plot = &self.plots[idx];
            let xb = plot.x_bounds();
            match (xb, bounds.as_mut()) {
                (None, _) => (),
                (Some(xb), None) => {
                    bounds = Some(xb.to_bounds());
                }
                (Some(xb), Some(b)) => {
                    b.unite_with(&xb)?;
                }
            }
        }
        Ok(bounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_idx() {
        assert_eq!(grid_idx(0, 0, 1), 0);
        assert_eq!(grid_idx(1, 0, 1), 1);

        assert_eq!(grid_idx(0, 0, 3), 0);
        assert_eq!(grid_idx(0, 1, 3), 1);
        assert_eq!(grid_idx(0, 2, 3), 2);
        assert_eq!(grid_idx(1, 0, 3), 3);
        assert_eq!(grid_idx(1, 1, 3), 4);
        assert_eq!(grid_idx(1, 2, 3), 5);
        assert_eq!(grid_idx(2, 0, 3), 6);
        assert_eq!(grid_idx(2, 1, 3), 7);
        assert_eq!(grid_idx(2, 2, 3), 8);
    }

    #[test]
    fn test_iter_col_indices() {
        let iter = RowColIter::iter_cols_for_row(2, 1, 0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0]);

        let iter = RowColIter::iter_cols_for_row(2, 1, 1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![1]);

        let iter = RowColIter::iter_cols_for_row(3, 2, 0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0, 1]);

        let iter = RowColIter::iter_cols_for_row(3, 2, 1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![2]);

        let iter = RowColIter::iter_cols_for_row(4, 2, 0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0, 1]);

        let iter = RowColIter::iter_cols_for_row(4, 2, 1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![2, 3]);
    }

    #[test]
    fn test_iter_row_indices() {
        let iter = RowColIter::iter_rows_for_col(2, 1, 0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0, 1]);

        let iter = RowColIter::iter_rows_for_col(2, 1, 1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![]);

        let iter = RowColIter::iter_rows_for_col(3, 2, 0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0, 2]);

        let iter = RowColIter::iter_rows_for_col(3, 2, 1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![1]);

        let iter = RowColIter::iter_rows_for_col(4, 2, 0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0, 2]);

        let iter = RowColIter::iter_rows_for_col(4, 2, 1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![1, 3]);
    }
}
