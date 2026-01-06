/*!
 * This module deals with colors and style of data series.
 */
use crate::{ResolveColor, ColorU8};
use crate::style::{self, catppuccin, defaults};

/// A palette for data series.
/// It provides ordered colors for series in a figure.
/// If more series are present than colors in the palette,
/// colors are reused in order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Palette {
    /// Black monochrome palette
    Black,
    #[default]
    /// Standard plotive palette
    Standard,
    /// Pastel plotive palette
    Pastel,
    /// Paul Tol's bright colorblind-safe palette
    TolBright,
    /// Okabe & Ito colorblind-safe palette
    OkabeIto,
    /// Catppuccin Mocha palette
    CatppuccinMocha,
    /// Catppuccin Macchiato palette
    CatppuccinMacchiato,
    /// Catppuccin Frappe palette
    CatppuccinFrappe,
    /// Catppuccin Latte palette
    CatppuccinLatte,
    /// A custom palette
    Custom(Vec<ColorU8>),
}

impl Palette {
    /// Get the colors in the palette
    pub const fn colors(&self) -> &[ColorU8] {
        match self {
            Palette::Black => palettes::BLACK,
            Palette::Standard => palettes::STANDARD,
            Palette::Pastel => palettes::PASTEL,
            Palette::TolBright => palettes::TOL_BRIGHT,
            Palette::OkabeIto => palettes::OKABE_ITO,
            Palette::CatppuccinMocha => catppuccin::series_colors::<catppuccin::Mocha>(),
            Palette::CatppuccinMacchiato => catppuccin::series_colors::<catppuccin::Macchiato>(),
            Palette::CatppuccinFrappe => catppuccin::series_colors::<catppuccin::Frappe>(),
            Palette::CatppuccinLatte => catppuccin::series_colors::<catppuccin::Latte>(),
            Palette::Custom(colors) => colors.as_slice(),
        }
    }

    /// Get the number of colors in the palette
    pub const fn len(&self) -> usize {
        self.colors().len()
    }

    /// Get a color from the palette by its index
    pub const fn get(&self, col: IndexColor) -> ColorU8 {
        self.colors()[col.0 % self.len()]
    }
}


/// A series color identified by its index in a palette
#[derive(Debug, Clone, Copy)]
pub struct IndexColor(pub usize);

impl style::Color for IndexColor {}

/// A series color that is automatically chosen from a palette based on the series index
#[derive(Debug, Clone, Copy)]
pub struct AutoColor;

impl style::Color for AutoColor {}

/// A flexible color for data series
#[derive(Debug, Clone, Copy, Default)]
pub enum Color {
    /// Automatic color from the palette
    #[default]
    Auto,
    /// Color from the palette by index
    Index(IndexColor),
    /// Fixed RGB color
    Fixed(ColorU8),
}

impl From<IndexColor> for Color {
    fn from(color: IndexColor) -> Self {
        Color::Index(color)
    }
}

impl From<AutoColor> for Color {
    fn from(_color: AutoColor) -> Self {
        Color::Auto
    }
}

impl From<ColorU8> for Color {
    fn from(color: ColorU8) -> Self {
        Color::Fixed(color)
    }
}

impl style::Color for Color {}

impl ResolveColor<IndexColor> for Palette
{
    fn resolve_color(&self, col: &IndexColor) -> ColorU8 {
        self.get(*col)
    }
}

impl ResolveColor<AutoColor> for (&Palette, usize)
{
    fn resolve_color(&self, _col: &AutoColor) -> ColorU8 {
        self.0.get(IndexColor(self.1))
    }
}

impl ResolveColor<Color> for (&Palette, usize)
{
    fn resolve_color(&self, col: &Color) -> ColorU8 {
        match col {
            Color::Auto => self.0.get(IndexColor(self.1)),
            Color::Index(idx) => self.0.get(*idx),
            Color::Fixed(c) => *c,
        }
    }
}

/// Line style for theme elements
pub type Line = style::Line<Color>;

impl Default for Line {
    fn default() -> Self {
        Line {
            color: Color::default(),
            width: defaults::SERIES_LINE_WIDTH,
            pattern: style::LinePattern::default(),
            opacity: None,
        }
    }
}

impl From<ColorU8> for Line {
    fn from(color: ColorU8) -> Self {
        Line {
            color: color.into(),
            width: defaults::SERIES_LINE_WIDTH,
            pattern: style::LinePattern::default(),
            opacity: None,
        }
    }
}

/// Fill style for theme elements
pub type Fill = style::Fill<Color>;

impl From<ColorU8> for Fill {
    fn from(color: ColorU8) -> Self {
        Fill::Solid {
            color: color.into(),
            opacity: None,
        }
    }
}

/// Marker style for theme elements
pub type Marker = style::Marker<Color>;

impl From<ColorU8> for Marker {
    fn from(color: ColorU8) -> Self {
        Marker {
            size: Default::default(),
            shape: Default::default(),
            fill: Some(Fill::Solid {
                color: color.into(),
                opacity: None,
            }),
            stroke: None,
        }
    }
}

/// Types for built-in and custom palettes
mod palettes {
    use crate::ColorU8;

    pub const BLACK: &[ColorU8] = &[ColorU8::from_html(b"#000000")];
    pub const STANDARD: &[ColorU8] = &[
        ColorU8::from_html(b"#1f77b4"), // blue
        ColorU8::from_html(b"#ff7f0e"), // orange
        ColorU8::from_html(b"#2ca02c"), // green
        ColorU8::from_html(b"#d62728"), // red
        ColorU8::from_html(b"#9467bd"), // purple
        ColorU8::from_html(b"#8c564b"), // brown
        ColorU8::from_html(b"#e377c2"), // pink
        ColorU8::from_html(b"#7f7f7f"), // gray
        ColorU8::from_html(b"#bcbd22"), // olive
        ColorU8::from_html(b"#17becf"), // cyan
    ];
    pub const PASTEL: &[ColorU8] = &[
        ColorU8::from_html(b"#aec7e8"), // light blue
        ColorU8::from_html(b"#ffbb78"), // light orange
        ColorU8::from_html(b"#98df8a"), // light green
        ColorU8::from_html(b"#ff9896"), // light red
        ColorU8::from_html(b"#c5b0d5"), // light purple
        ColorU8::from_html(b"#c49c94"), // light brown
        ColorU8::from_html(b"#f7b6d2"), // light pink
        ColorU8::from_html(b"#c7c7c7"), // light gray
        ColorU8::from_html(b"#dbdb8d"), // light olive
        ColorU8::from_html(b"#9edae5"), // light cyan
    ];
    pub const TOL_BRIGHT: &[ColorU8] = &[
        ColorU8::from_html(b"#4477AA"), // blue
        ColorU8::from_html(b"#EE6677"), // red
        ColorU8::from_html(b"#228833"), // green
        ColorU8::from_html(b"#CCBB44"), // yellow
        ColorU8::from_html(b"#66CCEE"), // cyan
        ColorU8::from_html(b"#AA3377"), // purple
        ColorU8::from_html(b"#BBBBBB"), // gray
    ];
    pub const OKABE_ITO: &[ColorU8] = &[
        ColorU8::from_html(b"#E69F00"), // orange
        ColorU8::from_html(b"#56B4E9"), // sky blue
        ColorU8::from_html(b"#009E73"), // bluish green
        ColorU8::from_html(b"#F0E442"), // yellow
        ColorU8::from_html(b"#0072B2"), // blue
        ColorU8::from_html(b"#D55E00"), // vermillion
        ColorU8::from_html(b"#CC79A7"), // reddish purple
    ];
}
