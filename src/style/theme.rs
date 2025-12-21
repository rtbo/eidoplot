//! Theme definitions and implementations
use crate::color::{self, ColorU8};
use crate::style::series::{Palette, palettes};
use crate::{style, text};

/// A trait for mapping theme elements to colors
pub trait ThemeMap {
    /// Return true if the theme is dark
    fn is_dark(&self) -> bool {
        self.background().luminance() < 0.5
    }

    /// Get the background color
    fn background(&self) -> ColorU8;

    /// Get the foreground color.
    /// That is, the main text and line color.
    fn foreground(&self) -> ColorU8;

    /// Get the grid line color
    fn grid(&self) -> ColorU8;

    /// Get the legend background fill color
    fn legend_fill(&self) -> ColorU8 {
        self.background().with_opacity(0.5)
    }

    /// Get the legend border color
    fn legend_border(&self) -> ColorU8 {
        self.foreground()
    }

    /// Convert the theme map into a palette
    fn into_palette(self) -> Palette;
}

/// A complete theme definition
#[derive(Debug, Clone)]
pub struct Theme {
    background: ColorU8,
    foreground: ColorU8,
    grid: ColorU8,
    legend_fill: ColorU8,
    legend_border: ColorU8,

    is_dark: bool,
    palette: Palette,
}

impl Default for Theme {
    fn default() -> Self {
        light(Palette::default())
    }
}

impl<M> From<M> for Theme
where
    M: ThemeMap,
{
    fn from(map: M) -> Self {
        Self {
            background: map.background(),
            foreground: map.foreground(),
            grid: map.grid(),
            legend_fill: map.legend_fill(),
            legend_border: map.legend_border(),
            is_dark: map.is_dark(),
            palette: map.into_palette(),
        }
    }
}

impl Theme {
    /// Check if the theme is dark
    pub fn is_dark(&self) -> bool {
        self.is_dark
    }

    /// Get the color for a given theme element
    pub fn get(&self, col: Col) -> ColorU8 {
        match col {
            Col::Background => self.background(),
            Col::Foreground => self.foreground(),
            Col::Grid => self.grid(),
            Col::LegendFill => self.legend_fill(),
            Col::LegendBorder => self.legend_border(),
        }
    }

    /// Get the palette associated with the theme
    pub fn background(&self) -> ColorU8 {
        self.background
    }

    /// Get the foreground color
    pub fn foreground(&self) -> ColorU8 {
        self.foreground
    }

    /// Get the grid line color
    pub fn grid(&self) -> ColorU8 {
        self.grid
    }

    /// Get the legend fill color
    pub fn legend_fill(&self) -> ColorU8 {
        self.legend_fill
    }

    /// Get the legend border color
    pub fn legend_border(&self) -> ColorU8 {
        self.legend_border
    }

    /// Get the palette associated with the theme
    pub fn palette(&self) -> &Palette {
        &self.palette
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

impl super::ResolveColor<Col> for Theme {
    fn resolve_color(&self, color: &Col) -> ColorU8 {
        self.get(*color)
    }
}

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

impl super::ResolveColor<Color> for Theme {
    fn resolve_color(&self, color: &Color) -> ColorU8 {
        match color {
            Color::Theme(col) => self.get(*col),
            Color::Fixed(c) => *c,
        }
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

/// Build the black on white thelme
pub fn black_white() -> Theme {
    Light(palettes::black()).into()
}

/// Build the default light theme
pub fn standard_light() -> Theme {
    Light(palettes::standard()).into()
}

/// Build a light theme with the given palette
pub fn light(palette: Palette) -> Theme {
    Light(palette).into()
}

/// Build the default dark theme
pub fn standard_dark() -> Theme {
    Dark(palettes::pastel()).into()
}

/// Build a dark theme with the given palette
pub fn dark(palette: Palette) -> Theme {
    Dark(palette).into()
}

#[derive(Debug, Clone)]
struct Light(Palette);

impl ThemeMap for Light {
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

    fn into_palette(self) -> Palette {
        self.0
    }
}

#[derive(Debug, Clone)]
struct Dark(Palette);

impl ThemeMap for Dark {
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

    fn into_palette(self) -> Palette {
        self.0
    }
}
