//! Theme definitions and implementations

use crate::color::{self, ColorU8};
use crate::style::catppuccin;
use crate::{style, text};

/// A trait for theming figures
pub trait Theme {
    /// Return true if the theme is dark
    fn is_dark(&self) -> bool {
        self.background().luminance() < 0.5
    }

    /// Get the theme background color
    fn background(&self) -> ColorU8;

    /// Get the theme foreground color
    /// That is, the main text and line color.
    fn foreground(&self) -> ColorU8;

    /// Get the theme grid line color
    fn grid(&self) -> ColorU8;

    /// Get the legend background fill color
    fn legend_fill(&self) -> ColorU8 {
        self.background().with_opacity(0.5)
    }

    /// Get the legend border color
    fn legend_border(&self) -> ColorU8 {
        self.foreground()
    }

    /// Convert the theme into a Custom theme
    fn to_custom(&self) -> Custom {
        Custom {
            background: self.background(),
            foreground: self.foreground(),
            grid: self.grid(),
            legend_fill: self.legend_fill(),
            legend_border: self.legend_border(),
        }
    }
}

/// Eidoplot built-in themes
#[derive(Debug, Clone, Copy, Default)]
pub enum Builtin {
    #[default]
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// Catppuccin Mocha theme
    CatppuccinMocha,
    /// Catppuccin Macchiato theme
    CatppuccinMacchiato,
    /// Catppuccin Frappe theme
    CatppuccinFrappe,
    /// Catppuccin Latte theme
    CatppuccinLatte,
}

impl Theme for Builtin {
    fn background(&self) -> ColorU8 {
        match self {
            Builtin::Light => Light.background(),
            Builtin::Dark => Dark.background(),
            Builtin::CatppuccinMocha => catppuccin::Mocha.background(),
            Builtin::CatppuccinMacchiato => catppuccin::Macchiato.background(),
            Builtin::CatppuccinFrappe => catppuccin::Frappe.background(),
            Builtin::CatppuccinLatte => catppuccin::Latte.background(),
        }
    }

    fn foreground(&self) -> ColorU8 {
        match self {
            Builtin::Light => Light.foreground(),
            Builtin::Dark => Dark.foreground(),
            Builtin::CatppuccinMocha => catppuccin::Mocha.foreground(),
            Builtin::CatppuccinMacchiato => catppuccin::Macchiato.foreground(),
            Builtin::CatppuccinFrappe => catppuccin::Frappe.foreground(),
            Builtin::CatppuccinLatte => catppuccin::Latte.foreground(),
        }
    }

    fn grid(&self) -> ColorU8 {
        match self {
            Builtin::Light => Light.grid(),
            Builtin::Dark => Dark.grid(),
            Builtin::CatppuccinMocha => catppuccin::Mocha.grid(),
            Builtin::CatppuccinMacchiato => catppuccin::Macchiato.grid(),
            Builtin::CatppuccinFrappe => catppuccin::Frappe.grid(),
            Builtin::CatppuccinLatte => catppuccin::Latte.grid(),
        }
    }

    fn legend_fill(&self) -> ColorU8 {
        match self {
            Builtin::Light => Light.legend_fill(),
            Builtin::Dark => Dark.legend_fill(),
            Builtin::CatppuccinMocha => catppuccin::Mocha.legend_fill(),
            Builtin::CatppuccinMacchiato => catppuccin::Macchiato.legend_fill(),
            Builtin::CatppuccinFrappe => catppuccin::Frappe.legend_fill(),
            Builtin::CatppuccinLatte => catppuccin::Latte.legend_fill(),
        }
    }

    fn legend_border(&self) -> ColorU8 {
        match self {
            Builtin::Light => Light.legend_border(),
            Builtin::Dark => Dark.legend_border(),
            Builtin::CatppuccinMocha => catppuccin::Mocha.legend_border(),
            Builtin::CatppuccinMacchiato => catppuccin::Macchiato.legend_border(),
            Builtin::CatppuccinFrappe => catppuccin::Frappe.legend_border(),
            Builtin::CatppuccinLatte => catppuccin::Latte.legend_border(),
        }
    }
}

/// A custom theme definition
#[derive(Debug, Clone, Copy)]
pub struct Custom {
    /// Background color
    pub background: ColorU8,
    /// Foreground color
    pub foreground: ColorU8,
    /// Grid line color
    pub grid: ColorU8,
    /// Legend background fill color
    pub legend_fill: ColorU8,
    /// Legend border color
    pub legend_border: ColorU8,
}

impl Default for Custom {
    fn default() -> Self {
        Builtin::default().to_custom()
    }
}

