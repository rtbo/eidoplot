use crate::data;
use crate::style;

#[derive(Debug, Clone)]
pub enum DataCol {
    Inline(data::VecColumn),
    SrcRef(String),
}

pub trait SeriesTrait {
    fn name(&self) -> Option<&str>;
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

#[derive(Debug, Clone)]
pub struct Line {
    pub name: Option<String>,
    pub line: style::series::Line,
    pub x_data: DataCol,
    pub y_data: DataCol,
}

impl SeriesTrait for Line {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct Scatter {
    pub name: Option<String>,
    pub marker: style::series::Marker,
    pub x_data: DataCol,
    pub y_data: DataCol,
}

impl SeriesTrait for Scatter {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct Histogram {
    pub name: Option<String>,
    pub fill: style::series::Fill,
    pub line: Option<style::series::Line>,
    pub bins: u32,
    pub density: bool,
    pub data: DataCol,
}

impl SeriesTrait for Histogram {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
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

/// The structure for [`SeriesPlot::Bars`]
/// One of the axis must be categories, and the other must be numeric
#[derive(Debug, Clone)]
pub struct Bars {
    pub name: Option<String>,
    pub fill: style::series::Fill,
    pub line: Option<style::series::Line>,
    pub position: BarPosition,
    pub x_data: DataCol,
    pub y_data: DataCol,
}

impl SeriesTrait for Bars {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// The series structure for [`SeriesPlot::Bars`] and [`SeriesPlot::BarsGroup`]
#[derive(Debug, Clone)]
pub struct BarSeries {
    pub name: Option<String>,
    pub fill: style::series::Fill,
    pub line: Option<style::series::Line>,
    pub data: DataCol,
}

impl SeriesTrait for BarSeries {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum BarsOrientation {
    #[default]
    Vertical,
    Horizontal,
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
    pub categories: DataCol,
    pub orientation: BarsOrientation,
    pub arrangement: BarsArrangement,
    pub series: Vec<BarSeries>,
}
