use std::fmt;

use ttf_parser as ttf;

pub mod font;
pub mod render;
pub mod shape;
pub mod style;

pub use render::{HorAlign, LineAlign, LineVerAlign, TextAlign, TextVerAlign, render_line, render_text};
pub use shape::shape_text;

#[derive(Debug, Clone)]
pub enum Error {
    NoSuchFont(style::Font),
    FaceParsingError(ttf::FaceParsingError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoSuchFont(style) => write!(f, "Could not find a font for {:?}", style),
            Error::FaceParsingError(err) => err.fmt(f),
        }
    }
}

impl From<ttf::FaceParsingError> for Error {
    fn from(err: ttf::FaceParsingError) -> Self {
        Error::FaceParsingError(err)
    }
}

impl std::error::Error for Error {}
