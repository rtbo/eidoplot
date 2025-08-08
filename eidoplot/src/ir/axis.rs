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
    use crate::style::defaults;
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
    }

    impl Default for Ticks {
        fn default() -> Self {
            Ticks {
                locator: Locator::default(),
                formatter: Formatter::default(),
                font: Font::default().with_size(defaults::TICKS_LABEL_FONT_SIZE),
                color: defaults::TICKS_LABEL_COLOR,
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

pub use ticks::{Ticks, MinorTicks};

#[derive(Debug, Clone)]
pub struct Axis {
    pub scale: Scale,
    pub label: Option<String>,
    pub ticks: Option<Ticks>,
    pub minor_ticks: Option<MinorTicks>,
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
