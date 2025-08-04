pub mod color;

pub use color::Color;

/// Line pattern defines how the line is drawn
#[derive(Debug, Clone, Copy)]
pub enum LinePattern {
    /// Solid line
    Solid,
    /// Dashed line. The pattern is relative to the line width.
    Dash(f32, f32),
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
    pub width: f32,
    pub color: Color,
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

#[derive(Debug, Clone, Copy)]
pub struct Fill {
    pub color: Color,
}

impl From<Color> for Fill {
    fn from(color: Color) -> Self {
        Fill { color }
    }
}
