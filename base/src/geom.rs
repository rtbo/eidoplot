/*!
 * Geometric primitives.
 *
 * Paths and transforms are publicly imported from tiny-skia-path.
 *
 * Y low coordinates are at the top.
 */

use strict_num::{FiniteF32, PositiveF32};
pub use tiny_skia_path::{Path, PathBuilder, PathSegment, PathVerb, Point, Transform};

/// A size in 2D space reprensented by width and height
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    w: f32,
    h: f32,
}

impl Size {
    /// Build a size from width and height
    pub const fn new(w: f32, h: f32) -> Self {
        Size { w, h }
    }

    /// The width
    pub const fn width(&self) -> f32 {
        self.w
    }

    /// The height
    pub const fn height(&self) -> f32 {
        self.h
    }

    /// Expand width and height by dw and dh
    pub const fn expand(&self, dw: f32, dh: f32) -> Size {
        Size {
            w: self.w + dw,
            h: self.h + dh,
        }
    }
}

/// A rectangle in 2D space reprensented by x, y, width and height
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    x: FiniteF32,
    y: FiniteF32,
    w: PositiveF32,
    h: PositiveF32,
}

impl Rect {
    pub fn null() -> Self {
        Rect {
            x: FiniteF32::new(0.0).unwrap(),
            y: FiniteF32::new(0.0).unwrap(),
            w: PositiveF32::new(0.0).unwrap(),
            h: PositiveF32::new(0.0).unwrap(),
        }
    }

