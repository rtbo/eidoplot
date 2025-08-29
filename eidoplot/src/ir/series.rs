use crate::data;
use crate::style;

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
    /// Plots data as a continuous line.
    Line(Line),
    /// Plots data as scatter points.
    Scatter(Scatter),
    /// Plots data in histograms.
    Histogram(Histogram),
    /// Plots data as discrete bars.
    /// One of the axis must be categories, and the other must be numeric
    Bars(Bars),
}

#[derive(Debug, Clone)]
pub struct Line {
    pub line: style::series::Line,
    pub x_data: DataCol,
    pub y_data: DataCol,
}

#[derive(Debug, Clone)]
pub struct Scatter {
    pub marker: style::series::Marker,
    pub x_data: DataCol,
    pub y_data: DataCol,
}

#[derive(Debug, Clone)]
pub struct Histogram {
    pub fill: style::series::Fill,
    pub line: Option<style::series::Line>,
    pub bins: u32,
    pub density: bool,
    pub data: DataCol,
}

/// Offset and width of the bar, in ratio of the category bin width
/// The default is offset of 0.3, and width of 0.4, which has effect of a bar centered in the bin.
/// (the bar starts at 30% of the bin and ends at 70% of the bin)
/// If multiple series are plotted, this offset and width are to be adjusted, otherwise the bars will overlap.
#[derive(Debug, Clone, Copy)]
pub struct BarPosition {
    pub offset: f32,
    pub width: f32,
}

impl Default for BarPosition {
    fn default() -> Self {
        BarPosition {
            offset: 0.3,
            width: 0.4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bars {
    pub fill: style::series::Fill,
    pub line: Option<style::series::Line>,
    pub position: BarPosition,
    pub x_data: DataCol,
    pub y_data: DataCol,
}
