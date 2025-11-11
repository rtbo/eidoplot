/*!
 * Axis design module.
 *
 * The structures of this module are used to describe the properties of an axis in a plot.
 * They are not tied to a specific orientation (X or Y), that is handled at the plot level.
 */

pub use ticks::{Grid, MinorGrid, MinorTicks, Ticks, TicksFont};

use crate::style::defaults;

super::define_rich_text_structs!(Title, TitleProps, TitleOptProps);

impl Default for TitleProps {
    fn default() -> Self {
        TitleProps::new(defaults::AXIS_LABEL_FONT_SIZE)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Side {
    /// Axis is on the main side of the plot.
    /// That is bottom for X axis, left for Y axis
    #[default]
    Main,
    /// Axis is on the opposite side of the plot.
    /// That is top for X axis, right for Y axis
    Opposite,
}

/// A reference to another axis, either by id, index or by title.
/// There is no need to specify the orientation (X or Y), as there they are always
/// treated separately.
/// Axes references can be used in two contexts:
///     - sharing axes across different subplots of a figure
///     - attach series to a specific axis in the case of multiple X or Y axes
#[derive(Debug, Clone)]
pub enum Ref {
    /// Reference by index in the order declared in the plot,
    /// for the given orientation (X or Y), and starting at 0.
    /// Can only refer to axes in the same plot.
    /// To refer to indices in a different plot (for subplot shared axes),
    /// you may use [Ref::Id]
    Idx(usize),
    /// Reference by index in the order declared in the figure,
    /// for the given orientation (X or Y), and starting at 0.
    /// As an example, if a figure has 2 plots, each with a single X axis and two Y axes,
    /// Ref::FigIdx(1) will refer to the X axis of the second plot in X context,
    /// and to the second Y axis of the first plot in Y context.
    FigIdx(usize),
    /// Reference by id (see [Axis::id]) or by title (see [Axis::title]).
    /// If two axes share the same id or title, the first one found will be used.
    Id(String),
}

#[derive(Debug, Clone)]
pub struct Axis {
    id: Option<String>,
    title: Option<Title>,
    side: Side,
    scale: Scale,
    ticks: Option<Ticks>,
    minor_ticks: Option<MinorTicks>,
    grid: Option<Grid>,
    minor_grid: Option<MinorGrid>,
}

impl Default for Axis {
    /// Create a new axis with default parameters:
    ///  - automatic linear scale
    ///  - main side (Bottom for X axis, Left for Y axis)
    ///  - no title, no ticks, no grid
    fn default() -> Self {
        Axis {
            id: None,
            title: None,
            side: Default::default(),
            scale: Default::default(),
            ticks: None,
            minor_ticks: None,
            grid: None,
            minor_grid: None,
        }
    }
}

impl Axis {
    /// Effectively the same as `Axis::default()`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the id of this axis.
    /// The id is used to refer to this axis in the context of shared axes.
    /// Note that title can also be used to refer to an axis.
    pub fn with_id(self, id: String) -> Self {
        Self {
            id: Some(id),
            ..self
        }
    }

    pub fn with_title(self, title: Title) -> Self {
        Self {
            title: Some(title),
            ..self
        }
    }

    pub fn with_opposite_side(self) -> Self {
        Self {
            side: Side::Opposite,
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

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn title(&self) -> Option<&Title> {
        self.title.as_ref()
    }
    pub fn side(&self) -> Side {
        self.side
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

    /// Returns whether this axis will show ticks labels
    pub fn has_tick_labels(&self) -> bool {
        match &self.ticks {
            Some(ticks) if self.scale.is_shared() => match ticks.formatter() {
                Some(ticks::Formatter::Auto) => false,
                Some(_) => true,
                None => false,
            },
            Some(ticks) => ticks.formatter().is_some(),
            None => false,
        }
    }
}

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

/// Describe a logarithmic scale options
#[derive(Debug, Clone, Copy)]
pub struct LogScale {
    pub base: f64,
    pub range: Range,
}

impl LogScale {
    pub fn new(base: f64, range: Range) -> Self {
        Self { base, range }
    }
}

impl Default for LogScale {
    fn default() -> Self {
        Self::new(10.0, Range::Auto)
    }
}

/// Describes the type of an axis scale
#[derive(Debug, Clone, Default)]
pub enum Scale {
    /// Full auto scale, depending on the data and type of plot.
    /// Will typically translate to auto linear axis for numerical data
    /// and auto categorical axis for categorical data
    #[default]
    Auto,
    /// Linear axis
    Linear(Range),
    /// Logarithmic axis
    Log(LogScale),
    /// Scale is shared with another axis.
    /// This is used when an axis is shared between two plots.
    /// In the context of shared axes, it is only the scale that is shared.
    /// Each axis can have its own title, ticks, grid, side, etc.
    Shared(Ref),
}

impl From<Range> for Scale {
    fn from(range: Range) -> Self {
        Scale::Linear(range)
    }
}

impl From<LogScale> for Scale {
    fn from(scale: LogScale) -> Self {
        Scale::Log(scale)
    }
}

impl Scale {
    /// Returns true if the scale is shared
    pub fn is_shared(&self) -> bool {
        matches!(self, Scale::Shared(_))
    }

    pub fn shared_ref(&self) -> Option<&Ref> {
        match self {
            Scale::Shared(ref_) => Some(ref_),
            _ => None,
        }
    }
}

/// Describe the ticks of an axis
pub mod ticks {
    use eidoplot_text::Font;

    use crate::style::{self, Dash, defaults, theme};

    /// Describes how to locate the ticks of an axis
    #[derive(Debug, Default, Clone)]
    pub enum Locator {
        /// Automatic tick placement, that depends on the type of axis (linear, logarithmic, categories),
        /// on the axis data range (bounds) and whether the ticks are major or minor
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
        /// Places ticks on a logarithmic scale, using the specified base and max number of bins
        Log {
            /// Logarithm base
            base: f64,
            /// Number of bins (that is number of ticks - 1)
            bins: u32,
        },
    }

    /// Describes how to format the ticks labels
    #[derive(Debug, Default, Clone, Copy)]
    pub enum Formatter {
        /// Automatic tick formatting.
        /// Depending on the scale and locator, the formatter will pick a suitable format.
        /// If the scale is shared, Some(Formatter::Auto) is equivalent to None. This effectively
        /// disables the tick labels, which is generally the desired behavior when sharing axes.
        /// You can actively set [Formatter::SharedAuto] to enable automatic formatting even for shared axes.
        #[default]
        Auto,
        /// Same as [Formatter::Auto] for all axes, even those that are shared.
        SharedAuto,
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
                font: defaults::FONT_FAMILY.parse().unwrap(),
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
        locator: Locator,
        formatter: Option<Formatter>,
        font: TicksFont,
        color: theme::Color,
    }

    impl Default for Ticks {
        /// Return the default tick configuration:
        /// - automatic locator
        /// - labels with automatic formatter (unless the scale is shared)
        /// - default font and theme foreground color
        fn default() -> Self {
            Ticks {
                locator: Locator::default(),
                formatter: Some(Formatter::default()),
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
        pub fn with_formatter(self, formatter: Option<Formatter>) -> Self {
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

        /// Generates the ticks at the specified locations
        pub fn locator(&self) -> &Locator {
            &self.locator
        }
        /// Formats the ticks labels.
        /// If None, no labels are shown, and the layout is packed accordingly.
        pub fn formatter(&self) -> Option<Formatter> {
            self.formatter
        }
        /// Font for the ticks labels
        pub fn font(&self) -> &TicksFont {
            &self.font
        }
        /// Color for the ticks and the labels
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