    /// Build a rectangle from x, y, width and height
    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect {
            x: FiniteF32::new(x).unwrap(),
            y: FiniteF32::new(y).unwrap(),
            w: PositiveF32::new(w).unwrap(),
            h: PositiveF32::new(h).unwrap(),
        }
    }

    /// Build a rectangle from top, right, bottom and left
    pub fn from_trbl(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Rect {
            x: FiniteF32::new(left).unwrap(),
            y: FiniteF32::new(top).unwrap(),
            w: PositiveF32::new(right - left).unwrap(),
            h: PositiveF32::new(bottom - top).unwrap(),
        }
    }

    /// Build a rectnagle from top left point and size
    pub fn from_ps(top_left: Point, size: Size) -> Self {
        Rect::from_xywh(top_left.x, top_left.y, size.w, size.h)
    }

    /// Build a rectangle from two corner points
    pub fn from_corners(p1: Point, p2: Point) -> Self {
        Rect::from_trbl(
            p1.y.min(p2.y),
            p1.x.max(p2.x),
            p1.y.max(p2.y),
            p1.x.min(p2.x),
        )
    }

    /// Pad the rectangle, removing padding from 4 sides
    pub fn pad(&self, padding: &Padding) -> Self {
        Rect {
            x: FiniteF32::new(self.x.get() + padding.left()).unwrap(),
            y: FiniteF32::new(self.y.get() + padding.top()).unwrap(),
            w: PositiveF32::new(self.w.get() - padding.sum_hor()).unwrap(),
            h: PositiveF32::new(self.h.get() - padding.sum_ver()).unwrap(),
        }
    }

    /// The top-left point of the rectangle
    pub const fn top_left(&self) -> Point {
        Point {
            x: self.left(),
            y: self.top(),
        }
    }

    /// The top-right point of the rectangle
    pub const fn top_right(&self) -> Point {
        Point {
            x: self.right(),
            y: self.top(),
        }
    }

    /// The bottom-right point of the rectangle
    pub const fn bottom_right(&self) -> Point {
        Point {
            x: self.right(),
            y: self.bottom(),
        }
    }

    /// The bottom-left point of the rectangle
    pub const fn bottom_left(&self) -> Point {
        Point {
            x: self.left(),
            y: self.bottom(),
        }
    }

    /// The size of the rectangle
    pub const fn size(&self) -> Size {
        Size {
            w: self.width(),
            h: self.height(),
        }
    }

    /// The X coordinate of the left side
    pub const fn x(&self) -> f32 {
        self.x.get()
    }

    /// The Y coordinate of the top side
    pub const fn y(&self) -> f32 {
        self.y.get()
    }

    /// The center point of the rectangle
    pub const fn center(&self) -> Point {
        Point {
            x: self.center_x(),
            y: self.center_y(),
        }
    }

    /// The horizontal center X coordinate
    pub const fn center_x(&self) -> f32 {
        self.x() + self.width() / 2.0
    }

    /// The vertical center Y coordinate
    pub const fn center_y(&self) -> f32 {
        self.y() + self.height() / 2.0
    }

    /// The width of the rectangle
    pub const fn width(&self) -> f32 {
        self.w.get()
    }

    /// The height of the rectangle
    pub const fn height(&self) -> f32 {
        self.h.get()
    }

    /// The top Y coordinate
    pub const fn top(&self) -> f32 {
        self.y.get()
    }

    /// The right X coordinate
    pub const fn right(&self) -> f32 {
        self.x.get() + self.w.get()
    }

    /// The bottom Y coordinate
    pub const fn bottom(&self) -> f32 {
        self.y.get() + self.h.get()
    }

    /// The left X coordinate
    pub const fn left(&self) -> f32 {
        self.x.get()
    }

    /// Set the top Y coordinate
    pub fn set_top(&mut self, top: f32) {
        self.y = FiniteF32::new(top).unwrap();
    }

    /// Set the right X coordinate
    pub fn set_right(&mut self, right: f32) {
        self.w = PositiveF32::new(right - self.x()).unwrap();
    }

    /// Set the bottom Y coordinate
    pub fn set_bottom(&mut self, bottom: f32) {
        self.h = PositiveF32::new(bottom - self.y()).unwrap();
    }

    /// Set the left X coordinate
    pub fn set_left(&mut self, left: f32) {
        self.x = FiniteF32::new(left).unwrap();
    }

    /// Shift the top side down by shift
    pub fn shifted_top_side(&self, shift: f32) -> Rect {
        Rect {
            x: self.x,
            y: FiniteF32::new(self.y.get() + shift).unwrap(),
            w: self.w,
            h: PositiveF32::new(self.h.get() - shift).unwrap(),
        }
    }

    /// Shift the right side right by shift
    pub fn shifted_right_side(&self, shift: f32) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            w: PositiveF32::new(self.w.get() + shift).unwrap(),
            h: self.h,
        }
    }

    /// Shift the bottom side down by shift
    pub fn shifted_bottom_side(&self, shift: f32) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: PositiveF32::new(self.h.get() + shift).unwrap(),
        }
    }

    /// Shift the left side right by shift
    pub fn shifted_left_side(&self, shift: f32) -> Rect {
        Rect {
            x: FiniteF32::new(self.x.get() + shift).unwrap(),
            y: self.y,
            w: PositiveF32::new(self.w.get() - shift).unwrap(),
            h: self.h,
        }
    }

    /// Shift the top side down by shift (in-place)
    pub fn shift_top_side(&mut self, shift: f32) {
        self.y = FiniteF32::new(self.y.get() + shift).unwrap();
        self.h = PositiveF32::new(self.h.get() - shift).unwrap();
    }

    /// Shift the right side right by shift (in-place)
    pub fn shift_right_side(&mut self, shift: f32) {
        self.w = PositiveF32::new(self.w.get() + shift).unwrap();
    }

    /// Shift the bottom side down by shift (in-place)
    pub fn shift_bottom_side(&mut self, shift: f32) {
        self.h = PositiveF32::new(self.h.get() + shift).unwrap();
    }

    /// Shift the left side right by shift (in-place)
    pub fn shift_left_side(&mut self, shift: f32) {
        self.x = FiniteF32::new(self.x.get() + shift).unwrap();
        self.w = PositiveF32::new(self.w.get() - shift).unwrap();
    }

    /// Build a copy of the rect with a new top side
    pub fn with_top(self, top: f32) -> Rect {
        let new_h = self.bottom() - top;
        Rect {
            y: FiniteF32::new(top).unwrap(),
            h: PositiveF32::new(new_h).unwrap(),
            ..self
        }
    }

    /// Build a copy of the rect with a new right side
    pub fn with_right(self, right: f32) -> Rect {
        let new_w = right - self.x();
        Rect {
            w: PositiveF32::new(new_w).unwrap(),
            ..self
        }
    }

    /// Build a copy of the rect with a new bottom side
    pub fn with_bottom(self, bottom: f32) -> Rect {
        let new_h = bottom - self.y();
        Rect {
            h: PositiveF32::new(new_h).unwrap(),
            ..self
        }
    }

    /// Build a copy of the rect with a new left side
    pub fn with_left(self, left: f32) -> Rect {
        let new_w = self.right() - left;
        Rect {
            x: FiniteF32::new(left).unwrap(),
            w: PositiveF32::new(new_w).unwrap(),
            ..self
        }
    }

    /// Unite two rectangles into one that contains both
    pub fn unite(r1: &Rect, r2: &Rect) -> Rect {
        let left = r1.left().min(r2.left());
        let top = r1.top().min(r2.top());
        let right = r1.right().max(r2.right());
        let bottom = r1.bottom().max(r2.bottom());
        Rect::from_trbl(top, right, bottom, left)
    }

    /// Unite two optional rectangles into one that contains both
    pub fn unite_opt(r1: Option<&Rect>, r2: Option<&Rect>) -> Option<Rect> {
        match (r1, r2) {
            (Some(r1), Some(r2)) => Some(Rect::unite(r1, r2)),
            (Some(r1), None) => Some(*r1),
            (None, Some(r2)) => Some(*r2),
            (None, None) => None,
        }
    }

    /// Transform the rectangle with a transform
    pub fn transform(self, transform: &Transform) -> Rect {
        let mut tlbr = [
            Point {
                x: self.left(),
                y: self.top(),
            },
            Point {
                x: self.right(),
                y: self.bottom(),
            },
        ];
        transform.map_points(&mut tlbr);

        let [p1, p2] = tlbr;
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p2.x - p1.x).abs();
        let height = (p2.y - p1.y).abs();

        Rect::from_xywh(x, y, width, height)
    }

    /// Translate the rectangle by dx and dy
    pub fn translate(&self, dx: f32, dy: f32) -> Rect {
        Rect {
            x: FiniteF32::new(self.x.get() + dx).unwrap(),
            y: FiniteF32::new(self.y.get() + dy).unwrap(),
            w: self.w,
            h: self.h,
        }
    }

    /// Test if the rectangle contains a point
    pub fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    /// Build a path from the rectangle
    pub fn to_path(&self) -> Path {
        PathBuilder::from_rect(
            tiny_skia_path::Rect::from_xywh(self.x.get(), self.y.get(), self.w.get(), self.h.get())
                .unwrap(),
        )
    }
}

