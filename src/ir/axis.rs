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

/// Side of the axis in the plot, applies to both X and Y axes.
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
    /// If two axes have the same id or title, the first one found will be used.
    Id(String),
}

impl Default for Ref {
    fn default() -> Self {
        Ref::Idx(0)
    }
}

impl<T> From<T> for Ref
where
    T: Into<String>,
{
    fn from(id: T) -> Self {
        Ref::Id(id.into())
    }
}

/// Create an axis reference from a string id.
/// The id can refer to the axis id (see [Axis::id]) or to the axis title (see [Axis::title]).
pub fn ref_id(id: impl Into<String>) -> Ref {
    Ref::Id(id.into())
}

/// Axis definition
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
    pub fn with_id(self, id: impl Into<String>) -> Self {
        Self {
            id: Some(id.into()),
            ..self
        }
    }

    /// Set the title of this axis and return self for chaining
    pub fn with_title(self, title: Title) -> Self {
        Self {
            title: Some(title),
            ..self
        }
    }

    /// Set this axis on the opposite side of the plot and return self for chaining
    pub fn with_opposite_side(self) -> Self {
        Self {
            side: Side::Opposite,
            ..self
        }
    }

    /// Set the side of this axis and return self for chaining
    pub fn with_scale(self, scale: Scale) -> Self {
        Self { scale, ..self }
    }

    /// Set the ticks of this axis and return self for chaining
    pub fn with_ticks(self, ticks: Ticks) -> Self {
        Self {
            ticks: Some(ticks),
            ..self
        }
    }

    /// Set the minor ticks of this axis and return self for chaining
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

    /// Get the id of this axis, if any
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Get the title of this axis, if any
    pub fn title(&self) -> Option<&Title> {
        self.title.as_ref()
    }

    /// Get the side of this axis
    pub fn side(&self) -> Side {
        self.side
    }

    /// Get the scale of this axis
    pub fn scale(&self) -> &Scale {
        &self.scale
    }

    /// Major ticks configuration
    pub fn ticks(&self) -> Option<&Ticks> {
        self.ticks.as_ref()
    }

    /// Minor ticks configuration
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
    /// Logarithm base (typically 10.0)
    pub base: f64,
    /// Data range (both min and max must have the same sign)
    pub range: Range,
}

