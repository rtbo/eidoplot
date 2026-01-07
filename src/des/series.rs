//! Data series definitions for plots.
use crate::data;
use crate::des::axis;
use crate::style::{self, defaults};

/// A data column, either inline or a reference to a data source.
///
/// Data columns can contain either inline data (vectors of values) or references
/// to columns in a data source. This allows for flexible data handling in series.
#[derive(Debug, Clone)]
pub enum DataCol {
    /// The data is provided inline, directly in the series
    Inline(data::VecColumn),
    /// The data is a column reference to a data source
    SrcRef(String),
}

/// Build a data source column reference.
///
/// Creates a [`DataCol::SrcRef`] variant from a string-like value.
/// Use this to reference a column in an external data source.
///
/// # Examples
///
/// ```ignore
/// let col = data_src_ref("temperature");
/// ```
pub fn data_src_ref<S: Into<String>>(src: S) -> DataCol {
    DataCol::SrcRef(src.into())
}

/// Build a inline data column.
/// Creates a [`DataCol::Inline`] variant from a vector of values.
/// Use this to provide data directly in the series.
/// Doing this, you may pass `()` as data source when building the plot.
/// # Examples
/// ```ignore
/// let col = data_inline(vec![1.0, 2.0, 3.0]);
/// ```
pub fn data_inline<T: Into<data::VecColumn>>(data: T) -> DataCol {
    DataCol::Inline(data.into())
}

impl From<data::VecColumn> for DataCol {
    fn from(col: data::VecColumn) -> Self {
        DataCol::Inline(col)
    }
}

impl From<Vec<f64>> for DataCol {
    fn from(col: Vec<f64>) -> Self {
        DataCol::Inline(col.into())
    }
}

impl From<Vec<String>> for DataCol {
    fn from(col: Vec<String>) -> Self {
        DataCol::Inline(col.into())
    }
}

/// A data series to be plotted in a plot.
///
/// This enum represents the different types of series that can be visualized.
/// Each variant contains specific configuration and data for that series type.
#[derive(Debug, Clone)]
pub enum Series {
    /// Plots data as a continuous line.
    Line(Line),
    /// Plots data as scatter points.
    Scatter(Scatter),
    /// Plots data in histograms.
    Histogram(Histogram),
    /// Plots data as discrete bars.
    Bars(Bars),
    /// Plots data as a group of bars, that can be either stacked or aside
    BarsGroup(BarsGroup),
}

impl Series {
    /// Get the x and y axis references used by this series
    pub fn axes(&self) -> (&axis::Ref, &axis::Ref) {
        match self {
            Series::Line(s) => (s.x_axis(), s.y_axis()),
            Series::Scatter(s) => (s.x_axis(), s.y_axis()),
            Series::Histogram(s) => (s.x_axis(), s.y_axis()),
            Series::Bars(s) => (s.x_axis(), s.y_axis()),
            Series::BarsGroup(s) => (s.x_axis(), s.y_axis()),
        }
    }
}

impl From<Line> for Series {
    fn from(line: Line) -> Self {
        Series::Line(line)
    }
}

impl From<Scatter> for Series {
    fn from(scatter: Scatter) -> Self {
        Series::Scatter(scatter)
    }
}

impl From<Histogram> for Series {
    fn from(histogram: Histogram) -> Self {
        Series::Histogram(histogram)
    }
}

impl From<Bars> for Series {
    fn from(bars: Bars) -> Self {
        Series::Bars(bars)
    }
}

impl From<BarsGroup> for Series {
    fn from(bars_group: BarsGroup) -> Self {
        Series::BarsGroup(bars_group)
    }
}

/// A line series structure.
///
/// Plots data as a continuous line connecting points in order.
/// This is one of the most common series types for visualizing trends and continuous data.
#[derive(Debug, Clone)]
pub struct Line {
    x_data: DataCol,
    y_data: DataCol,

    name: Option<String>,
    x_axis: axis::Ref,
    y_axis: axis::Ref,
    line: style::series::Line,
}

impl Line {
    /// Create a new line series with the given x and y data columns
    pub fn new(x_data: DataCol, y_data: DataCol) -> Self {
        Line {
            x_data,
            y_data,

            name: None,
            x_axis: Default::default(),
            y_axis: Default::default(),
            line: style::series::Line::default().with_width(defaults::SERIES_LINE_WIDTH),
        }
    }

    /// Set the name and return self for chaining
    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    /// Set a reference to the x axis and return self for chaining
    /// Use this to associate the series with a specific x axis in the plot, when a plot has multiple x axes.
    pub fn with_x_axis(mut self, axis: axis::Ref) -> Self {
        self.x_axis = axis;
        self
    }

