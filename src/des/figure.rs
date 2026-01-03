//! Figure IR structures
use std::iter::FusedIterator;

use crate::geom;
use crate::des::{Legend, Plot, PlotIdx, Subplots};
use crate::style::{defaults, theme};

super::define_rich_text_structs!(Title, TitleProps, TitleOptProps);

impl Default for TitleProps {
    fn default() -> Self {
        TitleProps::new(defaults::TITLE_FONT_SIZE)
    }
}

/// Position of the legend relatively to the figure
#[derive(Debug, Clone, Copy, Default)]
pub enum LegendPos {
    /// Position the legend at the top of the figure
    Top,
    /// Position the legend at the right of the figure
    Right,
    /// Position the legend at the bottom of the figure (default)
    #[default]
    Bottom,
    /// Position the legend at the left of the figure
    Left,
}

impl LegendPos {
    /// Whether the position prefers vertical layout if no amount of column is defined
    pub fn prefers_vertical(&self) -> bool {
        matches!(self, LegendPos::Left | LegendPos::Right)
    }
}

/// A per-figure legend
#[derive(Debug, Clone)]
pub struct FigLegend {
    pos: LegendPos,
    legend: Legend,
    margin: f32,
}

impl FigLegend {
    /// Build a new legend
    pub fn new(legend: Legend) -> Self {
        FigLegend {
            pos: Default::default(),
            legend,
            margin: defaults::LEGEND_MARGIN,
        }
    }

    /// The position of the legend relatively to the plot
    pub fn pos(&self) -> LegendPos {
        self.pos
    }

    /// The underlying legend
    pub fn legend(&self) -> &Legend {
        &self.legend
    }

    /// The margin around the legend
    pub fn margin(&self) -> f32 {
        self.margin
    }

    /// Set the position of the legend
    pub fn with_pos(self, pos: LegendPos) -> Self {
        Self { pos, ..self }
    }

    /// Set the margin around the legend
    pub fn with_margin(self, margin: f32) -> Self {
        Self { margin, ..self }
    }
}

impl Default for FigLegend {
    fn default() -> Self {
        FigLegend::new(Legend::default())
    }
}

impl From<LegendPos> for FigLegend {
    fn from(pos: LegendPos) -> Self {
        FigLegend::new(Legend::default()).with_pos(pos)
    }
}

/// Figure structure. This is the top-level structure representing a figure to be drawn.
#[derive(Debug, Clone)]
pub struct Figure {
    plots: Plots,

    title: Option<Title>,
    size: geom::Size,
    legend: Option<FigLegend>,
    fill: Option<theme::Fill>,
    padding: geom::Padding,
}

impl Figure {
    /// Create a new figure with the given plots
    pub fn new(plots: Plots) -> Figure {
        Figure {
            plots,

            title: None,
            size: defaults::FIG_SIZE,
            legend: None,
            fill: Some(theme::Col::Background.into()),
            padding: defaults::FIG_PADDING,
        }
    }

    /// Set the title and return self for chaining
    pub fn with_title(self, title: Title) -> Self {
        Figure {
            title: Some(title),
            ..self
        }
    }

    /// Set the size and return self for chaining
    pub fn with_size(self, size: geom::Size) -> Self {
        Figure { size: size, ..self }
    }

    /// Set the legend and return self for chaining
    pub fn with_legend(self, legend: FigLegend) -> Self {
        Figure {
            legend: Some(legend),
            ..self
        }
    }

    /// Set the fill and return self for chaining
    /// Set this to None for a transparent background
    pub fn with_fill(self, fill: Option<theme::Fill>) -> Self {
        Figure { fill, ..self }
    }

    /// Set the padding and return self for chaining
    pub fn with_padding(self, padding: geom::Padding) -> Self {
        Figure { padding, ..self }
    }

    /// Get the size of the figure
    pub fn size(&self) -> geom::Size {
        self.size
    }

    /// Get the title of the figure
    pub fn title(&self) -> Option<&Title> {
        self.title.as_ref()
    }

