use crate::{geom, style};

pub trait Surface {
    type Error;

    /// Prepare the surface for drawing, with the given size in plot units
    fn prepare(&mut self, size: geom::Size) -> Result<(), Self::Error>;

    /// Fill the entire surface with the given fill pattern
    fn fill(&mut self, fill: style::Fill) -> Result<(), Self::Error>;

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: &Rect) -> Result<(), Self::Error>;

    /// Draw a path
    fn draw_path(&mut self, path: &Path) -> Result<(), Self::Error>;

    /// Draw some text
    fn draw_text(&mut self, text: &Text) -> Result<(), Self::Error>;

    /// Push a clipping path
    /// Subsequent draw operations will be clipped to this path,
    /// until a matching [`pop_clip`] is called
    fn push_clip(&mut self, clip: &Clip) -> Result<(), Self::Error>;

    /// Pop a clipping path that was pushed previously with [`push_clip`]
    fn pop_clip(&mut self) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub rect: geom::Rect,
    pub fill: Option<style::Fill>,
    pub stroke: Option<style::Line>,
    pub transform: Option<geom::Transform>,
}

impl Rect {
    pub fn new(rect: geom::Rect) -> Self {
        Rect {
            rect,
            fill: None,
            stroke: None,
            transform: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    pub path: geom::Path,
    pub fill: Option<style::Fill>,
    pub stroke: Option<style::Line>,
    pub transform: Option<geom::Transform>,
}

#[derive(Debug, Clone)]
pub struct Clip {
    pub path: geom::Path,
    pub transform: Option<geom::Transform>,
}

#[derive(Debug, Clone, Copy)]
pub enum TextAlign {
    Start,
    Center,
    End,
}

impl Default for TextAlign {
    fn default() -> Self {
        TextAlign::Center
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextBaseline {
    Base,
    Center,
    Hanging,
}

impl Default for TextBaseline {
    fn default() -> Self {
        TextBaseline::Base
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextAnchor {
    pub pos: geom::Point,
    pub align: TextAlign,
    pub baseline: TextBaseline,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
    pub font: style::Font,
    pub fill: style::Fill,
    pub anchor: TextAnchor,
    pub transform: Option<geom::Transform>,
}
