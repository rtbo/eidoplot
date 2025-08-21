use std::fmt;

use crate::{geom, style};

#[derive(Debug)]
pub enum Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "render error")
    }
}

impl std::error::Error for Error {}

pub trait Surface {
    /// Prepare the surface for drawing, with the given size in plot units
    fn prepare(&mut self, size: geom::Size) -> Result<(), Error>;

    /// Fill the entire surface with the given fill pattern
    fn fill(&mut self, fill: style::Fill) -> Result<(), Error>;

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: &Rect) -> Result<(), Error>;

    /// Draw a path
    fn draw_path(&mut self, path: &Path) -> Result<(), Error>;

    /// Draw some text
    fn draw_text(&mut self, text: &Text) -> Result<(), Error>;

    /// Draw some text that has already been layed out
    fn draw_text_layout(&mut self, text: &TextLayout) -> Result<(), Error>;

    /// Push a clipping path
    /// Subsequent draw operations will be clipped to this path,
    /// until a matching [`pop_clip`] is called
    fn push_clip(&mut self, clip: &Clip) -> Result<(), Error>;

    /// Pop a clipping path that was pushed previously with [`push_clip`]
    fn pop_clip(&mut self) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct Rect<'a> {
    pub rect: geom::Rect,
    pub fill: Option<style::Fill>,
    pub stroke: Option<style::Line>,
    pub transform: Option<&'a geom::Transform>,
}

impl<'a> Rect<'a> {
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
pub struct Path<'a> {
    pub path: &'a geom::Path,
    pub fill: Option<style::Fill>,
    pub stroke: Option<style::Line>,
    pub transform: Option<&'a geom::Transform>,
}

#[derive(Debug, Clone)]
pub struct Clip<'a> {
    pub path: &'a geom::Path,
    pub transform: Option<&'a geom::Transform>,
}

#[derive(Debug, Clone)]
pub struct Text<'a> {
    pub text: &'a str,
    pub font: &'a eidoplot_text::Font,
    pub font_size: f32,
    pub fill: style::Fill,
    pub options: eidoplot_text::layout::Options,
    pub transform: Option<&'a geom::Transform>,
}

#[derive(Debug, Clone)]
pub struct TextLayout<'a> {
    pub layout: &'a eidoplot_text::TextLayout,
    pub fill: style::Fill,
    pub transform: Option<&'a geom::Transform>,
}
