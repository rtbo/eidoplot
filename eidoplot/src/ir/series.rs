use crate::{
    data,
    style::{self, defaults},
};

#[derive(Debug, Clone)]
pub enum DataCol {
    Inline(data::VecColumn),
    SrcRef(String),
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

#[derive(Debug, Clone)]
pub struct Line {
    x_data: DataCol,
    y_data: DataCol,

    name: Option<String>,
    line: style::series::Line,
}

impl Line {
    pub fn new(x_data: DataCol, y_data: DataCol) -> Self {
        Line {
            x_data,
            y_data,

            name: None,
            line: style::series::Line::default().with_width(defaults::SERIES_LINE_WIDTH),
        }
    }

    pub fn with_name(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    pub fn with_line(mut self, line: style::series::Line) -> Self {
        self.line = line;
        self
    }

    pub fn x_data(&self) -> &DataCol {
        &self.x_data
    }

    pub fn y_data(&self) -> &DataCol {
        &self.y_data
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn line(&self) -> &style::series::Line {
        &self.line
    }
}

#[derive(Debug, Clone)]
pub struct Scatter {
    x_data: DataCol,
    y_data: DataCol,

    name: Option<String>,
    marker: style::series::Marker,
}

impl Scatter {
    pub fn new(x_data: DataCol, y_data: DataCol) -> Self {
        Scatter {
            x_data,
            y_data,

            name: None,
            marker: style::series::Marker::default(),
        }
    }

    pub fn with_name(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    pub fn with_marker(mut self, marker: style::series::Marker) -> Self {
        self.marker = marker;
        self
    }

    pub fn x_data(&self) -> &DataCol {
        &self.x_data
    }

    pub fn y_data(&self) -> &DataCol {
        &self.y_data
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn marker(&self) -> &style::series::Marker {
        &self.marker
    }
}

#[derive(Debug, Clone)]
pub struct Histogram {
    data: DataCol,

    name: Option<String>,
    fill: style::series::Fill,
    line: Option<style::series::Line>,
    bins: u32,
    density: bool,
}

impl Histogram {
    pub fn new(data: DataCol) -> Self {
        Histogram {
            data,

            name: None,
            fill: style::series::Fill::default(),
            line: None,
            bins: 10,
            density: false,
        }
    }

    pub fn with_name(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    pub fn with_fill(self, fill: style::series::Fill) -> Self {
        Self { fill, ..self }
    }

    pub fn with_line(mut self, line: style::series::Line) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_bins(mut self, bins: u32) -> Self {
        self.bins = bins;
        self
    }

    pub fn with_density(mut self) -> Self {
        self.density = true;
        self
    }

    pub fn data(&self) -> &DataCol {
        &self.data
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn fill(&self) -> &style::series::Fill {
        &self.fill
    }

    pub fn line(&self) -> Option<&style::series::Line> {
        self.line.as_ref()
    }

    pub fn bins(&self) -> u32 {
        self.bins
    }

    pub fn density(&self) -> bool {
        self.density
    }
}

/// Offset and width of the bar, in ratio of the category bin width
/// The default is offset of 0.3, and width of 0.4, which has effect of a bar centered in the bin.
/// (the bar starts at 30% of the bin and ends at 70% of the bin)
/// If multiple series are plotted, this offset and width are to be adjusted, otherwise the bars will overlap.
#[derive(Debug, Clone, Copy)]
pub struct BarsPosition {
    pub offset: f32,
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

/// The structure for [`SeriesPlot::Bars`]
/// One of the axis must be categories, and the other must be numeric
#[derive(Debug, Clone)]
pub struct Bars {
    x_data: DataCol,
    y_data: DataCol,

    name: Option<String>,
    fill: style::series::Fill,
    line: Option<style::series::Line>,
    position: BarsPosition,
}

impl Bars {
    pub fn new(x_data: DataCol, y_data: DataCol) -> Self {
        Bars {
            x_data,
            y_data,

            name: None,
            fill: style::series::Fill::default(),
            line: None,
            position: BarsPosition::default(),
        }
    }

    pub fn with_name(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    pub fn with_fill(self, fill: style::series::Fill) -> Self {
        Self { fill, ..self }
    }

    pub fn with_line(self, line: style::series::Line) -> Self {
        Self {
            line: Some(line),
            ..self
        }
    }

    pub fn with_position(self, position: BarsPosition) -> Self {
        Self { position, ..self }
    }

    pub fn x_data(&self) -> &DataCol {
        &self.x_data
    }

    pub fn y_data(&self) -> &DataCol {
        &self.y_data
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn fill(&self) -> &style::series::Fill {
        &self.fill
    }

    pub fn line(&self) -> Option<&style::series::Line> {
        self.line.as_ref()
    }

    pub fn position(&self) -> &BarsPosition {
        &self.position
    }
}

/// The series structure for [`SeriesPlot::Bars`] and [`SeriesPlot::BarsGroup`]
#[derive(Debug, Clone)]
pub struct BarSeries {
    data: DataCol,

    name: Option<String>,
    fill: style::series::Fill,
    line: Option<style::series::Line>,
}

impl BarSeries {
    pub fn new(data: DataCol) -> Self {
        BarSeries {
            data,

            name: None,
            fill: style::series::Fill::default(),
            line: None,
        }
    }

    pub fn with_name(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    pub fn with_fill(self, fill: style::series::Fill) -> Self {
        Self { fill, ..self }
    }

    pub fn with_line(self, line: style::series::Line) -> Self {
        Self {
            line: Some(line),
            ..self
        }
    }

    pub fn data(&self) -> &DataCol {
        &self.data
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn fill(&self) -> &style::series::Fill {
        &self.fill
    }

    pub fn line(&self) -> Option<&style::series::Line> {
        self.line.as_ref()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum BarsOrientation {
    #[default]
    Vertical,
    Horizontal,
}

impl BarsOrientation {
    pub fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BarsArrangement {
    Aside(BarsAsideArrangement),
    Stack(BarsStackArrangement),
}

#[derive(Debug, Clone, Copy)]
pub struct BarsAsideArrangement {
    /// offset of the first bar within the bin
    pub offset: f32,
    /// width of the whole group within the bin
    pub width: f32,
    /// gap between the bars
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

#[derive(Debug, Clone, Copy)]
pub struct BarsStackArrangement {
    /// offset of the first bar within the bin
    pub offset: f32,
    /// width of the whole group within the bin
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

#[derive(Debug, Clone)]
pub struct BarsGroup {
    categories: DataCol,
    series: Vec<BarSeries>,

    orientation: BarsOrientation,
    arrangement: BarsArrangement,
}

impl BarsGroup {
    pub fn new(categories: DataCol, series: Vec<BarSeries>) -> Self {
        BarsGroup {
            categories,
            series,
            orientation: Default::default(),
            arrangement: Default::default(),
        }
    }

    pub fn with_orientation(self, orientation: BarsOrientation) -> Self {
        Self {
            orientation,
            ..self
        }
    }

    pub fn with_arrangement(self, arrangement: BarsArrangement) -> Self {
        Self {
            arrangement,
            ..self
        }
    }

    pub fn categories(&self) -> &DataCol {
        &self.categories
    }

    pub fn series(&self) -> &[BarSeries] {
        &self.series
    }

    pub fn orientation(&self) -> &BarsOrientation {
        &self.orientation
    }

    pub fn arrangement(&self) -> &BarsArrangement {
        &self.arrangement
    }
}
