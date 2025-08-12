use crate::geom;
use crate::ir::{Legend, Plot, Text};
use crate::style::{self, color, defaults};

#[derive(Debug, Clone)]
pub struct Figure {
    size: geom::Size,
    title: Option<Text>,
    plots: Plots,
    legend: Option<Legend>,
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

    pub fn with_title(self, title: Option<Text>) -> Self {
        Figure {
            title: title,
            ..self
        }
    }

    pub fn with_legend(self, legend: Option<Legend>) -> Self {
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

    pub fn title(&self) -> Option<&Text> {
        self.title.as_ref()
    }

    pub fn plots(&self) -> &Plots {
        &self.plots
    }

    pub fn legend(&self) -> Option<&Legend> {
        self.legend.as_ref()
    }

    pub fn fill(&self) -> Option<style::Fill> {
        self.fill
    }

    pub fn layout(&self) -> Option<&Layout> {
        self.layout.as_ref()
    }
}

#[derive(Debug, Clone)]
pub enum Plots {
    Plot(Plot),
    Subplots(Subplots),
}

#[derive(Debug, Clone)]
pub struct Subplots {
    pub rows: u32,
    pub cols: u32,
    pub space: f32,
    pub plots: Vec<Plot>,
}

#[derive(Debug, Clone)]
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