    /// Set a reference to the y axis and return self for chaining
    /// Use this to associate the series with a specific y axis in the plot, when a plot has multiple y axes.
    pub fn with_y_axis(mut self, axis: axis::Ref) -> Self {
        self.y_axis = axis;
        self
    }

    /// Set the line style and return self for chaining
    pub fn with_line(mut self, line: style::series::Line) -> Self {
        self.line = line;
        self
    }

    /// Get the x data column
    pub fn x_data(&self) -> &DataCol {
        &self.x_data
    }

    /// Get the y data column
    pub fn y_data(&self) -> &DataCol {
        &self.y_data
    }

    /// Get the name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get a reference to the x axis
    pub fn x_axis(&self) -> &axis::Ref {
        &self.x_axis
    }

    /// Get a reference to the y axis
    pub fn y_axis(&self) -> &axis::Ref {
        &self.y_axis
    }

    /// Get the line style
    pub fn line(&self) -> &style::series::Line {
        &self.line
    }
}

/// A scatter series structure.
///
/// Plots data as individual scatter points without connecting them.
/// Useful for visualizing correlations, distributions, and discrete data points.
#[derive(Debug, Clone)]
pub struct Scatter {
    x_data: DataCol,
    y_data: DataCol,

    name: Option<String>,
    x_axis: axis::Ref,
    y_axis: axis::Ref,
    marker: style::series::Marker,
}

impl Scatter {
    /// Create a new scatter series with the given x and y data columns
    pub fn new(x_data: DataCol, y_data: DataCol) -> Self {
        Scatter {
            x_data,
            y_data,

            name: None,
            x_axis: Default::default(),
            y_axis: Default::default(),
            marker: style::series::Marker::default(),
        }
    }

    /// Set the name and return self for chaining
    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    /// Set a reference to the x axis and return self for chaining
    /// Use this to associate the series with a specific x axis in the plot, when a plot has multiple x axes.
    pub fn with_x_axis(mut self, axis: axis::Ref) -> Self {
        self.x_axis = axis;
        self
    }

    /// Set a reference to the y axis and return self for chaining
    /// Use this to associate the series with a specific y axis in the plot, when a plot has multiple y axes.
    pub fn with_y_axis(mut self, axis: axis::Ref) -> Self {
        self.y_axis = axis;
        self
    }

    /// Set the marker style and return self for chaining
    pub fn with_marker(mut self, marker: style::series::Marker) -> Self {
        self.marker = marker;
        self
    }

    /// Get the x data column
    pub fn x_data(&self) -> &DataCol {
        &self.x_data
    }

    /// Get the y data column
    pub fn y_data(&self) -> &DataCol {
        &self.y_data
    }

    /// Get the name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get a reference to the x axis
    pub fn x_axis(&self) -> &axis::Ref {
        &self.x_axis
    }

    /// Get a reference to the y axis
    pub fn y_axis(&self) -> &axis::Ref {
        &self.y_axis
    }

    /// Get the marker style
    pub fn marker(&self) -> &style::series::Marker {
        &self.marker
    }
}

/// A histogram series structure.
///
/// Plots data by grouping values into bins and showing the frequency or density
/// of values in each bin. Useful for visualizing distributions of continuous data.
#[derive(Debug, Clone)]
pub struct Histogram {
    data: DataCol,

    name: Option<String>,
    x_axis: axis::Ref,
    y_axis: axis::Ref,
    fill: style::series::Fill,
    line: Option<style::series::Line>,
    bins: u32,
    density: bool,
}

impl Histogram {
    /// Create a new histogram series with the given data column
    pub fn new(data: DataCol) -> Self {
        Histogram {
            data,

            name: None,
            x_axis: Default::default(),
            y_axis: Default::default(),
            fill: style::series::Fill::default(),
            line: None,
            bins: 10,
            density: false,
        }
    }

    /// Set the name and return self for chaining
    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    /// Set a reference to the x axis and return self for chaining
    pub fn with_x_axis(mut self, axis: axis::Ref) -> Self {
        self.x_axis = axis;
        self
    }

    /// Set a reference to the y axis and return self for chaining
    pub fn with_y_axis(mut self, axis: axis::Ref) -> Self {
        self.y_axis = axis;
        self
    }

    /// Set the fill style and return self for chaining
    pub fn with_fill(self, fill: style::series::Fill) -> Self {
        Self { fill, ..self }
    }

    /// Set the line style for the histogram outline and return self for chaining
    pub fn with_line(mut self, line: style::series::Line) -> Self {
        self.line = Some(line);
        self
    }

    /// Set the number of bins and return self for chaining
    pub fn with_bins(mut self, bins: u32) -> Self {
        self.bins = bins;
        self
    }

