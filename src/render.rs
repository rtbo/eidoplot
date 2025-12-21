//! Render module: provides abstraction over rendering surfaces, like pixel-based, SVG, or GUI.
//!
//! All rendering surfaces must implement the `Surface` trait.
//! See the `eidoplot-pxl` and `eidoplot-svg` crates for examples.
use std::fmt;

use crate::{ColorU8, geom, text};

/// Errors that can occur during rendering
#[derive(Debug)]
pub enum Error {
    /// Font or text related error
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

/// Surface trait: defines the rendering surface API
pub trait Surface {
    /// Prepare the surface for drawing, with the given size in plot units
    fn prepare(&mut self, size: geom::Size) -> Result<(), Error>;

    /// Fill the entire surface with the given fill pattern
    fn fill(&mut self, fill: Paint) -> Result<(), Error>;

    /// Draw a rectangle
    ///
    /// Default implementation converts the rectangle to a path and call [`draw_path`](Surface::draw_path)
    fn draw_rect(&mut self, rect: &Rect) -> Result<(), Error> {
        let path = rect.rect.to_path();
        let path = self::Path {
            path: &path,
            fill: rect.fill,
            stroke: rect.stroke,
            transform: rect.transform,
        };
        self.draw_path(&path)?;
        Ok(())
    }

    /// Draw a path
    fn draw_path(&mut self, path: &Path) -> Result<(), Error>;

    /// Draw a line of text
    fn draw_text(&mut self, text: &Text) -> Result<(), Error>;

    /// Draw a pre-shaped line of text
    fn draw_line_text(&mut self, text: &LineText) -> Result<(), Error>;

    /// Draw a rich text
    fn draw_rich_text(&mut self, text: &RichText) -> Result<(), Error>;

    /// Push a clipping rect
    /// Subsequent draw operations will be clipped to this rect,
    /// until a matching [`pop_clip`] is called
    fn push_clip(&mut self, clip: &Clip) -> Result<(), Error>;

    /// Pop a clipping rect that was pushed previously with [`push_clip`]
    fn pop_clip(&mut self) -> Result<(), Error>;
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

/// Text to draw
#[derive(Debug, Clone)]
pub struct Text<'a> {
    /// Text content
    pub text: &'a str,
    /// Font to use
    pub font: &'a text::Font,
    /// Font size in figure units
    pub font_size: f32,
    /// Fill style
    pub fill: Paint,
    /// Alignment
    pub align: (text::line::Align, text::line::VerAlign),
    /// Optional transform to apply to the text
    pub transform: Option<&'a geom::Transform>,
}

/// Pre-shaped line of text to draw
#[derive(Debug, Clone)]
pub struct LineText<'a> {
    /// Line text content
    pub text: &'a text::LineText,
    /// Font size in figure units
    pub fill: Paint,
    /// Optional transform to apply to the text
    pub transform: geom::Transform,
}

/// Rich text to draw
#[derive(Debug, Clone)]
pub struct RichText<'a> {
    /// Rich text content
    pub text: &'a text::RichText,
    /// Fill style
    pub transform: geom::Transform,
}
