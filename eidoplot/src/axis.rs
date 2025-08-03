pub mod scale;
pub mod tick;

pub use scale::Scale;

#[derive(Debug, Clone)]
pub struct Axis {
    pub name: Option<String>,
    pub scale: Scale,
    pub ticks: Option<tick::Locator>,
    pub ticks_min: Option<tick::Locator>,
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