    /// Enable density mode (normalize by total count) and return self for chaining
    pub fn with_density(mut self) -> Self {
        self.density = true;
        self
    }

    /// Get the data column
    pub fn data(&self) -> &DataCol {
        &self.data
    }

    /// Get the name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get a reference to the x axis, if any
    pub fn x_axis(&self) -> &axis::Ref {
        &self.x_axis
    }

    /// Get a reference to the y axis
    pub fn y_axis(&self) -> &axis::Ref {
        &self.y_axis
    }

    /// Get the fill style
    pub fn fill(&self) -> &style::series::Fill {
        &self.fill
    }

    /// Get the line style, if any
    pub fn line(&self) -> Option<&style::series::Line> {
        self.line.as_ref()
    }

    /// Get the number of bins
    pub fn bins(&self) -> u32 {
        self.bins
    }

    /// Get whether density mode is enabled
    pub fn density(&self) -> bool {
        self.density
    }
}

/// Offset and width of the bar, in ratio of the category bin width.
///
/// The default is offset of 0.3, and width of 0.4, which has the effect of a bar centered in the bin
/// (the bar starts at 30% of the bin and ends at 70% of the bin).
///
/// If multiple series are plotted, this offset and width should be adjusted, otherwise the bars will overlap.
#[derive(Debug, Clone, Copy)]
pub struct BarsPosition {
    /// Offset from the start of the category bin (0.0 to 1.0).
    pub offset: f32,
    /// Width of the bar as a ratio of the bin width (0.0 to 1.0).
    pub width: f32,
}

impl Default for BarsPosition {
    fn default() -> Self {
        BarsPosition {
            offset: 0.3,
            width: 0.4,
        }
    }
}

/// A bars series structure.
///
/// Plots data as discrete bars. One axis must contain categories, and the other must be numeric.
/// Each category gets one bar whose height (or length for horizontal bars) represents the data value.
#[derive(Debug, Clone)]
pub struct Bars {
    x_data: DataCol,
    y_data: DataCol,

    name: Option<String>,
    x_axis: axis::Ref,
    y_axis: axis::Ref,
    fill: style::series::Fill,
    line: Option<style::series::Line>,
    position: BarsPosition,
}

impl Bars {
    /// Create a new bars series with the given x and y data columns
    pub fn new(x_data: DataCol, y_data: DataCol) -> Self {
        Bars {
            x_data,
            y_data,

            name: None,
            x_axis: Default::default(),
            y_axis: Default::default(),
            fill: style::series::Fill::default(),
            line: None,
            position: BarsPosition::default(),
        }
    }

    /// Set the name and return self for chaining
    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    /// Set the fill style and return self for chaining
    pub fn with_fill(self, fill: style::series::Fill) -> Self {
        Self { fill, ..self }
    }

    /// Set the line style for the bar outline and return self for chaining
    pub fn with_line(self, line: style::series::Line) -> Self {
        Self {
            line: Some(line),
            ..self
        }
    }

    /// Set the position (offset and width) and return self for chaining
    pub fn with_position(self, position: BarsPosition) -> Self {
        Self { position, ..self }
    }

    /// Get the x data column
    pub fn x_data(&self) -> &DataCol {
        &self.x_data
    }

    /// Get the y data column
    pub fn y_data(&self) -> &DataCol {
        &self.y_data
    }

    /// Get the name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get a reference to the x axis
    pub fn x_axis(&self) -> &axis::Ref {
        &self.x_axis
    }

    /// Get a reference to the y axis
    pub fn y_axis(&self) -> &axis::Ref {
        &self.y_axis
    }

    /// Get the fill style
    pub fn fill(&self) -> &style::series::Fill {
        &self.fill
    }

    /// Get the line style, if any
    pub fn line(&self) -> Option<&style::series::Line> {
        self.line.as_ref()
    }

    /// Get the position configuration
    pub fn position(&self) -> &BarsPosition {
        &self.position
    }
}

/// A bar series within a bars group.
///
/// Represents a single series of bars within a [`BarsGroup`].
/// Each `BarSeries` contains data for one set of bars across all categories.
#[derive(Debug, Clone)]
pub struct BarSeries {
    data: DataCol,

    name: Option<String>,
    fill: style::series::Fill,
    line: Option<style::series::Line>,
}

impl BarSeries {
    /// Create a new bar series with the given data column
    pub fn new(data: DataCol) -> Self {
        BarSeries {
            data,

            name: None,
            fill: style::series::Fill::default(),
            line: None,
        }
    }

    /// Set the name and return self for chaining
    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    /// Set the fill style and return self for chaining
    pub fn with_fill(self, fill: style::series::Fill) -> Self {
        Self { fill, ..self }
    }

