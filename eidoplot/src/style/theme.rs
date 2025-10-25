use crate::style;
use crate::style::color::{self, ColorU8};
use crate::style::series::Palette;

pub trait ThemeMap {
    fn is_dark(&self) -> bool;
    fn background(&self) -> ColorU8;
    fn foreground(&self) -> ColorU8;
    fn grid(&self) -> ColorU8;
    fn legend_fill(&self) -> ColorU8 {
        self.background().with_opacity(0.5)
    }
    fn legend_border(&self) -> ColorU8 {
        self.foreground()
    }

    fn into_palette(self) -> Palette;
}

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
    pub fn is_dark(&self) -> bool {
        self.is_dark
    }

    pub fn get(&self, col: Col) -> ColorU8 {
        match col {
            Col::Background => self.background(),
            Col::Foreground => self.foreground(),
            Col::Grid => self.grid(),
            Col::LegendFill => self.legend_fill(),
            Col::LegendBorder => self.legend_border(),
        }
    }

    pub fn background(&self) -> ColorU8 {
        self.background
    }
    pub fn foreground(&self) -> ColorU8 {
        self.foreground
    }
    pub fn grid(&self) -> ColorU8 {
        self.grid
    }

    pub fn legend_fill(&self) -> ColorU8 {
        self.legend_fill
    }

    pub fn legend_border(&self) -> ColorU8 {
        self.legend_border
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Col {
    Background,
    Foreground,
    Grid,
    LegendFill,
    LegendBorder,
}

impl super::Color for Col {}

impl super::ResolveColor<Col> for Theme {
    fn resolve_color(&self, color: &Col) -> ColorU8 {
        self.get(*color)
    }
}

/// A flexible color for theme elements
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Theme(Col),
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

impl eidoplot_text::rich::Foreground for Color {
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
    Light(style::series::palettes::black()).into()
}

/// Build a light theme with the given palette
pub fn light(palette: Palette) -> Theme {
    Light(palette).into()
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
