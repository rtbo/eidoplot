pub mod color;

pub use color::RgbaColor;

pub struct DashPattern {
    pub length: f32,
    pub gap: f32,
}

impl Default for DashPattern {
    fn default() -> Self {
        DashPattern { length: 5.0, gap: 5.0 }
    }
}

/// Line pattern defines how the line is drawn
pub enum LinePattern {
    /// Solid line
    Solid,
    /// Dashed line. The pattern is relative to the line width.
    Dash(DashPattern),
    /// Dotted line. Equivalent to Dash(DashPattern { length: 1.0, gap: 1.0 })
    Dot, 
}

impl Default for LinePattern {
    fn default() -> Self {
        LinePattern::Solid
    }
}

pub struct Line {
    pub width: f32,
    pub color: RgbaColor,
    pub pattern: LinePattern,
}
