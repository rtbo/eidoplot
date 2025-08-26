use crate::ir::{Axis, Legend, Series};
use crate::style::{defaults, theme};

#[derive(Debug, Clone)]
pub enum Border {
    Box(theme::Line),
    Axis(theme::Line),
    AxisArrow {
        stroke: theme::Line,
        size: f32,
        overflow: f32,
    },
}

impl Default for Border {
    fn default() -> Self {
        Border::Box(theme::Col::Foreground.into())
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

/// Position of the legend relatively to the plot
#[derive(Debug, Default, Clone, Copy)]
pub enum LegendPos {
    OutTop,
    OutRight,
    #[default]
    OutBottom,
    OutLeft,
    InTop,
    InTopRight,
    InRight,
    InBottomRight,
    InBottom,
    InBottomLeft,
    InLeft,
    InTopLeft,
}

impl LegendPos {
    /// Whether the legend is placed inside or outside the plot area
    pub fn is_inside(&self) -> bool {
        matches!(
            self,
            LegendPos::InTop
                | LegendPos::InTopRight
                | LegendPos::InRight
                | LegendPos::InBottomRight
                | LegendPos::InBottom
                | LegendPos::InBottomLeft
                | LegendPos::InLeft
                | LegendPos::InTopLeft
        )
    }

    /// Whether the position prefers vertical layout if no amount of column is defined
    pub fn prefers_vertical(&self) -> bool {
        self.is_inside() || matches!(self, LegendPos::OutLeft | LegendPos::OutRight)
    }
}

/// A per-plot legend
#[derive(Debug, Clone)]
pub struct PlotLegend {
    pos: LegendPos,
    legend: Legend,
    margin: f32,
}

impl PlotLegend {
    /// Build a new legend
    pub fn new(pos: LegendPos, legend: Legend) -> Self {
        PlotLegend {
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

    pub fn margin(&self) -> f32 {
        self.margin
    }

    pub fn with_margin(self, margin: f32) -> Self {
        Self { margin, ..self }
    }
}

impl Default for PlotLegend {
    fn default() -> Self {
        PlotLegend::new(LegendPos::default(), Legend::default())
    }
}

impl From<LegendPos> for PlotLegend {
    fn from(pos: LegendPos) -> Self {
        PlotLegend::new(pos, Legend::default())
    }
}

#[derive(Debug, Clone)]
pub struct Plot {
    pub title: Option<String>,
    pub fill: Option<theme::Fill>,
    pub border: Option<Border>,
    pub insets: Option<Insets>,
    pub legend: Option<PlotLegend>,
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
            legend: Some(PlotLegend::default()),
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            series: vec![],
        }
    }
}
