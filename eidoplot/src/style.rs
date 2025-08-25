pub mod color;
pub(crate) mod defaults;
pub mod palette;
pub mod theme;

pub use color::ColorU8;

pub mod font {
    pub use eidoplot_text::font::{Family, Font, Style, Weight, Width, parse_font_families};
}

pub use font::Font;
pub use palette::Palette;
pub use theme::Theme;

use crate::render;

/// A color that can be either from a theme or palette or a fixed color
#[derive(Debug, Clone, Copy)]
pub enum Color {
    /// Color from the current series palette
    Palette(palette::Color),
    /// Color from the current theme
    Theme(theme::Color),
    /// A fixed color
    Fixed(ColorU8),
}

impl Color {
    pub fn resolve<T>(&self, theme: &T) -> ColorU8
    where
        T: theme::Theme,
    {
        match self {
            Color::Palette(col) => theme.palette().get(*col),
            Color::Theme(col) => theme.get(*col),
            Color::Fixed(col) => *col,
        }
    }
}

impl From<palette::Color> for Color {
    fn from(col: palette::Color) -> Self {
        Color::Palette(col)
    }
}

impl From<theme::Color> for Color {
    fn from(col: theme::Color) -> Self {
        Color::Theme(col)
    }
}

impl From<ColorU8> for Color {
    fn from(color: ColorU8) -> Self {
        Color::Fixed(color)
    }
}

#[derive(Debug, Clone)]
pub struct Dash(Vec<f32>);

impl Default for Dash {
    fn default() -> Self {
        Dash(vec![5.0, 5.0])
    }
}

/// Line pattern defines how the line is drawn
#[derive(Debug, Clone)]
pub enum LinePattern {
    /// Solid line
    Solid,
    /// Dashed line. The pattern is relative to the line width.
    Dash(Dash),
    /// Dotted line. Equivalent to Dash(1.0, 1.0)
    Dot,
}

impl Default for LinePattern {
    fn default() -> Self {
        LinePattern::Solid
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pub color: Color,
    pub width: f32,
    pub pattern: LinePattern,
}

const DOT_DASH: &[f32] = &[1.0, 1.0];

impl Line {
    pub fn as_stroke<'a, T>(&'a self, theme: &T) -> render::Stroke<'a>
    where
        T: Theme,
    {
        let pattern = match &self.pattern {
            LinePattern::Solid => render::LinePattern::Solid,
            LinePattern::Dash(Dash(a)) => render::LinePattern::Dash(a.as_slice()),
            LinePattern::Dot => render::LinePattern::Dash(DOT_DASH),
        };

        render::Stroke {
            color: self.color.resolve(theme),
            width: self.width,
            pattern,
        }
    }
}

impl From<Color> for Line {
    fn from(color: Color) -> Self {
        Line {
            width: 1.0,
            color,
            pattern: LinePattern::default(),
        }
    }
}

impl From<theme::Color> for Line {
    fn from(color: theme::Color) -> Self {
        Line {
            width: 1.0,
            color: color.into(),
            pattern: LinePattern::default(),
        }
    }
}

impl From<(Color, f32)> for Line {
    fn from((color, width): (Color, f32)) -> Self {
        Line {
            color,
            width,
            pattern: LinePattern::default(),
        }
    }
}

impl From<(Color, f32, LinePattern)> for Line {
    fn from((color, width, pattern): (Color, f32, LinePattern)) -> Self {
        Line {
            color,
            width,
            pattern,
        }
    }
}

impl From<(Color, f32, Dash)> for Line {
    fn from((color, width, dash): (Color, f32, Dash)) -> Self {
        Line {
            color,
            width,
            pattern: LinePattern::Dash(dash),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Fill {
    Solid(Color),
}

impl Fill {
    pub fn as_paint<T>(&self, theme: &T) -> render::Paint
    where
        T: Theme,
    {
        match self {
            Fill::Solid(c) => render::Paint::Solid(c.resolve(theme)),
        }
    }
}

impl From<Color> for Fill {
    fn from(color: Color) -> Self {
        Fill::Solid(color)
    }
}

impl From<palette::Color> for Fill {
    fn from(color: palette::Color) -> Self {
        Fill::Solid(color.into())
    }
}

impl From<theme::Color> for Fill {
    fn from(color: theme::Color) -> Self {
        Fill::Solid(color.into())
    }
}

impl From<ColorU8> for Fill {
    fn from(color: ColorU8) -> Self {
        Fill::Solid(color.into())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum MarkerShape {
    #[default]
    Circle,
    Square,
    Diamond,
    Cross,
    Plus,
    TriangleUp,
    TriangleDown,
}

#[derive(Debug, Clone, Copy)]
pub struct MarkerSize(pub f32);

impl Default for MarkerSize {
    fn default() -> Self {
        MarkerSize(defaults::MARKER_SIZE)
    }
}

impl From<f32> for MarkerSize {
    fn from(size: f32) -> Self {
        MarkerSize(size)
    }
}

#[derive(Debug, Clone)]
pub struct Marker {
    pub size: MarkerSize,
    pub shape: MarkerShape,
    pub fill: Option<Fill>,
    pub stroke: Option<Line>,
}
