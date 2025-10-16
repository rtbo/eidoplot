use std::fmt;

use ttf_parser as ttf;

mod bidi;
pub mod font;
pub mod fontdb;
pub mod layout;
pub mod line;
pub mod render;
pub mod rich;
pub mod shape;

pub use font::{Font, ScaledMetrics, parse_font_families};
pub use layout::{Anchor, BBox, HorAlign, LineVerAlign, TextLayout, VerAlign};
pub use render::{render_text, render_text_tiny_skia};
pub use rich::{RichTextBuilder, RichTextLayout};
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

fn script_is_rtl(text: &str) -> Option<bool> {
    use unicode_bidi::{BidiClass, bidi_class};
    let mut in_doublt_rtl = false;
    for c in text.chars() {
        let bc = bidi_class(c);
        match bc {
            BidiClass::L | BidiClass::LRE | BidiClass::LRO | BidiClass::LRI => {
                return Some(false);
            }
            BidiClass::R | BidiClass::AL | BidiClass::RLE | BidiClass::RLO | BidiClass::RLI => {
                return Some(true);
            }
            BidiClass::AN => {
                // arabic number, can be in both contexts, but if we have only those, we chose RTL
                in_doublt_rtl = true;
            }
            _ => (),
        }
    }
    if in_doublt_rtl { Some(true) } else { None }
}
