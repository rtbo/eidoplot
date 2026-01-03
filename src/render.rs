//! Render module: provides abstraction over rendering surfaces, like pixel-based, SVG, or GUI.
//!
//! All rendering surfaces must implement the `Surface` trait.
//! See the `plotive-pxl` and `plotive-svg` crates for examples.

use crate::{ColorU8, geom};

/// Surface trait: defines the rendering surface API
pub trait Surface {
    /// Prepare the surface for drawing, with the given size in plot units
    fn prepare(&mut self, size: geom::Size);

    /// Fill the entire surface with the given fill pattern
    fn fill(&mut self, fill: Paint);

    /// Draw a rectangle
    ///
    /// Default implementation converts the rectangle to a path and call [`draw_path`](Surface::draw_path)
    fn draw_rect(&mut self, rect: &Rect) {
        let path = rect.rect.to_path();
        let rpath = self::Path {
            path: &path,
            fill: rect.fill,
            stroke: rect.stroke,
            transform: rect.transform,
        };
        self.draw_path(&rpath);
    }

    /// Draw a path
    fn draw_path(&mut self, path: &Path);

    /// Push a clipping rect
    /// Subsequent draw operations will be clipped to this rect,
    /// until a matching [`pop_clip`] is called
    fn push_clip(&mut self, clip: &Clip);

    /// Pop a clipping rect that was pushed previously with [`push_clip`]
    fn pop_clip(&mut self);
}

/// Paint pattern, used for fill operations
#[derive(Debug, Clone, Copy)]
pub enum Paint {
    /// Solid color fill
    Solid(ColorU8),
}

impl From<ColorU8> for Paint {
    fn from(value: ColorU8) -> Self {
        Paint::Solid(value)
    }
}

/// Line pattern defines how the line is drawn
#[derive(Debug, Clone, Copy, Default)]
pub enum LinePattern<'a> {
    /// Solid line
    #[default]
    Solid,
    /// Dashed line. The pattern is relative to the line width.
    Dash(&'a [f32]),
}

/// Stroke style definition
#[derive(Debug, Clone, Copy)]
pub struct Stroke<'a> {
    /// Line color
    pub color: ColorU8,
    /// Line width in figure units
    pub width: f32,
    /// Line pattern
    pub pattern: LinePattern<'a>,
}

/// Rectangle to draw
#[derive(Debug, Clone)]
pub struct Rect<'a> {
    /// Rectangle geometry
    pub rect: geom::Rect,
    /// Fill style
    pub fill: Option<Paint>,
    /// Stroke style
    pub stroke: Option<Stroke<'a>>,
    /// Optional transform to apply to the rectangle
    pub transform: Option<&'a geom::Transform>,
}

/// Path to draw
#[derive(Debug, Clone)]
pub struct Path<'a> {
    /// Path geometry
    pub path: &'a geom::Path,
    /// Fill style
    pub fill: Option<Paint>,
    /// Stroke style
    pub stroke: Option<Stroke<'a>>,
    /// Optional transform to apply to the path
    pub transform: Option<&'a geom::Transform>,
}

/// Clipping rectangle
#[derive(Debug, Clone)]
pub struct Clip<'a> {
    /// Clipping rectangle
    pub rect: &'a geom::Rect,
    /// Optional transform to apply to the clipping rectangle
    pub transform: Option<&'a geom::Transform>,
}
