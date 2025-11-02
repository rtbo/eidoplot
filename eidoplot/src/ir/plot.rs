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

    pub fn with_pos(self, pos: LegendPos) -> Self {
        Self { pos, ..self }
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

    x_axes: Vec<Axis>,
    y_axes: Vec<Axis>,
    x_axis_set: bool,
    y_axis_set: bool,
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
            x_axes: vec![Axis::default()],
            y_axes: vec![Axis::default()],
            x_axis_set: false,
            y_axis_set: false,
            title: None,
            fill: None,
            border: Some(Border::default()),
            insets: Some(Insets::default()),
            legend: None,
        }
    }

    pub fn with_x_axis(mut self, x_axis: Axis) -> Self {
        if !self.x_axis_set {
            self.x_axes.clear();
            self.x_axis_set = true;
        }
        self.x_axes.push(x_axis);
        self
    }

    pub fn with_y_axis(mut self, y_axis: Axis) -> Self {
        if !self.y_axis_set {
            self.y_axes.clear();
            self.y_axis_set = true;
        }
        self.y_axes.push(y_axis);
        self
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
        Self {
            legend: Some(legend),
            ..self
        }
    }

    pub fn series(&self) -> &[Series] {
        &self.series
    }

    pub fn x_axes(&self) -> &[Axis] {
        &self.x_axes
    }

    pub fn y_axes(&self) -> &[Axis] {
        &self.y_axes
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

    pub fn push_series(&mut self, series: Series) {
        self.series.push(series);
    }
}

/// A collection of plots, arranged in a grid
#[derive(Debug, Clone)]
pub struct Subplots {
    rows: u32,
    cols: u32,
    plots: Vec<Option<Plot>>,
    space: f32,
}

impl Subplots {
    pub fn new(rows: u32, cols: u32) -> Self {
        Subplots {
            rows,
            cols,
            plots: vec![None; (rows * cols) as usize],
            space: 0.0,
        }
    }

    pub fn with_plot(mut self, row: u32, col: u32, plot: Plot) -> Self {
        let index = self.index(row, col);
        self.plots[index] = Some(plot);
        self
    }

    pub fn with_space(self, space: f32) -> Self {
        Self { space, ..self }
    }

    pub fn plot(&self, row: u32, col: u32) -> Option<&Plot> {
        self.plots[self.index(row, col)].as_ref()
    }

    pub fn plot_mut(&mut self, row: u32, col: u32) -> Option<&mut Plot> {
        let index = self.index(row, col);
        self.plots[index].as_mut()
    }

    pub fn len(&self) -> usize {
        self.plots.len()
    }

    pub fn rows(&self) -> u32 {
        self.rows
    }

    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn space(&self) -> f32 {
        self.space
    }

    fn index(&self, row: u32, col: u32) -> usize {
        (row * self.cols + col) as usize
    }
}
