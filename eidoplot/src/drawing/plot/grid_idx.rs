//! Module that handles indexing of subplots

use std::iter::FusedIterator;

#[derive(Debug, Clone, Copy)]
pub struct GridIdx {
    cols: usize,
    plots: usize,
}

impl<P: super::IrPlotsExt> From<&P> for GridIdx {
    fn from(s: &P) -> Self {
        GridIdx::new(s.plots().len(), s.cols() as usize)
    }
}

impl GridIdx {
    pub fn new(plots: usize, cols: usize) -> Self {
        assert!(cols > 0);
        GridIdx { plots, cols }
    }

    pub fn len(&self) -> usize {
        self.plots
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        calc_rows(self.plots, self.cols)
    }

    pub fn plot_idx(&self, row: usize, col: usize) -> Option<usize> {
        if col >= self.cols {
            return None;
        }
        let idx = grid_idx(row, col, self.cols);
        if idx < self.plots { Some(idx) } else { None }
    }

    pub fn iter_plot_indices_within_row(&self, row: usize) -> RowPlotIdxIter {
        RowPlotIdxIter::new(*self, row)
    }

    pub fn iter_plot_indices_within_col(&self, col: usize) -> ColPlotIdxIter {
        ColPlotIdxIter::new(*self, col)
    }
}

/// Iterate plots indices within a given row
#[derive(Debug, Clone, Copy)]
pub struct RowPlotIdxIter {
    grid: GridIdx,
    row: usize,
    c: usize,
}

impl RowPlotIdxIter {
    fn new(grid: GridIdx, row: usize) -> Self {
        RowPlotIdxIter { grid, row, c: 0 }
    }
}

impl Iterator for RowPlotIdxIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.grid.plot_idx(self.row, self.c);
        self.c += 1;
        idx
    }
}

impl FusedIterator for RowPlotIdxIter {}

/// Iterate plots indices within a given column
#[derive(Debug, Clone, Copy)]
pub struct ColPlotIdxIter {
    grid: GridIdx,
    col: usize,
    r: usize,
}

impl ColPlotIdxIter {
    fn new(grid: GridIdx, col: usize) -> Self {
        ColPlotIdxIter { grid, col, r: 0 }
    }
}

impl Iterator for ColPlotIdxIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.grid.plot_idx(self.r, self.col);
        self.r += 1;
        idx
    }
}

impl FusedIterator for ColPlotIdxIter {}

#[inline]
fn calc_rows(nplots: usize, ncols: usize) -> usize {
    (nplots + ncols - 1) / ncols
}

#[inline]
fn grid_idx(r: usize, c: usize, cols: usize) -> usize {
    r * cols + c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_rows() {
        assert_eq!(calc_rows(0, 1), 0);
        assert_eq!(calc_rows(1, 1), 1);
        assert_eq!(calc_rows(1, 2), 1);
        assert_eq!(calc_rows(2, 1), 2);
        assert_eq!(calc_rows(2, 2), 1);
        assert_eq!(calc_rows(3, 2), 2);
        assert_eq!(calc_rows(4, 2), 2);
        assert_eq!(calc_rows(5, 2), 3);
    }

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
        let grid = GridIdx { plots: 2, cols: 1 };
        let iter = grid.iter_plot_indices_within_row(0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0]);

        let iter = grid.iter_plot_indices_within_row(1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![1]);

        let grid = GridIdx { plots: 3, cols: 2 };
        let iter = grid.iter_plot_indices_within_row(0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0, 1]);

        let iter = grid.iter_plot_indices_within_row(1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![2]);

        let grid = GridIdx { plots: 4, cols: 2 };
        let iter = grid.iter_plot_indices_within_row(0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0, 1]);

        let iter = grid.iter_plot_indices_within_row(1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![2, 3]);
    }

    #[test]
    fn test_iter_row_indices() {
        let grid = GridIdx { plots: 2, cols: 1 };
        let iter = grid.iter_plot_indices_within_col(0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0, 1]);

        let iter = grid.iter_plot_indices_within_col(1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![]);

        let grid = GridIdx { plots: 3, cols: 2 };
        let iter = grid.iter_plot_indices_within_col(0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0, 2]);

        let iter = grid.iter_plot_indices_within_col(1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![1]);

        let grid = GridIdx { plots: 4, cols: 2 };
        let iter = grid.iter_plot_indices_within_col(0);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![0, 2]);

        let iter = grid.iter_plot_indices_within_col(1);
        let v: Vec<usize> = iter.collect();
        assert_eq!(v, vec![1, 3]);
    }
}
