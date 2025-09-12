/*!
 * Axis design module
 */

/// Describe the bounds of an axis in data space
#[derive(Debug, Clone, Copy, Default)]
pub enum Range {
    /// Auto determine the bounds
    #[default]
    Auto,
    /// Lower bound defined, and upper bound automatic
    MinAuto(f64),
    /// Higher bound defined, and upper bound automatic
    AutoMax(f64),
    /// Lower and higher bound defined
    MinMax(f64, f64),
}

/// Describes the type of an axis scale
#[derive(Debug, Clone, Copy, Default)]
pub enum Scale {
    /// Full auto scale, depending on the data and type of plot.
    /// Will typically translate to auto linear axis for numerical data
    /// and auto categorical axis for categorical data
    #[default]
    Auto,
    /// Linear axis
    Linear(Range),
    /// Logarithmic axis
    Log(Range),
}

/// Describe the ticks of an axis
pub mod ticks {
    use eidoplot_text::Font;

    use crate::style::{self, Dash, defaults, theme};

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

    /// Describes the style of the major grid lines
    #[derive(Debug, Clone)]
    pub struct Grid(pub theme::Line);

    impl Default for Grid {
        fn default() -> Self {
            Grid(theme::Line {
                width: 1.0,
                color: theme::Col::Grid.into(),
                pattern: style::LinePattern::Solid,
                opacity: None,
            })
        }
    }

    impl From<theme::Line> for Grid {
        fn from(line: theme::Line) -> Self {
            Grid(line)
        }
    }

    /// Describes the major ticks of an axis
    #[derive(Debug, Clone)]
    pub struct Ticks {
        /// Generates the ticks at the specified locations
        locator: Locator,
        /// Formats the ticks labels
        formatter: Formatter,
        /// Font for the ticks labels
        font: TicksFont,
        /// Color for the ticks and the labels
        color: theme::Color,
    }

    impl Default for Ticks {
        fn default() -> Self {
            Ticks {
                locator: Locator::default(),
                formatter: Formatter::default(),
                font: TicksFont::default(),
                color: theme::Col::Foreground.into(),
            }
        }
    }

    impl Ticks {
        pub fn new() -> Self {
            Self::default()
        }

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
        pub fn with_color(self, color: theme::Color) -> Self {
            Self { color, ..self }
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
        pub fn color(&self) -> theme::Color {
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

    /// Describes the style of the minor grid lines
    #[derive(Debug, Clone)]
    pub struct MinorGrid(pub theme::Line);

    impl Default for MinorGrid {
        fn default() -> Self {
            MinorGrid(theme::Line {
                width: 0.5,
                color: theme::Col::Grid.into(),
                pattern: style::LinePattern::Dash(Dash::default()),
                opacity: None,
            })
        }
    }

    impl From<theme::Line> for MinorGrid {
        fn from(line: theme::Line) -> Self {
            MinorGrid(line)
        }
    }

    #[derive(Debug, Clone)]
    pub struct MinorTicks {
        /// Minor ticks locator
        locator: Locator,
        /// Ticks color
        color: theme::Color,
    }

    impl Default for MinorTicks {
        fn default() -> Self {
            MinorTicks {
                locator: Locator::default(),
                color: theme::Col::Foreground.into(),
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
        pub fn new() -> Self {
            Self::default()
        }
        pub fn with_locator(self, locator: Locator) -> Self {
            Self { locator, ..self }
        }
        pub fn with_color(self, color: theme::Color) -> Self {
            Self { color, ..self }
        }

        pub fn locator(&self) -> &Locator {
            &self.locator
        }
        pub fn color(&self) -> theme::Color {
            self.color
        }
    }
}

pub use ticks::{Grid, MinorGrid, MinorTicks, Ticks, TicksFont};

use crate::style::{self, defaults, theme};

#[derive(Debug, Clone)]
pub struct TitleFont {
    pub font: style::Font,
    pub size: f32,
    pub color: theme::Color,
}

impl Default for TitleFont {
    fn default() -> Self {
        TitleFont {
            font: defaults::AXIS_LABEL_FONT_FAMILY.parse().unwrap(),
            size: defaults::AXIS_LABEL_FONT_SIZE,
            color: theme::Col::Foreground.into(),
        }
    }
}

impl TitleFont {
    pub fn font(&self) -> &style::Font {
        &self.font
    }
}

#[derive(Debug, Clone)]
pub struct Title {
    text: String,
    font: TitleFont,
}

impl Title {
    pub fn new(text: String) -> Self {
        Title {
            text,
            font: TitleFont::default(),
        }
    }

    pub fn with_font(mut self, font: TitleFont) -> Self {
        self.font = font;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn font(&self) -> &TitleFont {
        &self.font
    }
}

impl From<String> for Title {
    fn from(value: String) -> Self {
        Title::new(value)
    }
}

#[derive(Debug, Clone)]
pub struct Axis {
    title: Option<Title>,
    scale: Scale,
    ticks: Option<Ticks>,
    minor_ticks: Option<MinorTicks>,
    grid: Option<Grid>,
    minor_grid: Option<MinorGrid>,
}

impl Default for Axis {
    fn default() -> Self {
        Axis {
            title: None,
            scale: Default::default(),
            ticks: None,
            minor_ticks: None,
            grid: None,
            minor_grid: None,
        }
    }
}

impl Axis {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_title(self, title: Title) -> Self {
        Self {
            title: Some(title),
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

    /// Returns a new axis with the specified grid
    /// If this axis has no major ticks, default ticks are
    /// created and used to locate the grid
    pub fn with_grid(self, grid: Grid) -> Self {
        Self {
            ticks: Some(self.ticks.unwrap_or_default()),
            grid: Some(grid),
            ..self
        }
    }

    /// Returns a new axis with the specified minor grid
    /// If this axis has no minor ticks, default ticks are
    /// created and used to locate the grid
    pub fn with_minor_grid(self, minor_grid: MinorGrid) -> Self {
        Self {
            minor_ticks: Some(self.minor_ticks.unwrap_or_default()),
            minor_grid: Some(minor_grid),
            ..self
        }
    }

    pub fn title(&self) -> Option<&Title> {
        self.title.as_ref()
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
    /// Gridline style
    pub fn grid(&self) -> Option<&Grid> {
        self.grid.as_ref()
    }
    /// Minor gridline style
    pub fn minor_grid(&self) -> Option<&MinorGrid> {
        self.minor_grid.as_ref()
    }
}
