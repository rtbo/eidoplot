use std::fmt;


use crate::geom;
use crate::style::ColorU8;
use crate::text;

#[derive(Debug)]
pub enum Error {
    FontOrText(text::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FontOrText(err) => err.fmt(f),
        }
    }
}

impl From<text::Error> for Error {
    fn from(err: text::Error) -> Self {
        Error::FontOrText(err)
    }
}

impl From<ttf_parser::FaceParsingError> for Error {
    fn from(err: ttf_parser::FaceParsingError) -> Self {
        Error::FontOrText(err.into())
    }
}

impl std::error::Error for Error {}

pub trait Surface {
    /// Prepare the surface for drawing, with the given size in plot units
    fn prepare(&mut self, size: geom::Size) -> Result<(), Error>;

    /// Fill the entire surface with the given fill pattern
    fn fill(&mut self, fill: Paint) -> Result<(), Error>;

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: &Rect) -> Result<(), Error>;

    /// Draw a path
    fn draw_path(&mut self, path: &Path) -> Result<(), Error>;

    /// Draw a line of text
    fn draw_text(&mut self, text: &Text) -> Result<(), Error>;

    /// Draw a line of text
    fn draw_line_text(&mut self, text: &LineText) -> Result<(), Error>;

    /// Draw a rich text
    fn draw_rich_text(&mut self, text: &RichText) -> Result<(), Error>;

    /// Push a clipping path
    /// Subsequent draw operations will be clipped to this path,
    /// until a matching [`pop_clip`] is called
    fn push_clip(&mut self, clip: &Clip) -> Result<(), Error>;

    /// Pop a clipping path that was pushed previously with [`push_clip`]
    fn pop_clip(&mut self) -> Result<(), Error>;
}

#[derive(Debug, Clone, Copy)]
pub enum Paint {
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

#[derive(Debug, Clone, Copy)]
pub struct Stroke<'a> {
    pub color: ColorU8,
    pub width: f32,
    pub pattern: LinePattern<'a>,
}

#[derive(Debug, Clone)]
pub struct Rect<'a> {
    pub rect: geom::Rect,
    pub fill: Option<Paint>,
    pub stroke: Option<Stroke<'a>>,
    pub transform: Option<&'a geom::Transform>,
}

#[derive(Debug, Clone)]
pub struct Path<'a> {
    pub path: &'a geom::Path,
    pub fill: Option<Paint>,
    pub stroke: Option<Stroke<'a>>,
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
    pub font: &'a text::Font,
    pub font_size: f32,
    pub fill: Paint,
    pub align: (text::line::Align, text::line::VerAlign),
    pub transform: Option<&'a geom::Transform>,
}

#[derive(Debug, Clone)]
pub struct LineText<'a> {
    pub text: &'a text::LineText,
    pub fill: Paint,
    pub transform: geom::Transform,
}

#[derive(Debug, Clone)]
pub struct RichText<'a> {
    pub text: &'a text::RichText,
    pub transform: geom::Transform,
}
