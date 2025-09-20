use std::fmt;

use ttf_parser as ttf;

pub mod font;
pub mod fontdb;
pub mod layout;
pub mod render;
pub mod rich;
pub mod shape;

pub use font::{Font, ScaledMetrics, parse_font_families};
pub use layout::{Anchor, BBox, HorAlign, LineVerAlign, TextLayout, VerAlign};
pub use render::{render_text, render_text_tiny_skia};
pub use shape::{Direction, TextShape};

#[derive(Debug, Clone)]
pub enum Error {
    NoSuchFont(Font),
    FaceParsingError(ttf::FaceParsingError),
    BadLayoutParamsError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoSuchFont(font) => write!(f, "Could not find a face for {:?}", font),
            Error::FaceParsingError(err) => err.fmt(f),
            Error::BadLayoutParamsError => write!(f, "Bad text layout parameters"),
        }
    }
}

impl From<ttf::FaceParsingError> for Error {
    fn from(err: ttf::FaceParsingError) -> Self {
        Error::FaceParsingError(err)
    }
}

impl From<layout::BadLayoutParamsError> for Error {
    fn from(_err: layout::BadLayoutParamsError) -> Self {
        Error::BadLayoutParamsError
    }
}

impl std::error::Error for Error {}

/// A shorthand function that shape a text and create a layout of a string
pub fn shape_and_layout_str(
    text: &str,
    font: &Font,
    db: &font::Database,
    font_size: f32,
    opts: &layout::Options,
) -> Result<TextLayout, Error> {
    let shape = TextShape::shape_str(text, font, db)?;
    Ok(TextLayout::from_shape(&shape, font_size, opts)?)
}