impl LogScale {
    /// Create a new logarithmic scale with the specified base and range
    ///
    /// Panics if the range min and max have different signs
    /// For automatic min or max, the panic might happen during drawing, when the data bounds are known.
    pub fn new(base: f64, range: Range) -> Self {
        if let Range::MinMax(min, max) = range {
            assert!(
                (min > 0.0 && max > 0.0) || (min < 0.0 && max < 0.0),
                "LogScale range min and max must have the same sign"
            );
        }

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

impl From<Ref> for Scale {
    fn from(ref_: Ref) -> Self {
        Scale::Shared(ref_)
    }
}

impl Scale {
    /// Returns true if the scale is shared
    pub fn is_shared(&self) -> bool {
        matches!(self, Scale::Shared(_))
    }

    /// If the scale is shared, return the reference to the shared axis
    pub fn shared_ref(&self) -> Option<&Ref> {
        match self {
            Scale::Shared(ref_) => Some(ref_),
            _ => None,
        }
    }
}

/// Describe the ticks of an axis
pub mod ticks {
    use crate::style::{self, Dash, defaults, theme};
    use crate::text::Font;

    /// Describes how to locate the ticks of an axis
    #[derive(Debug, Default, Clone)]
    pub enum Locator {
        /// Automatic tick placement, that depends on the type of axis (linear, logarithmic, categories),
        /// on the axis data range (bounds) and whether the ticks are major or minor
        #[default]
        Auto,
        /// Places ticks automatically, using the specified number of bins and steps
        MaxN(MaxNLocator),
        /// Places the ticks automatically, using the specified number of bins and multiples of PI.
        /// The axis will be annotated with `× π`
        PiMultiple(PiMultipleLocator),
        /// Places ticks on a logarithmic scale, using the specified base and max number of bins
        Log(LogLocator),
        #[cfg(feature = "time")]
        /// Places ticks on a time scale
        /// The series data must be DateTime, otherwise an error is returned.
        DateTime(DateTimeLocator),
        #[cfg(feature = "time")]
        /// Places ticks on a time delta scale
        /// The series data can be either numeric or TimeDelta.
        /// In the case of numeric data, seconds are assumed.
        TimeDelta(TimeDeltaLocator),
    }

    /// A locator that places ticks automatically, using the specified number of bins and steps
    #[derive(Debug, Clone)]
    pub struct MaxNLocator {
        /// Number of bins (that is number of ticks - 1)
        pub bins: u32,
        /// List of steps multiple to the scale
        /// The locator will pick one of the steps, multiplying it by a power of 10 scale
        pub steps: Vec<f64>,
    }

    impl Default for MaxNLocator {
        fn default() -> Self {
            MaxNLocator {
                bins: 9,
                steps: vec![1.0, 2.0, 2.5, 5.0],
            }
        }
    }

    impl From<MaxNLocator> for Locator {
        fn from(locator: MaxNLocator) -> Self {
            Locator::MaxN(locator)
        }
    }

    /// A locator that places ticks at multiples of π
    /// The axis will be annotated with `× π`
    #[derive(Debug, Clone, Copy)]
    pub struct PiMultipleLocator {
        /// Number of bins (that is number of ticks - 1)
        pub bins: u32,
    }

    impl Default for PiMultipleLocator {
        fn default() -> Self {
            PiMultipleLocator { bins: 9 }
        }
    }

    impl From<PiMultipleLocator> for Locator {
        fn from(locator: PiMultipleLocator) -> Self {
            Locator::PiMultiple(locator)
        }
    }

    /// A locator that places ticks on a logarithmic scale
    #[derive(Debug, Clone, Copy)]
    pub struct LogLocator {
        /// Logarithm base
        pub base: f64,
    }

    impl Default for LogLocator {
        fn default() -> Self {
            LogLocator { base: 10.0 }
        }
    }

    impl From<LogLocator> for Locator {
        fn from(locator: LogLocator) -> Self {
            Locator::Log(locator)
        }
    }

    #[cfg(feature = "time")]
    /// Describes how to locate the ticks of a DateTime axis
    #[derive(Debug, Default, Clone, Copy)]
    pub enum DateTimeLocator {
        /// Automatic tick placement for DateTime axis using
        /// the axis bounds and heuristics to have a reasonable number of ticks
        #[default]
        Auto,
        /// Place ticks every N years
        Years(u32),
        /// Place ticks every N months
        Months(u32),
        /// Place ticks every N weeks
        Weeks(u32),
        /// Place ticks every N days
        Days(u32),
        /// Place ticks every N hours
        Hours(u32),
        /// Place ticks every N minutes
        Minutes(u32),
        /// Place ticks every N seconds
        Seconds(u32),
        /// Place ticks every N microseconds
        Micros(u32),
    }

    #[cfg(feature = "time")]
    impl From<DateTimeLocator> for Locator {
        fn from(locator: DateTimeLocator) -> Self {
            Locator::DateTime(locator)
        }
    }

    #[cfg(feature = "time")]
    /// Describes how to locate the ticks of a TimeDelta axis
    #[derive(Debug, Default, Clone, Copy)]
    pub enum TimeDeltaLocator {
        /// Automatic tick placement for TimeDelta axis using
        /// the axis bounds and heuristics to have a reasonable number of ticks
        #[default]
        Auto,
        /// Place ticks every N days
        Days(u32),
        /// Place ticks every N hours
        Hours(u32),
        /// Place ticks every N minutes
        Minutes(u32),
        /// Place ticks every N seconds
        Seconds(u32),
        /// Place ticks every N microseconds
        Micros(u32),
    }

    #[cfg(feature = "time")]
    impl From<TimeDeltaLocator> for Locator {
        fn from(locator: TimeDeltaLocator) -> Self {
            Locator::TimeDelta(locator)
        }
    }

    #[allow(missing_copy_implementations)]
    /// Describes how to format the ticks labels
    #[derive(Debug, Default, Clone)]
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
        Percent(PercentFormatter),
        #[cfg(feature = "time")]
        /// Formats the time ticks
        /// The data must be DateTime, otherwise an error is returned.
        DateTime(DateTimeFormatter),
        #[cfg(feature = "time")]
        /// Formats the time delta ticks
        /// The series must be either TimeDelta or f64, otherwise an error is returned
        /// If the data is f64, it is assumed to be in seconds
        TimeDelta(TimeDeltaFormatter),
    }

    /// A label formatter for DateTime ticks
    #[derive(Debug, Clone, Copy, Default)]
    pub struct PercentFormatter {
        /// Number of decimal places
        /// None means automatic
        pub decimal_places: Option<usize>,
    }

    impl From<PercentFormatter> for Formatter {
        fn from(fmt: PercentFormatter) -> Self {
            Formatter::Percent(fmt)
        }
    }

    #[cfg(feature = "time")]
    /// A label formatter for DateTime ticks
    #[derive(Debug, Clone, Default)]
    pub enum DateTimeFormatter {
        /// Choose the format automatically according to time bounds
        #[default]
        Auto,
        /// Format dates as `YYYY-MM-DD HH:MM:SS`
        DateTime,
        /// Format dates as `YYYY-MM-DD`
        Date,
        /// Format time as `HH:MM:SS`
        Time,
        /// Format the ticks with a custom DateTime format (see [crate::time::DateTime::fmt_parse])
        Custom(String),
    }

    #[cfg(feature = "time")]
    impl From<DateTimeFormatter> for Formatter {
        fn from(fmt: DateTimeFormatter) -> Self {
            Formatter::DateTime(fmt)
        }
    }

    #[cfg(feature = "time")]
    /// A label formatter for TimeDelta ticks
    #[derive(Debug, Clone, Default)]
    pub enum TimeDeltaFormatter {
        /// Choose the format automatically based on data bounds
        #[default]
        Auto,
        /// Format the ticks with a custom TimeDelta format (see [crate::time::TimeDelta::fmt_parse])
        Custom(String),
    }

    #[cfg(feature = "time")]
    impl From<TimeDeltaFormatter> for Formatter {
        fn from(fmt: TimeDeltaFormatter) -> Self {
            Formatter::TimeDelta(fmt)
        }
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
        /// Returns a new `Ticks` with default parameters.
        /// (same as [`Ticks::default()`])
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
        pub fn formatter(&self) -> Option<&Formatter> {
            self.formatter.as_ref()
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

    /// Describes the minor ticks of an axis
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
        /// Returns a new `MinorTicks` with default parameters.
        /// (same as [`MinorTicks::default()`])
        pub fn new() -> Self {
            Self::default()
        }
        /// Returns a new `MinorTicks` with the specified locator and return self for chaining
        pub fn with_locator(self, locator: Locator) -> Self {
            Self { locator, ..self }
        }
        /// Returns a new `MinorTicks` with the specified color and return self for chaining
        pub fn with_color(self, color: theme::Color) -> Self {
            Self { color, ..self }
        }

        /// Get the locator of these minor ticks
        pub fn locator(&self) -> &Locator {
            &self.locator
        }
        /// Get the color of these minor ticks
        pub fn color(&self) -> theme::Color {
            self.color
        }
    }
}
