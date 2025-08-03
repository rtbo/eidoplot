pub mod scale;

pub use scale::Scale;

#[derive(Debug, Clone)]
pub enum TickLocator {
    Auto,
    MaxN { num: u32, steps: Vec<f64> },
    Multiple(f64),
    PiMultiple { num: f64, den: f64 },
    Linear(u32),
    Fixed(Vec<f64>),
}

impl Default for TickLocator {
    fn default() -> Self {
        TickLocator::Auto
    }
}

#[derive(Debug, Clone)]
pub struct Axis {
    pub name: Option<String>,
    pub scale: Scale,
    pub ticks: Option<TickLocator>,
    pub ticks_min: Option<TickLocator>,
}

impl Default for Axis {
    fn default() -> Self {
        Axis {
            name: None,
            scale: Default::default(),
            ticks: Some(Default::default()),
            ticks_min: None,
        }
    }
}