/// Padding within a graphical element
#[derive(Debug, Clone, Copy)]
pub enum Padding {
    /// Uniform padding in all directions
    Even(f32),
    /// Vertical and horizontal padding
    Center {
        /// Vertical padding
        v: f32,
        /// Horizontal padding
        h: f32,
    },
    /// Top, right, bottom and left padding
    Custom {
        /// Top padding
        t: f32,
        /// Right padding
        r: f32,
        /// Bottom padding
        b: f32,
        /// Left padding
        l: f32,
    },
}

impl Padding {
    /// The top padding
    pub const fn top(&self) -> f32 {
        match self {
            Padding::Even(p) => *p,
            Padding::Center { v, .. } => *v,
            Padding::Custom { t, .. } => *t,
        }
    }

    /// The right padding
    pub const fn right(&self) -> f32 {
        match self {
            Padding::Even(p) => *p,
            Padding::Center { h, .. } => *h,
            Padding::Custom { r, .. } => *r,
        }
    }

    /// The bottom padding
    pub const fn bottom(&self) -> f32 {
        match self {
            Padding::Even(p) => *p,
            Padding::Center { v, .. } => *v,
            Padding::Custom { b, .. } => *b,
        }
    }

    /// The left padding
    pub const fn left(&self) -> f32 {
        match self {
            Padding::Even(p) => *p,
            Padding::Center { h, .. } => *h,
            Padding::Custom { l, .. } => *l,
        }
    }

    /// The total vertical padding
    pub const fn sum_ver(&self) -> f32 {
        match self {
            Padding::Even(p) => *p * 2.0,
            Padding::Center { v, .. } => *v * 2.0,
            Padding::Custom { t, b, .. } => *t + *b,
        }
    }

    /// The total horizontal padding
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

/// Margin around a graphical element
#[derive(Debug, Clone, Copy)]
pub enum Margin {
    /// Uniform margin in all directions
    Even(f32),
    /// Vertical and horizontal margin
    Center {
        /// Vertical margin
        v: f32,
        /// Horizontal margin
        h: f32,
    },
    /// Top, right, bottom and left margin
    Custom {
        /// Top margin
        t: f32,
        /// Right margin
        r: f32,
        /// Bottom margin
        b: f32,
        /// Left margin
        l: f32,
    },
}

impl Margin {
    /// The top margin
    pub const fn top(&self) -> f32 {
        match self {
            Margin::Even(p) => *p,
            Margin::Center { v, .. } => *v,
            Margin::Custom { t, .. } => *t,
        }
    }

    /// The right margin
    pub const fn right(&self) -> f32 {
        match self {
            Margin::Even(p) => *p,
            Margin::Center { h, .. } => *h,
            Margin::Custom { r, .. } => *r,
        }
    }

    /// The bottom margin
    pub const fn bottom(&self) -> f32 {
        match self {
            Margin::Even(p) => *p,
            Margin::Center { v, .. } => *v,
            Margin::Custom { b, .. } => *b,
        }
    }

    /// The left margin
    pub const fn left(&self) -> f32 {
        match self {
            Margin::Even(p) => *p,
            Margin::Center { h, .. } => *h,
            Margin::Custom { l, .. } => *l,
        }
    }

    /// The total vertical margin
    pub const fn sum_ver(&self) -> f32 {
        match self {
            Margin::Even(p) => *p * 2.0,
            Margin::Center { v, .. } => *v * 2.0,
            Margin::Custom { t, b, .. } => *t + *b,
        }
    }

    /// The total horizontal margin
    pub const fn sum_hor(&self) -> f32 {
        match self {
            Margin::Even(p) => *p * 2.0,
            Margin::Center { h, .. } => *h * 2.0,
            Margin::Custom { l, r, .. } => *l + *r,
        }
    }
}

impl From<f32> for Margin {
    fn from(value: f32) -> Self {
        Margin::Even(value)
    }
}

impl From<(f32, f32)> for Margin {
    fn from((v, h): (f32, f32)) -> Self {
        Margin::Center { v, h }
    }
}

impl From<(f32, f32, f32, f32)> for Margin {
    fn from((t, r, b, l): (f32, f32, f32, f32)) -> Self {
        Margin::Custom { t, r, b, l }
    }
}
