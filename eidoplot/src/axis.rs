/// Describe the bounds of an axis in data space
#[derive(Debug, Clone, Copy)]
pub enum Range {
    Auto,
    MinAuto(f64),
    AutoMax(f64),
    MinMax(f64, f64),
}

/// Describes the type of an axis
#[derive(Debug, Clone, Copy)]
pub enum Scale {
    Linear,
    Log,
}

#[derive(Debug, Clone)]
pub enum TickLocator {
    Auto,
    MaxN { num: u32, steps: Vec<f64> },
    Multiple(f64),
    PiMultiple { num: f64, den: f64 },
    Linear(u32),
    Fixed(Vec<f64>),
    Log { base: f64, num: u32 },
}

#[derive(Debug, Clone)]
pub struct Axis {
    pub name: Option<String>,
    pub range: Range,
    pub scale: Scale,
    pub ticks: Option<TickLocator>,
    pub ticks_min: Option<TickLocator>,
}

impl Default for Axis {
    fn default() -> Self {
        Axis {
            name: None,
            range: Range::Auto,
            scale: Scale::Linear,
            ticks: Some(TickLocator::Auto),
            ticks_min: None,
        }
    }
}
