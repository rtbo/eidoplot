use crate::style;

#[derive(Debug, Clone)]
pub struct Series {
    pub name: Option<String>,
    pub plot: SeriesPlot,
}

#[derive(Debug, Clone)]
pub enum SeriesPlot {
    /// Plots data in XY space.
    Xy(Xy),
    /// Plots data in histograms
    Histogram(Histogram),
}

#[derive(Debug, Clone)]
pub struct Xy {
    pub line: style::Line,
    pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone)]
pub struct Histogram {
    pub fill: style::Fill,
    pub line: Option<style::Line>,
    pub bins: u32,
    pub density: bool,
    pub points: Vec<f64>,
}
