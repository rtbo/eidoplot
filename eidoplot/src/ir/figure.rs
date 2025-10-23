use std::iter::FusedIterator;
use std::slice;

use crate::geom;
use crate::ir::{Legend, Plot, Subplots};
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
    Top,
    Right,
    #[default]
    Bottom,
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
    pub fn new(pos: LegendPos, legend: Legend) -> Self {
        FigLegend {
            pos,
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

    /// Set the margin around the legend
    pub fn with_margin(self, margin: f32) -> Self {
        Self { margin, ..self }
    }
}

impl Default for FigLegend {
    fn default() -> Self {
        FigLegend::new(LegendPos::default(), Legend::default())
    }
}

impl From<LegendPos> for FigLegend {
    fn from(pos: LegendPos) -> Self {
        FigLegend::new(pos, Legend::default())
    }
}

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

    pub fn with_title(self, title: Title) -> Self {
        Figure {
            title: Some(title),
            ..self
        }
    }

    pub fn with_size(self, size: geom::Size) -> Self {
        Figure { size: size, ..self }
    }

    pub fn with_legend(self, legend: FigLegend) -> Self {
        Figure {
            legend: Some(legend),
            ..self
        }
    }

    pub fn with_fill(self, fill: Option<theme::Fill>) -> Self {
        Figure { fill, ..self }
    }

    pub fn with_padding(self, padding: geom::Padding) -> Self {
        Figure { padding, ..self }
    }

    pub fn size(&self) -> geom::Size {
        self.size
    }

    pub fn title(&self) -> Option<&Title> {
        self.title.as_ref()
    }

    pub fn plots(&self) -> &Plots {
        &self.plots
    }

    pub fn plots_mut(&mut self) -> &mut Plots {
        &mut self.plots
    }

    pub fn legend(&self) -> Option<&FigLegend> {
        self.legend.as_ref()
    }

    pub fn fill(&self) -> Option<theme::Fill> {
        self.fill
    }

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
    pub fn plots(&self) -> &[Plot] {
        match self {
            Plots::Plot(plot) => slice::from_ref(plot),
            Plots::Subplots(subplots) => subplots.plots(),
        }
    }

    pub fn plots_mut(&mut self) -> &mut [Plot] {
        match self {
            Plots::Plot(plot) => slice::from_mut(plot),
            Plots::Subplots(subplots) => subplots.plots_mut(),
        }
    }

    pub fn iter(&self) -> PlotIter<'_> {
        PlotIter {
            plots: self,
            index: 0,
        }
    }
}

/// An Iterator around a figure's plots, as returned by [`Plots::iter()`].
#[derive(Debug, Clone)]
pub struct PlotIter<'a> {
    plots: &'a Plots,
    index: usize,
}

impl<'a> Iterator for PlotIter<'a> {
    type Item = &'a Plot;

    fn next(&mut self) -> Option<Self::Item> {
        match self.plots {
            Plots::Plot(plot) => {
                if self.index == 0 {
                    self.index += 1;
                    Some(plot)
                } else {
                    None
                }
            }
            Plots::Subplots(subplots) => {
                if self.index < subplots.plots().len() {
                    let plot = &subplots.plots()[self.index];
                    self.index += 1;
                    Some(plot)
                } else {
                    None
                }
            }
        }
    }
}

impl FusedIterator for PlotIter<'_> {}
