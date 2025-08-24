use crate::{data, style};

#[derive(Debug, Clone)]
pub enum DataCol {
    Inline(data::VecColumn),
    SrcRef(String),
}

#[derive(Debug, Clone)]
pub struct Series {
    pub name: Option<String>,
    pub plot: SeriesPlot,
}

#[derive(Debug, Clone)]
pub enum SeriesPlot {
    /// Plots data in XY space.
    Line(Line),
    /// Plots data as scatter points.
    Scatter(Scatter),
    /// Plots data in histograms
    Histogram(Histogram),
}

#[derive(Debug, Clone)]
pub struct Line {
    pub line: style::Line,
    pub x_data: DataCol,
    pub y_data: DataCol,
}

#[derive(Debug, Clone)]
pub struct Scatter {
    pub marker: style::Marker,
    pub x_data: DataCol,
    pub y_data: DataCol,
}

#[derive(Debug, Clone)]
pub struct Histogram {
    pub fill: style::Fill,
    pub line: Option<style::Line>,
    pub bins: u32,
    pub density: bool,
    pub data: DataCol,
}
