/*!
 * Axis design module
 */

/// Describe the bounds of an axis in data space
#[derive(Debug, Clone, Copy)]
pub enum Range {
    /// Auto determine the bounds
    Auto,
    /// Lower bound defined, and upper bound automatic
    MinAuto(f64),
    /// Higher bound defined, and upper bound automatic
    AutoMax(f64),
    /// Lower and higher bound defined
    MinMax(f64, f64),
}

/// Describes the type of an axis
#[derive(Debug, Clone, Copy)]
pub enum Scale {
    /// Linear axis
    Linear(Range),
}

impl Default for Scale {
    fn default() -> Self {
        Scale::Linear(Range::Auto)
    }
}

/// Describe the ticks of an axis
pub mod ticks {
    use eidoplot_text::Font;

    use crate::style::{self, Color, defaults};

    /// Describes how to locate the ticks of an axis
    #[derive(Debug, Default, Clone)]
    pub enum Locator {
        /// Automatic tick placement. This is equvalent to `MaxN { bins: 10 }` with relevant decimal steps
        #[default]
        Auto,
        /// Places ticks automatically, using the specified number of bins and steps
        MaxN {
            /// Number of bins (that is number of ticks - 1)
            bins: u32,
            /// List of steps multiple to the scale
            /// The locator will pick one of the steps, multiplying it by a power of 10 scale
            steps: Vec<f64>,
        },
        /// Places the ticks automatically, using the specified number of bins and multiples of PI.
        /// The axis will be annotated with `× π`
        PiMultiple {
            /// Number of bins (that is number of ticks - 1)
            bins: u32,
        },
    }


    /// Describes how to format the ticks labels
    #[derive(Debug, Default, Clone, Copy)]
    pub enum Formatter {
        /// Automatic tick formatting
        #[default]
        Auto,
        /// Format the ticks with decimal precision
        Prec(usize),
        /// The labels are percentages (E.g. `0.5` will be formatted as `50%`)
        Percent,
    }

    /// Describes the font of the ticks labels
    #[derive(Debug, Clone)]
    pub struct TicksFont {
        /// The font of the ticks labels
        pub font: Font,
        /// The font size of the ticks labels
        pub size: f32,
    }

    impl Default for TicksFont {
        fn default() -> Self {
            TicksFont {
                font: defaults::TICKS_LABEL_FONT_FAMILY.parse().unwrap(),
                size: defaults::TICKS_LABEL_FONT_SIZE,
            }
        }
    }

    /// Describes the ticks of an axis
    #[derive(Debug, Clone)]
    pub struct Ticks {
        /// Generates the ticks at the specified locations
        locator: Locator,
        /// Formats the ticks labels
        formatter: Formatter,
        /// Font for the ticks labels
        font: TicksFont,
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
                font: TicksFont::default(),
                color: defaults::TICKS_LABEL_COLOR,
                grid: defaults::TICKS_GRID_LINE,
            }
        }
    }

    impl Ticks {
        /// Returns a new `Ticks` with the specified locator
        pub fn with_locator(self, locator: Locator) -> Self {
            Self { locator, ..self }
        }
        /// Returns a new `Ticks` with the specified formatter
        pub fn with_formatter(self, formatter: Formatter) -> Self {
            Self { formatter, ..self }
        }
        /// Returns a new ticks with the specified font
        pub fn with_font(self, font: TicksFont) -> Self {
            Self { font, ..self }
        }
        /// Returns a new ticks with the specified color
        pub fn with_color(self, color: Color) -> Self {
            Self { color, ..self }
        }
        /// Returns a new ticks with the specified grid
        pub fn with_grid(self, grid: Option<style::Line>) -> Self {
            Self { grid, ..self }
        }

        /// The locator
        pub fn locator(&self) -> &Locator {
            &self.locator
        }
        /// The formatter
        pub fn formatter(&self) -> &Formatter {
            &self.formatter
        }
        /// The font
        pub fn font(&self) -> &TicksFont {
            &self.font
        }
        /// The color
        pub fn color(&self) -> Color {
            self.color
        }
        /// The grid
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

pub use ticks::{MinorTicks, Ticks, TicksFont};

use crate::style::{self, defaults};

#[derive(Debug, Clone)]
pub struct LabelFont {
    pub font: style::Font,
    pub size: f32,
}

impl Default for LabelFont {
    fn default() -> Self {
        LabelFont {
            font: defaults::AXIS_LABEL_FONT_FAMILY.parse().unwrap(),
            size: defaults::AXIS_LABEL_FONT_SIZE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    pub text: String,
    pub font: LabelFont,
}

impl From<&str> for Label {
    fn from(value: &str) -> Self {
        Label {
            text: value.to_string(),
            font: LabelFont::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Axis {
    scale: Scale,
    label: Option<Label>,
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

    pub fn with_label(self, label: Label) -> Self {
        Self {
            label: Some(label),
            ..self
        }
    }

    pub fn with_scale(self, scale: Scale) -> Self {
        Self { scale, ..self }
    }

    pub fn with_ticks(self, ticks: Ticks) -> Self {
        Self {
            ticks: Some(ticks),
            ..self
        }
    }

    pub fn with_minor_ticks(self, minor_ticks: MinorTicks) -> Self {
        Self {
            minor_ticks: Some(minor_ticks),
            ..self
        }
    }

    pub fn label(&self) -> Option<&Label> {
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
