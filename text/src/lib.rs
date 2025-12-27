use std::fmt;

use eidoplot_base::geom;
use ttf_parser as ttf;

mod bidi;
pub mod font;
pub mod fontdb;
pub mod line;
pub mod rich;

pub use font::{Font, ScaledMetrics, parse_font_families};
pub use line::{LineText, render_line_text};
pub use rich::{
    ParseRichTextError, ParsedRichText, RichPrimitive, RichText, RichTextBuilder, parse_rich_text,
    parse_rich_text_with_classes, render_rich_text, render_rich_text_with,
};

#[cfg(any(
    feature = "noto-sans",
    feature = "noto-sans-italic",
    feature = "noto-serif",
    feature = "noto-serif-italic",
    feature = "noto-mono"
))]
/// Loads fonts that are bundled with eidoplot
/// and returns the database.
pub fn bundled_font_db() -> fontdb::Database {
    let mut db = fontdb::Database::new();

    #[cfg(feature = "noto-sans")]
    db.load_font_data(include_bytes!("noto/NotoSans-VariableFont_wdth,wght.ttf").to_vec());
    #[cfg(feature = "noto-sans-italic")]
    db.load_font_data(include_bytes!("noto/NotoSans-Italic-VariableFont_wdth,wght.ttf").to_vec());
    #[cfg(any(feature = "noto-sans", feature = "noto-sans-italic"))]
    db.set_sans_serif_family("Noto Sans");

    #[cfg(feature = "noto-serif")]
    db.load_font_data(include_bytes!("noto/NotoSerif-VariableFont_wdth,wght.ttf").to_vec());
    #[cfg(feature = "noto-serif-italic")]
    db.load_font_data(include_bytes!("noto/NotoSerif-Italic-VariableFont_wdth,wght.ttf").to_vec());
    #[cfg(any(feature = "noto-serif", feature = "noto-serif-italic"))]
    db.set_serif_family("Noto Serif");

    #[cfg(feature = "noto-mono")]
    db.load_font_data(include_bytes!("noto/NotoSansMono-VariableFont_wdth,wght.ttf").to_vec());
    #[cfg(feature = "noto-mono")]
    db.set_monospace_family("Noto Sans Mono");

    db
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidSpan(String),
    NoSuchFont(font::Font),
    FaceParsingError(ttf::FaceParsingError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidSpan(s) => write!(f, "Invalid span: {}", s),
            Error::NoSuchFont(font) => write!(f, "Could not find a face for {:?}", font),
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

/// Script direction
#[derive(Debug, Clone, Copy)]
pub enum ScriptDir {
    /// Left to right
    LeftToRight,
    /// Right to left
    RightToLeft,
}

impl From<ScriptDir> for rustybuzz::Direction {
    fn from(dir: ScriptDir) -> Self {
        match dir {
            ScriptDir::LeftToRight => rustybuzz::Direction::LeftToRight,
            ScriptDir::RightToLeft => rustybuzz::Direction::RightToLeft,
        }
    }
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

struct Outliner<'a>(&'a mut geom::PathBuilder);

impl ttf::OutlineBuilder for Outliner<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.0.close();
    }
}
