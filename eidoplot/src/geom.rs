#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }
    pub fn new_ps(top_left: Point, size: Size) -> Self {
        Rect::new(top_left.x, top_left.y, size.width, size.height)
    }

    pub fn pad(&self, padding: &Padding) -> Self {
        Rect {
            x: self.x + padding.left(),
            y: self.y + padding.top(),
            w: self.w - padding.sum_hor(),
            h: self.h - padding.sum_ver(),
        }
    }
}

/// Padding within a graphical element
#[derive(Debug, Clone, Copy)]
pub enum Padding {
    /// No padding
    None,
    /// Uniform padding in all directions
    Even(f32),
    /// Vertical and horizontal padding
    Center { v: f32, h: f32 },
    /// Top, right, bottom and left padding
    Custom { t: f32, r: f32, b: f32, l: f32 },
}

impl Padding {
    pub fn top(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => *p,
            Padding::Center { v, .. } => *v,
            Padding::Custom { t, .. } => *t,
        }
    }

    pub fn right(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => *p,
            Padding::Center { h, .. } => *h,
            Padding::Custom { r, .. } => *r,
        }
    }

    pub fn bottom(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => *p,
            Padding::Center { v, .. } => *v,
            Padding::Custom { b, .. } => *b,
        }
    }

    pub fn left(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => *p,
            Padding::Center { h, .. } => *h,
            Padding::Custom { l, .. } => *l,
        }
    }

    pub fn sum_ver(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => 2.0 * p,
            Padding::Center { v, .. } => 2.0 * v,
            Padding::Custom { t, b, .. } => t + b,
        }
    }

    pub fn sum_hor(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => 2.0 * p,
            Padding::Center { h, .. } => 2.0 * h,
            Padding::Custom { l, r, .. } => l + r,
        }
    }
}

impl Default for Padding {
    fn default() -> Self {
        Padding::None
    }
}

impl From<()> for Padding {
    fn from(_: ()) -> Self {
        Padding::None
    }
}

impl From<f32> for Padding {
    fn from(value: f32) -> Self {
        Padding::Even(value)
    }
}

impl From<(f32, f32)> for Padding {
    fn from((v, h): (f32, f32)) -> Self {
        Padding::Center { v, h }
    }
}

impl From<(f32, f32, f32, f32)> for Padding {
    fn from((t, r, b, l): (f32, f32, f32, f32)) -> Self {
        Padding::Custom { t, r, b, l }
    }
}
