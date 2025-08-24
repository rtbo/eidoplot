pub mod color;
pub mod defaults;

pub use color::Color;

pub mod font {
    pub use eidoplot_text::font::{Family, Font, Style, Weight, Width, parse_font_families};
}

pub use font::Font;

#[derive(Debug, Clone, Copy)]
pub struct Dash(pub f32, pub f32);

impl Default for Dash {
    fn default() -> Self {
        defaults::DASH_PATTERN
    }
}

/// Line pattern defines how the line is drawn
#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub color: Color,
    pub width: f32,
    pub pattern: LinePattern,
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

impl From<Color> for Fill {
    fn from(color: Color) -> Self {
        Fill::Solid(color)
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

#[derive(Debug, Clone, Copy)]
pub struct Marker {
    pub size: MarkerSize,
    pub shape: MarkerShape,
    pub fill: Option<Fill>,
    pub stroke: Option<Line>,
}