    /// Get the plots of the figure
    pub fn plots(&self) -> &Plots {
        &self.plots
    }

    /// Get a mutable reference to the plots of the figure
    pub fn plots_mut(&mut self) -> &mut Plots {
        &mut self.plots
    }

    /// Get the legend of the figure
    pub fn legend(&self) -> Option<&FigLegend> {
        self.legend.as_ref()
    }

    /// Get the fill of the figure
    pub fn fill(&self) -> Option<theme::Fill> {
        self.fill
    }

    /// Get the padding of the figure
    pub fn padding(&self) -> &geom::Padding {
        &self.padding
    }
}

/// Collection of plots for a figure
#[derive(Debug, Clone)]
pub enum Plots {
    /// Unique plot on the figure
    Plot(Plot),
    /// Subplots on the same figure
    Subplots(Subplots),
}

impl From<Plot> for Plots {
    fn from(plot: Plot) -> Self {
        Plots::Plot(plot)
    }
}

impl From<Subplots> for Plots {
    fn from(subplots: Subplots) -> Self {
        Plots::Subplots(subplots)
    }
}

impl Plots {
    /// The number of plots in this figure
    pub fn len(&self) -> usize {
        match self {
            Plots::Plot(..) => 1,
            Plots::Subplots(subplots) => subplots.len(),
        }
    }

    /// The number of rows of plots in this figure
    pub fn rows(&self) -> u32 {
        match self {
            Plots::Plot(..) => 1,
            Plots::Subplots(subplots) => subplots.rows(),
        }
    }

    /// The number of columns of plots in this figure
    pub fn cols(&self) -> u32 {
        match self {
            Plots::Plot(..) => 1,
            Plots::Subplots(subplots) => subplots.cols(),
        }
    }

    /// Get a reference to a plot at the given row and column
    pub fn plot(&self, idx: impl Into<PlotIdx>) -> Option<&Plot> {
        let idx = idx.into();
        match self {
            Plots::Plot(plot) if idx.row == 0 && idx.col == 0 => Some(plot),
            Plots::Plot(..) => None,
            Plots::Subplots(subplots) => subplots.plot(idx),
        }
    }

    /// Get a mutable reference to a plot at the given row and column
    pub fn plot_mut(&mut self, idx: impl Into<PlotIdx>) -> Option<&mut Plot> {
        let idx = idx.into();
        match self {
            Plots::Plot(plot) if idx.row == 0 && idx.col == 0 => Some(plot),
            Plots::Plot(..) => None,
            Plots::Subplots(subplots) => subplots.plot_mut(idx),
        }
    }

    /// Returns an iterator over the plots in this figure.
    /// The plots are iterated row by row, from top to bottom and left to right.
    pub fn iter(&self) -> PlotIter<'_> {
        PlotIter {
            plots: self,
            idx: (0, 0).into(),
        }
    }

    /// The space between plots in this figure (only for subplots)
    pub fn space(&self) -> f32 {
        match self {
            Plots::Plot(..) => 0.0,
            Plots::Subplots(subplots) => subplots.space(),
        }
    }
}

/// An Iterator around a figure's plots, as returned by [`Plots::iter()`].
#[derive(Debug, Clone)]
pub struct PlotIter<'a> {
    plots: &'a Plots,
    idx: PlotIdx,
}

impl<'a> Iterator for PlotIter<'a> {
    type Item = Option<&'a Plot>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.plots {
            Plots::Plot(plot) => {
                if self.idx.is_first() {
                    self.idx = self.idx.next(1);
                    Some(Some(plot))
                } else {
                    None
                }
            }
            Plots::Subplots(subplots) => {
                if self.idx.row < subplots.rows() {
                    let plot = subplots.plot(self.idx);
                    self.idx = self.idx.next(subplots.cols());
                    Some(plot)
                } else {
                    None
                }
            }
        }
    }
}

impl FusedIterator for PlotIter<'_> {}
