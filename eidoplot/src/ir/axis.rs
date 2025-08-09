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
    Linear(Range),
}

impl Default for Scale {
    fn default() -> Self {
        Scale::Linear(Range::Auto)
    }
}

pub mod ticks {
    use crate::style::{self, defaults};
    use crate::style::{Color, Font};

    #[derive(Debug, Default, Clone)]
    pub enum Locator {
        #[default]
        Auto,
        MaxN {
            bins: u32,
            steps: Vec<f64>,
        },
        PiMultiple {
            bins: u32,
        },
    }

    #[derive(Debug, Default, Clone)]
    pub enum Formatter {
        #[default]
        Auto,
        Prec(usize),
    }

    #[derive(Debug, Clone)]
    pub struct Ticks {
        /// Generates the ticks at the specified locations
        locator: Locator,
        /// Formats the ticks labels
        formatter: Formatter,
        /// Font for the ticks labels
        font: Font,
        /// Color for the ticks and the labels
        color: Color,
        /// Gridline style
        grid: Option<style::Line>,
    }

    impl Default for Ticks {
        fn default() -> Self {
            Ticks {
                locator: Locator::default(),
                formatter: Formatter::default(),
                font: Font::new(
                    defaults::TICKS_LABEL_FONT_FAMILY.into(),
                    defaults::TICKS_LABEL_FONT_SIZE,
                ),
                color: defaults::TICKS_LABEL_COLOR,
                grid: defaults::TICKS_GRID_LINE,
            }
        }
    }

    impl Ticks {
        pub fn with_locator(self, locator: Locator) -> Self {
            Self { locator, ..self }
        }
        pub fn with_formatter(self, formatter: Formatter) -> Self {
            Self { formatter, ..self }
        }
        pub fn with_font(self, font: Font) -> Self {
            Self { font, ..self }
        }
        pub fn with_color(self, color: Color) -> Self {
            Self { color, ..self }
        }
        pub fn with_grid(self, grid: Option<style::Line>) -> Self {
            Self { grid, ..self }
        }

        pub fn locator(&self) -> &Locator {
            &self.locator
        }
        pub fn formatter(&self) -> &Formatter {
            &self.formatter
        }
        pub fn font(&self) -> &Font {
            &self.font
        }
        pub fn color(&self) -> Color {
            self.color
        }
        pub fn grid(&self) -> Option<&style::Line> {
            self.grid.as_ref()
        }
    }

    impl From<Locator> for Ticks {
        fn from(value: Locator) -> Self {
            Ticks {
                locator: value,
                ..Default::default()
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct MinorTicks {
        pub locator: Locator,
        pub color: Color,
    }

    impl Default for MinorTicks {
        fn default() -> Self {
            MinorTicks {
                locator: Locator::default(),
                color: defaults::TICKS_LABEL_COLOR,
            }
        }
    }

    impl From<Locator> for MinorTicks {
        fn from(value: Locator) -> Self {
            MinorTicks {
                locator: value,
                ..Default::default()
            }
        }
    }

    impl MinorTicks {
        pub fn with_locator(self, locator: Locator) -> Self {
            Self { locator, ..self }
        }
        pub fn with_color(self, color: Color) -> Self {
            Self { color, ..self }
        }

        pub fn locator(&self) -> &Locator {
            &self.locator
        }
        pub fn color(&self) -> Color {
            self.color
        }
    }
}

pub use ticks::{MinorTicks, Ticks};

use crate::{ir::Text};

#[derive(Debug, Clone)]
pub struct Axis {
    scale: Scale,
    label: Option<Text>,
    ticks: Option<Ticks>,
    minor_ticks: Option<MinorTicks>,
}

impl Default for Axis {
    fn default() -> Self {
        Axis {
            label: None,
            scale: Default::default(),
            ticks: Some(Default::default()),
            minor_ticks: None,
        }
    }
}

impl Axis {
    pub fn new(scale: Scale) -> Self {
        Axis {
            scale,
            ..Default::default()
        }
    }

    pub fn with_label(self, label: Text) -> Self {
        Self { label: Some(label), ..self }
    }

    pub fn with_scale(self, scale: Scale) -> Self {
        Self { scale, ..self }
    }

    pub fn with_ticks(self, ticks: Ticks) -> Self {
        Self { ticks: Some(ticks), ..self }
    }

    pub fn with_minor_ticks(self, minor_ticks: MinorTicks) -> Self {
        Self { minor_ticks: Some(minor_ticks), ..self }
    }

    pub fn label(&self) -> Option<&Text> {
        self.label.as_ref()
    }
    pub fn scale(&self) -> &Scale {
        &self.scale
    }
    pub fn ticks(&self) -> Option<&Ticks> {
        self.ticks.as_ref()
    }
    pub fn minor_ticks(&self) -> Option<&MinorTicks> {
        self.minor_ticks.as_ref()
    }
}
