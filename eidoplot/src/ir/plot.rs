use crate::ir::{Axis, Legend};
use crate::style;
use crate::style::color;

#[derive(Debug, Clone, Copy)]
pub enum Border {
    Box(style::Line),
    Axis(style::Line),
    AxisArrow {
        stroke: style::Line,
        size: f32,
        overflow: f32,
    },
}

impl Default for Border {
    fn default() -> Self {
        Border::Box(color::BLACK.into())
    }
}

/// Insets inside the plot area
/// around the data.
#[derive(Debug, Default, Clone, Copy)]
pub enum Insets {
    /// The insets depends on the style of series
    #[default]
    Auto,
    Fixed(f32, f32),
}

#[derive(Debug, Clone)]
pub struct Plot {
    pub title: Option<String>,
    pub fill: Option<style::Fill>,
    pub border: Option<Border>,
    pub insets: Option<Insets>,
    pub legend: Option<Legend>,
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub series: Vec<Series>,
}

impl Default for Plot {
    fn default() -> Self {
        Plot {
            title: None,
            fill: None,
            border: Some(Border::default()),
            insets: Some(Insets::default()),
            legend: Some(Legend::default()),
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            series: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Series {
    pub name: Option<String>,
    pub plot: SeriesPlot,
}

#[derive(Debug, Clone)]
pub enum SeriesPlot {
    /// Plots data in XY space.
    Xy(XySeries),
}

#[derive(Debug, Clone)]
pub struct XySeries {
    pub line: style::Line,
    pub points: Vec<(f64, f64)>,
}
