pub mod axis;
pub mod style;
pub mod prelude;

pub struct FigSize {
    pub w: f32,
    pub h: f32,
}

impl Default for FigSize {
    fn default() -> Self {
        FigSize { w: 800.0, h: 600.0 }
    }
}

pub struct Figure {
    pub size: FigSize,
    pub title: Option<String>,
    pub plots: Plots,
}

pub enum Plots {
    Plot(Plot),
    Subplots {
        rows: u32,
        cols: u32,
        plots: Vec<Plot>,
    }
}

pub struct Plot {
    pub title: Option<String>,
    pub desc: PlotDesc,
}

pub enum PlotDesc {
    Curves(Curves),
}

pub struct Curves {
    pub x_axis: axis::Axis,
    pub y_axis: axis::Axis,
    pub series: Vec<XySeries>,
}

pub struct XySeries {
    pub name: String,
    pub line_style: style::Line,
    pub points: Vec<(f64, f64)>,
}
