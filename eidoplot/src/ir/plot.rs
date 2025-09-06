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

impl Default for PlotLegend {
    fn default() -> Self {
        PlotLegend {
            pos: LegendPos::default(),
            legend: Legend::default(),
            margin: defaults::LEGEND_MARGIN,
        }
    }
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


impl From<LegendPos> for PlotLegend {
    fn from(pos: LegendPos) -> Self {
        PlotLegend {
            pos,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Plot {
    series: Vec<Series>,

    x_axis: Axis,
    y_axis: Axis,
    title: Option<String>,
    fill: Option<theme::Fill>,
    border: Option<Border>,
    insets: Option<Insets>,
    legend: Option<PlotLegend>,
}

impl Plot {
    pub fn new(series: Vec<Series>) -> Self {
        Plot {
            series,
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            title: None,
            fill: None,
            border: Some(Border::default()),
            insets: Some(Insets::default()),
            legend: None,
        }
    }

    pub fn with_x_axis(self, x_axis: Axis) -> Self {
        Self { x_axis, ..self }
    }

    pub fn with_y_axis(self, y_axis: Axis) -> Self {
        Self { y_axis, ..self }
    }
    
    pub fn with_title(self, title: String) -> Self {
        Self {
            title: Some(title),
            ..self
        }
    }

    pub fn with_fill(self, fill: theme::Fill) -> Self {
        Self {
            fill: Some(fill),
            ..self
        }
    }

    pub fn with_border(self, border: Option<Border>) -> Self {
        Self { border, ..self }
    }

    pub fn with_insets(self, insets: Option<Insets>) -> Self {
        Self { insets, ..self }
    }

    pub fn with_legend(self, legend: PlotLegend) -> Self {
        Self { legend: Some(legend), ..self }
    }

    pub fn series(&self) -> &[Series] {
        &self.series
    }

    pub fn x_axis(&self) -> &Axis {
        &self.x_axis
    }

    pub fn y_axis(&self) -> &Axis {
        &self.y_axis
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn fill(&self) -> Option<&theme::Fill> {
        self.fill.as_ref()
    }

    pub fn border(&self) -> Option<&Border> {
        self.border.as_ref()
    }

    pub fn insets(&self) -> Option<&Insets> {
        self.insets.as_ref()
    }

    pub fn legend(&self) -> Option<&PlotLegend> {
        self.legend.as_ref()
    }
}
