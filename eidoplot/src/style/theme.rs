use crate::style;
use crate::style::color::{self, ColorU8};
use crate::style::series::Palette;

pub trait Theme {
    type Palette: Palette;

    fn is_dark(&self) -> bool;

    fn get(&self, col: Col) -> ColorU8 {
        match col {
            Col::Background => self.background(),
            Col::Foreground => self.foreground(),
            Col::Grid => self.grid(),
            Col::LegendFill => self.legend_fill(),
            Col::LegendBorder => self.legend_border(),
        }
    }

    fn background(&self) -> ColorU8;
    fn foreground(&self) -> ColorU8;
    fn grid(&self) -> ColorU8;

    fn legend_fill(&self) -> ColorU8 {
        self.background().with_opacity(0.5)
    }

    fn legend_border(&self) -> ColorU8 {
        self.foreground()
    }

    fn palette(&self) -> &Self::Palette;
}

#[derive(Debug, Clone, Copy)]
pub enum Col {
    Background,
    Foreground,
    Grid,
    LegendFill,
    LegendBorder,
}

impl super::Color for Col {}

impl<T, P> super::ResolveColor<Col> for T
where
    T: Theme<Palette = P>,
    P: Palette,
{
    fn resolve_color(&self, color: &Col) -> ColorU8 {
        self.get(*color)
    }
}

/// A flexible color for theme elements
#[derive(Debug, Clone, Copy)]
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

impl<T, P> super::ResolveColor<Color> for T
where
    T: Theme<Palette = P>,
    P: Palette,
{
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

#[derive(Debug, Clone, Copy)]
pub struct Light<P: Palette> {
    pub palette: P,
}

impl<P: Palette> Light<P> {
    pub fn new(palette: P) -> Self {
        Light { palette }
    }
}

impl<P: Palette> Theme for Light<P> {
    type Palette = P;

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

    fn palette(&self) -> &Self::Palette {
        &self.palette
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Dark<P: Palette> {
    pub palette: P,
}

impl<P: Palette> Dark<P> {
    pub fn new(palette: P) -> Self {
        Dark { palette }
    }
}

impl<P: Palette> Theme for Dark<P> {
    type Palette = P;

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

    fn palette(&self) -> &Self::Palette {
        &self.palette
    }
}
