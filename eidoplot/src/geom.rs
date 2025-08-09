pub use tiny_skia_path::{Path, PathBuilder, PathSegment, Transform};

#[derive(Debug, Clone, Copy)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }

    pub const fn x(&self) -> f32 {
        self.x
    }

    pub const fn y(&self) -> f32 {
        self.y
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    w: f32,
    h: f32,
}

impl Size {
    pub const fn new(w: f32, h: f32) -> Self {
        Size { w, h }
    }

    pub const fn width(&self) -> f32 {
        self.w
    }

    pub const fn height(&self) -> f32 {
        self.h
    }
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
        Rect::from_xywh(top_left.x, top_left.y, size.w, size.h)
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

    pub const fn center(&self) -> Point {
        Point {
            x: self.center_x(),
            y: self.center_y(),
        }
    }

    pub const fn center_x(&self) -> f32 {
        self.x() + self.width() / 2.0
    }

    pub const fn center_y(&self) -> f32 {
        self.y() + self.height() / 2.0
    }

    pub const fn width(&self) -> f32 {
        self.w
    }

    pub const fn height(&self) -> f32 {
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

    pub const fn shifted_top_side(&self, shift: f32) -> Rect {
        Rect {
            x: self.x,
            y: self.y + shift,
            w: self.w,
            h: self.h - shift,
        }
    }

    pub const fn shifted_right_side(&self, shift: f32) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            w: self.w + shift,
            h: self.h,
        }
    }

    pub const fn shifted_bottom_side(&self, shift: f32) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h + shift,
        }
    }

    pub const fn shifted_left_side(&self, shift: f32) -> Rect {
        Rect {
            x: self.x + shift,
            y: self.y,
            w: self.w - shift,
            h: self.h,
        }
    }

    pub const fn shift_top_side(&mut self, shift: f32) {
        self.y += shift;
        self.h -= shift;
    }

    pub const fn shift_right_side(&mut self, shift: f32) {
        self.w += shift;
    }

    pub const fn shift_bottom_side(&mut self, shift: f32) {
        self.h += shift;
    }

    pub const fn shift_left_side(&mut self, shift: f32) {
        self.x += shift;
        self.w -= shift;
    }

    pub const fn with_top(self, top: f32) -> Rect {
        Rect {
            y: top,
            h: self.bottom() - top,
            ..self
        }
    }

    pub const fn with_right(self, right: f32) -> Rect {
        Rect {
            w: right - self.x,
            ..self
        }
    }

    pub const fn with_bottom(self, bottom: f32) -> Rect {
        Rect {
            h: bottom - self.y,
            ..self
        }
    }

    pub const fn with_left(self, left: f32) -> Rect {
        Rect {
            x: left,
            w: self.right() - left,
            ..self
        }
    }

    pub fn to_path(&self) -> Path {
        PathBuilder::from_rect(
            tiny_skia_path::Rect::from_xywh(self.x, self.y, self.w, self.h).unwrap(),
        )
    }
}

/// Padding within a graphical element
#[derive(Debug, Clone, Copy)]
pub enum Padding {
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
            Padding::Even(p) => *p,
            Padding::Center { v, .. } => *v,
            Padding::Custom { t, .. } => *t,
        }
    }

    pub const fn right(&self) -> f32 {
        match self {
            Padding::Even(p) => *p,
            Padding::Center { h, .. } => *h,
            Padding::Custom { r, .. } => *r,
        }
    }

    pub const fn bottom(&self) -> f32 {
        match self {
            Padding::Even(p) => *p,
            Padding::Center { v, .. } => *v,
            Padding::Custom { b, .. } => *b,
        }
    }

    pub const fn left(&self) -> f32 {
        match self {
            Padding::Even(p) => *p,
            Padding::Center { h, .. } => *h,
            Padding::Custom { l, .. } => *l,
        }
    }

    pub const fn sum_ver(&self) -> f32 {
        match self {
            Padding::Even(p) => *p * 2.0,
            Padding::Center { v, .. } => *v * 2.0,
            Padding::Custom { t, b, .. } => *t + *b,
        }
    }

    pub const fn sum_hor(&self) -> f32 {
        match self {
            Padding::Even(p) => *p * 2.0,
            Padding::Center { h, .. } => *h * 2.0,
            Padding::Custom { l, r, .. } => *l + *r,
        }
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
