pub mod scale;
pub mod tick;

pub use scale::Scale;

#[derive(Debug, Clone)]
pub struct Axis {
    pub scale: Scale,
    pub label: Option<String>,
    pub ticks: Option<tick::Ticks>,
    pub ticks_min: Option<tick::Locator>,
}

impl Default for Axis {
    fn default() -> Self {
        Axis {
            label: None,
            scale: Default::default(),
            ticks: Some(Default::default()),
            ticks_min: None,
        }
    }
}
