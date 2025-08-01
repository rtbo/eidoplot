/// Describe the bounds of an axis in data space
pub enum Range {
    Auto,
    MinAuto(f64),
    AutoMax(f64),
    MinMax(f64, f64),
}

/// Describes the type of an axis
pub enum Scale {
    Linear,
    Log,
}

pub enum TickLocator {
    Auto,
    MaxN { num: u32, steps: Vec<f64> },
    Multiple(f64),
    PiMultiple { num: f64, den: f64 },
    Linear(u32),
    Fixed(Vec<f64>),
    Log { base: f64, num: u32 },
}

pub struct Axis {
    pub name: String,
    pub range: Range,
    pub scale: Scale,
    pub ticks: Option<TickLocator>,
    pub ticks_min: Option<TickLocator>,
}
