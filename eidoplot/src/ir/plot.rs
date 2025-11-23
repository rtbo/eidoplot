use crate::ir::{Axis, Legend, Series, axis};
use crate::style::{self, defaults, theme};

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

/// A line traced on the plot in addition to the series.
/// By default it is plotted under the series, unless `with_above()` is called.
#[derive(Debug, Clone)]
pub struct PlotLine {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) direction: Direction,
    pub(crate) line: theme::Line,
    pub(crate) above: bool,
    pub(crate) x_axis: Option<axis::Ref>,
    pub(crate) y_axis: Option<axis::Ref>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Direction {
    Horizontal,
    Vertical,
    Slope(f32),
    SecondPoint(f64, f64),
}

impl PlotLine {
    /// Plot a vertical line passing by x
    pub fn vertical(x: f64) -> Self {
        PlotLine {
            x,
            y: 0.0,
            direction: Direction::Vertical,
            line: theme::Col::Foreground.into(),
            above: false,
            x_axis: None,
            y_axis: None,
        }
    }

    /// Plot a horizontal line passing by y
    pub fn horizontal(y: f64) -> Self {
        PlotLine {
            x: 0.0,
            y,
            direction: Direction::Horizontal,
            line: theme::Col::Foreground.into(),
            above: false,
            x_axis: None,
            y_axis: None,
        }
    }

    /// Plot a line passing by x and y with the given slope.
    /// This is only meaningful on linear scales, and will raise an error
    /// if either X or Y axes are logarithmic.
    pub fn slope(x: f64, y: f64, slope: f32) -> Self {
        PlotLine {
            x,
            y,
            direction: Direction::Slope(slope),
            line: theme::Col::Foreground.into(),
            above: false,
            x_axis: None,
            y_axis: None,
        }
    }

    /// Plot a line passing by (x1, y1) and (x2, y2).
    pub fn two_points(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        PlotLine {
            x: x1,
            y: y1,
            direction: Direction::SecondPoint(x2, y2),
            line: theme::Col::Foreground.into(),
            above: false,
            x_axis: None,
            y_axis: None,
        }
    }

    /// Set the line to be displayed.
    /// By default, the line is a solid line of the foreground theme color.
    pub fn with_line(self, line: theme::Line) -> Self {
        Self { line, ..self }
    }

    /// Set the pattern of the line
    pub fn with_pattern(self, pattern: style::LinePattern) -> Self {
        Self {
            line: self.line.with_pattern(pattern),
            ..self
        }
    }

    /// Set the line to be displayed above the series.
    pub fn with_above(self) -> Self {
        Self {
            above: true,
            ..self
        }
    }

    /// Set the X-axis to use for this line.
    /// Only useful if multiple X-axes are used.
    /// By default, the first X-axis is used.
    pub fn with_x_axis(self, x_axis: axis::Ref) -> Self {
        Self {
            x_axis: Some(x_axis),
            ..self
        }
    }

    /// Set the Y-axis to use for this line.
    /// Only useful if multiple Y-axes are used.
    /// By default, the first Y-axis is used.
    pub fn with_y_axis(self, y_axis: axis::Ref) -> Self {
        Self {
            y_axis: Some(y_axis),
            ..self
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
    lines: Vec<PlotLine>,
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
            lines: vec![],
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

    pub fn with_line(mut self, line: PlotLine) -> Self {
        self.lines.push(line);
        self
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

    pub fn lines(&self) -> &[PlotLine] {
        &self.lines
    }

    pub fn push_series(&mut self, series: Series) {
        self.series.push(series);
    }

    pub fn push_line(&mut self, line: PlotLine) {
        self.lines.push(line);
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