    /// Set the line style for the bar outline and return self for chaining
    pub fn with_line(self, line: style::series::Line) -> Self {
        Self {
            line: Some(line),
            ..self
        }
    }

    /// Get the data column
    pub fn data(&self) -> &DataCol {
        &self.data
    }

    /// Get the name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get the fill style
    pub fn fill(&self) -> &style::series::Fill {
        &self.fill
    }

    /// Get the line style, if any
    pub fn line(&self) -> Option<&style::series::Line> {
        self.line.as_ref()
    }
}

/// Orientation of bars in a bar chart.
///
/// Determines whether bars extend vertically (from the x-axis) or horizontally (from the y-axis).
#[derive(Debug, Clone, Copy, Default)]
pub enum BarsOrientation {
    /// Bars extend vertically from the x-axis.
    #[default]
    Vertical,
    /// Bars extend horizontally from the y-axis.
    Horizontal,
}

impl BarsOrientation {
    /// Check if the orientation is vertical
    pub fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }

    /// Check if the orientation is horizontal
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }
}

/// Arrangement of multiple bar series within a group.
///
/// Defines how multiple bar series are positioned relative to each other:
/// either side-by-side or stacked on top of each other.
#[derive(Debug, Clone, Copy)]
pub enum BarsArrangement {
    /// Bars are placed side-by-side within each category.
    Aside(BarsAsideArrangement),
    /// Bars are stacked on top of each other within each category.
    Stack(BarsStackArrangement),
}

/// Configuration for side-by-side bar arrangement.
///
/// Specifies how bars are positioned when placed side-by-side within each category.
#[derive(Debug, Clone, Copy)]
pub struct BarsAsideArrangement {
    /// Offset of the first bar within the bin (0.0 to 1.0).
    pub offset: f32,
    /// Width of the whole group within the bin (0.0 to 1.0).
    pub width: f32,
    /// Gap between adjacent bars as a ratio of the available space.
    pub gap: f32,
}

impl Default for BarsAsideArrangement {
    fn default() -> Self {
        BarsAsideArrangement {
            offset: 0.15,
            width: 0.7,
            gap: 0.04,
        }
    }
}

/// Configuration for stacked bar arrangement.
///
/// Specifies how bars are positioned when stacked on top of each other within each category.
#[derive(Debug, Clone, Copy)]
pub struct BarsStackArrangement {
    /// Offset of the stacked bars within the bin (0.0 to 1.0).
    pub offset: f32,
    /// Width of the stacked bars within the bin (0.0 to 1.0).
    pub width: f32,
}

impl Default for BarsStackArrangement {
    fn default() -> Self {
        BarsStackArrangement {
            offset: 0.22,
            width: 0.56,
        }
    }
}

impl Default for BarsArrangement {
    fn default() -> Self {
        BarsArrangement::Aside(Default::default())
    }
}

/// A group of bar series.
///
/// Represents multiple bar series that share the same categories.
/// The bars can be arranged either side-by-side or stacked, and can be oriented
/// vertically or horizontally.
#[derive(Debug, Clone)]
pub struct BarsGroup {
    categories: DataCol,
    series: Vec<BarSeries>,

    x_axis: axis::Ref,
    y_axis: axis::Ref,
    orientation: BarsOrientation,
    arrangement: BarsArrangement,
}

impl BarsGroup {
    /// Create a new bars group with the given categories and bar series
    pub fn new(categories: DataCol, series: Vec<BarSeries>) -> Self {
        BarsGroup {
            categories,
            series,
            x_axis: Default::default(),
            y_axis: Default::default(),
            orientation: Default::default(),
            arrangement: Default::default(),
        }
    }

    /// Set the orientation and return self for chaining
    pub fn with_orientation(self, orientation: BarsOrientation) -> Self {
        Self {
            orientation,
            ..self
        }
    }

    /// Set the arrangement and return self for chaining
    pub fn with_arrangement(self, arrangement: BarsArrangement) -> Self {
        Self {
            arrangement,
            ..self
        }
    }

    /// Get the categories data column
    pub fn categories(&self) -> &DataCol {
        &self.categories
    }

    /// Get the bar series
    pub fn series(&self) -> &[BarSeries] {
        &self.series
    }

    /// Get a reference to the x axis
    pub fn x_axis(&self) -> &axis::Ref {
        &self.x_axis
    }

    /// Get a reference to the y axis
    pub fn y_axis(&self) -> &axis::Ref {
        &self.y_axis
    }

    /// Get the orientation
    pub fn orientation(&self) -> &BarsOrientation {
        &self.orientation
    }

    /// Get the arrangement
    pub fn arrangement(&self) -> &BarsArrangement {
        &self.arrangement
    }
}
