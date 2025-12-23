/*!
 * This module deals with colors and style of data series.
 */
use crate::ColorU8;
use crate::style::{self, defaults};

/// A trait for assigning colors to data series
pub trait Palette {
    /// Get the number of colors in the palette before repeating
    fn len(&self) -> usize;

    /// Get a color from the palette by its index
    fn get(&self, color: IndexColor) -> ColorU8;

    /// Convert the palette into a `Custom` struct
    fn to_custom(&self) -> palette::Custom {
        let mut colors = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            colors.push(self.get(IndexColor(i)));
        }
        palette::Custom(colors)
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
pub mod palette {

    use crate::ColorU8;
    use crate::style::catppuccin;
    use crate::style::series::Palette;

    /// Eidoplot built-in palettes
    #[derive(Debug, Clone, Copy, Default)]
    pub enum Builtin {
        /// Black monochrome palette
        Black,
        #[default]
        /// Standard eidoplot palette
        Standard,
        /// Pastel eidoplot palette
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
    }

    impl Palette for Builtin {
        fn len(&self) -> usize {
            match self {
                Builtin::Black => BLACK.len(),
                Builtin::Standard => STANDARD.len(),
                Builtin::Pastel => PASTEL.len(),
                Builtin::TolBright => TOL_BRIGHT.len(),
                Builtin::OkabeIto => OKABE_ITO.len(),
                Builtin::CatppuccinMocha => catppuccin::Mocha.len(),
                Builtin::CatppuccinMacchiato => catppuccin::Macchiato.len(),
                Builtin::CatppuccinFrappe => catppuccin::Frappe.len(),
                Builtin::CatppuccinLatte => catppuccin::Latte.len(),
            }
        }

        fn get(&self, color: super::IndexColor) -> ColorU8 {
            match self {
                Builtin::Black => BLACK[color.0 % BLACK.len()],
                Builtin::Standard => STANDARD[color.0 % STANDARD.len()],
                Builtin::Pastel => PASTEL[color.0 % PASTEL.len()],
                Builtin::TolBright => TOL_BRIGHT[color.0 % TOL_BRIGHT.len()],
                Builtin::OkabeIto => OKABE_ITO[color.0 % OKABE_ITO.len()],
                Builtin::CatppuccinMocha => catppuccin::Mocha.get(color),
                Builtin::CatppuccinMacchiato => catppuccin::Macchiato.get(color),
                Builtin::CatppuccinFrappe => catppuccin::Frappe.get(color),
                Builtin::CatppuccinLatte => catppuccin::Latte.get(color),
            }
        }
    }

    /// A custom palette
    #[derive(Debug, Clone)]
    pub struct Custom(pub Vec<ColorU8>);

    impl Palette for Custom {
        fn len(&self) -> usize {
            self.0.len()
        }

        fn get(&self, color: super::IndexColor) -> ColorU8 {
            self.0[color.0 % self.len()]
        }
    }

    const BLACK: &[ColorU8] = &[ColorU8::from_html(b"#000000")];
    const STANDARD: &[ColorU8] = &[
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
    const PASTEL: &[ColorU8] = &[
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
    const TOL_BRIGHT: &[ColorU8] = &[
        ColorU8::from_html(b"#4477AA"), // blue
        ColorU8::from_html(b"#EE6677"), // red
        ColorU8::from_html(b"#228833"), // green
        ColorU8::from_html(b"#CCBB44"), // yellow
        ColorU8::from_html(b"#66CCEE"), // cyan
        ColorU8::from_html(b"#AA3377"), // purple
        ColorU8::from_html(b"#BBBBBB"), // gray
    ];
    const OKABE_ITO: &[ColorU8] = &[
        ColorU8::from_html(b"#E69F00"), // orange
        ColorU8::from_html(b"#56B4E9"), // sky blue
        ColorU8::from_html(b"#009E73"), // bluish green
        ColorU8::from_html(b"#F0E442"), // yellow
        ColorU8::from_html(b"#0072B2"), // blue
        ColorU8::from_html(b"#D55E00"), // vermillion
        ColorU8::from_html(b"#CC79A7"), // reddish purple
    ];
}
