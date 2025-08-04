pub use tiny_skia_path::{Path, PathBuilder, PathSegment, Transform};

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
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    pub const fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }
    pub const fn from_trbl(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Rect {
            x: left,
            y: top,
            w: right - left,
            h: bottom - top,
        }
    }
    pub const fn from_ps(top_left: Point, size: Size) -> Self {
        Rect::from_xywh(top_left.x, top_left.y, size.width, size.height)
    }

    pub const fn pad(&self, padding: &Padding) -> Self {
        Rect {
            x: self.x + padding.left(),
            y: self.y + padding.top(),
            w: self.w - padding.sum_hor(),
            h: self.h - padding.sum_ver(),
        }
    }

    pub const fn x(&self) -> f32 {
        self.x
    }

    pub const fn y(&self) -> f32 {
        self.y
    }

    pub const fn w(&self) -> f32 {
        self.w
    }

    pub const fn h(&self) -> f32 {
        self.h
    }

    pub const fn top(&self) -> f32 {
        self.y
    }

    pub const fn right(&self) -> f32 {
        self.x + self.w
    }

    pub const fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub const fn left(&self) -> f32 {
        self.x
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
    pub const fn top(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => *p,
            Padding::Center { v, .. } => *v,
            Padding::Custom { t, .. } => *t,
        }
    }

    pub const fn right(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => *p,
            Padding::Center { h, .. } => *h,
            Padding::Custom { r, .. } => *r,
        }
    }

    pub const fn bottom(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => *p,
            Padding::Center { v, .. } => *v,
            Padding::Custom { b, .. } => *b,
        }
    }

    pub const fn left(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => *p,
            Padding::Center { h, .. } => *h,
            Padding::Custom { l, .. } => *l,
        }
    }

    pub const fn sum_ver(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => *p * 2.0,
            Padding::Center { v, .. } => *v * 2.0,
            Padding::Custom { t, b, .. } => *t + *b,
        }
    }

    pub const fn sum_hor(&self) -> f32 {
        match self {
            Padding::None => 0.0,
            Padding::Even(p) => *p * 2.0,
            Padding::Center { h, .. } => *h * 2.0,
            Padding::Custom { l, r, .. } => *l + *r,
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