impl Custom {
    /// Create a new custom theme from background and foreground colors
    /// The grid, legend fill and legend border colors are derived automatically.
    pub fn new_back_and_fore(background: ColorU8, foreground: ColorU8) -> Self {
        let grid = if background.luminance() < 0.5 {
            // Dark background
            ColorU8::from_rgb(192, 192, 192).with_opacity(0.6)
        } else {
            // Light background
            ColorU8::from_rgb(128, 128, 128).with_opacity(0.6)
        };

        let legend_fill = background.with_opacity(0.5);
        let legend_border = foreground;

        Self {
            background,
            foreground,
            grid,
            legend_fill,
            legend_border,
        }
    }
}

impl Theme for Custom {
    fn background(&self) -> ColorU8 {
        self.background
    }

    fn foreground(&self) -> ColorU8 {
        self.foreground
    }

    fn grid(&self) -> ColorU8 {
        self.grid
    }

    fn legend_fill(&self) -> ColorU8 {
        self.legend_fill
    }

    fn legend_border(&self) -> ColorU8 {
        self.legend_border
    }
}

/// Predefined colors for theme elements
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Col {
    /// Background color
    Background,
    /// Foreground color
    Foreground,
    /// Grid line color
    Grid,
    /// Legend background fill color
    LegendFill,
    /// Legend border color
    LegendBorder,
}

impl super::Color for Col {}

impl std::str::FromStr for Col {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "background" => Ok(Col::Background),
            "foreground" => Ok(Col::Foreground),
            "grid" => Ok(Col::Grid),
            "legend_fill" => Ok(Col::LegendFill),
            "legend_border" => Ok(Col::LegendBorder),
            _ => Err(()),
        }
    }
}

/// A flexible color for theme elements
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    /// A color from the theme
    Theme(Col),
    /// A fixed RGB color
    Fixed(ColorU8),
}

impl From<Col> for Color {
    fn from(color: Col) -> Self {
        Color::Theme(color)
    }
}

impl From<ColorU8> for Color {
    fn from(color: ColorU8) -> Self {
        Color::Fixed(color)
    }
}

impl super::Color for Color {}

impl std::str::FromStr for Color {
    type Err = <ColorU8 as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(col) = s.parse::<Col>() {
            Ok(Color::Theme(col))
        } else {
            let c = s.parse::<ColorU8>()?;
            Ok(Color::Fixed(c))
        }
    }
}

impl text::rich::Foreground for Color {
    fn foreground() -> Self {
        Color::Theme(Col::Foreground)
    }
}

/// Line style for theme elements
pub type Line = style::Line<Color>;

// From<Color> for Line is already defined in style.rs, using generics.
// We just add From<Col> for Line here.
impl From<Col> for Line {
    fn from(col: Col) -> Self {
        Line {
            color: col.into(),
            width: 1.0,
            pattern: style::LinePattern::default(),
            opacity: None,
        }
    }
}

/// Fill style for theme elements
pub type Fill = style::Fill<Color>;

// From<Color> for Fill is already defined in style.rs, using generics.
// We just add From<Col> for Fill here.
impl From<Col> for Fill {
    fn from(col: Col) -> Self {
        Fill::Solid {
            color: col.into(),
            opacity: None,
        }
    }
}

/// Marker style for theme elements
pub type Marker = style::Marker<Color>;

impl From<Col> for Marker {
    fn from(col: Col) -> Self {
        Marker {
            size: Default::default(),
            shape: Default::default(),
            fill: Some(Fill::Solid {
                color: col.into(),
                opacity: None,
            }),
            stroke: None,
        }
    }
}

impl Default for Marker {
    fn default() -> Self {
        Marker {
            size: Default::default(),
            shape: Default::default(),
            fill: Some(Fill::Solid {
                color: Col::Foreground.into(),
                opacity: None,
            }),
            stroke: None,
        }
    }
}

#[derive(Debug, Clone)]
struct Light;

impl Theme for Light {
    fn is_dark(&self) -> bool {
        false
    }

    fn background(&self) -> ColorU8 {
        color::WHITE
    }

    fn foreground(&self) -> ColorU8 {
        color::BLACK
    }

    fn grid(&self) -> ColorU8 {
        ColorU8::from_rgb(128, 128, 128).with_opacity(0.6)
    }

    fn legend_fill(&self) -> ColorU8 {
        ColorU8::from_rgba(255, 255, 255, 128)
    }

    fn legend_border(&self) -> ColorU8 {
        ColorU8::from_rgb(0, 0, 0)
    }
}

#[derive(Debug, Clone)]
struct Dark;

impl Theme for Dark {
    fn is_dark(&self) -> bool {
        true
    }

    fn background(&self) -> ColorU8 {
        ColorU8::from_html(b"#1e1e2e")
    }

    fn foreground(&self) -> ColorU8 {
        color::WHITE
    }

    fn grid(&self) -> ColorU8 {
        ColorU8::from_rgb(192, 192, 192).with_opacity(0.6)
    }

    fn legend_fill(&self) -> ColorU8 {
        self.background().with_opacity(0.5)
    }

    fn legend_border(&self) -> ColorU8 {
        ColorU8::from_rgb(255, 255, 255)
    }
}
