use std::iter::FusedIterator;

use crate::geom;
use crate::ir::{Plot, Legend};
use crate::style::{self, color, defaults};

#[derive(Debug, Clone)]
pub struct TitleFont {
    pub font: style::Font,
    pub size: f32,
}

impl Default for TitleFont {
    fn default() -> Self {
        TitleFont {
            font: defaults::TITLE_FONT_FAMILY.parse().unwrap(),
            size: defaults::TITLE_FONT_SIZE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Title {
    pub text: String,
    pub font: TitleFont,
}

impl<S: Into<String>> From<S> for Title {
    fn from(text: S) -> Self {
        Title {
            text: text.into(),
            font: TitleFont::default(),
        }
    }
}

/// Position of the legend relatively to the figure
#[derive(Debug, Default, Clone, Copy)]
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
        FigLegend { pos, legend, margin: defaults::LEGEND_MARGIN }
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
    size: geom::Size,
    title: Option<Title>,
    plots: Plots,
    legend: Option<FigLegend>,
    fill: Option<style::Fill>,
    layout: Option<Layout>,
}

impl Figure {
    pub fn new(plots: Plots) -> Figure {
        Figure {
            size: defaults::FIG_SIZE,
            title: None,
            plots,
            legend: None,
            fill: Some(color::WHITE.into()),
            layout: None,
        }
    }

    pub fn with_size(self, size: geom::Size) -> Self {
        Figure { size: size, ..self }
    }

    pub fn with_title(self, title: Option<Title>) -> Self {
        Figure {
            title: title,
            ..self
        }
    }

    pub fn with_legend(self, legend: Option<FigLegend>) -> Self {
        Figure { legend, ..self }
    }

    pub fn with_fill(self, fill: Option<style::Fill>) -> Self {
        Figure { fill, ..self }
    }

    pub fn with_layout(self, layout: Option<Layout>) -> Self {
        Figure { layout, ..self }
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

    pub fn legend(&self) -> Option<&FigLegend> {
        self.legend.as_ref()
    }

    pub fn fill(&self) -> Option<style::Fill> {
        self.fill
    }

    pub fn layout(&self) -> Option<&Layout> {
        self.layout.as_ref()
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

#[derive(Debug, Clone)]
pub struct Subplots {
    pub rows: u32,
    pub cols: u32,
    pub space: f32,
    pub plots: Vec<Plot>,
}

impl Plots {
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
                if self.index < subplots.plots.len() {
                    let plot = &subplots.plots[self.index];
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

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    padding: Option<geom::Padding>,
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            padding: Some(defaults::FIG_PADDING),
        }
    }
}

impl Layout {
    pub fn new() -> Self {
        Layout { padding: None }
    }

    pub fn with_padding(self, padding: Option<geom::Padding>) -> Self {
        Self {
            padding: padding,
            ..self
        }
    }

    pub fn padding(&self) -> Option<&geom::Padding> {
        self.padding.as_ref()
    }
}
