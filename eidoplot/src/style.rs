pub mod color;

pub use color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Dash(pub f32, pub f32);

impl Default for Dash {
    fn default() -> Self {
        Dash(5.0, 5.0)
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
pub struct Fill {
    pub color: Color,
}

impl From<Color> for Fill {
    fn from(color: Color) -> Self {
        Fill { color }
    }
}
