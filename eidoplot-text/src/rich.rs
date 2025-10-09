use crate::{
    font,
    fontdb,
};
use std::fmt;
use ttf_parser as ttf;

mod builder;

pub use builder::RichTextBuilder;

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

/// Typographic alignment, possibly depending on the script direction.
#[derive(Debug, Clone, Copy, Default)]
pub enum TypeAlign {
    /// The start of the text is aligned with the reference point.
    #[default]
    Start,
    /// Text is centered around the reference point.
    Center,
    /// The end of the text is aligned with the reference point.
    End,
    /// Text is left aligned.
    /// For vertical layout, this is the same as [`Start`].
    Left,
    /// Text is right aligned.
    /// For vertical layout, this is the same as [`End`].
    Right,
    /// The text is justified on both ends.
    /// The parameter is the total width of the text (or height for vertical text)
    Justify(f32),
}

/// Vertical alignment for a single line of horizontal text
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineAlign {
    /// Align the bottom of the descender
    Bottom,
    /// Align the baseline
    #[default]
    Baseline,
    /// Align at middle of the x-height
    Middle,
    /// Align at capital height
    Hanging,
    /// Align at the top of the ascender
    Top,
}

/// Vertical alignment for a whole horizontal text, possibly considering multiple lines
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// Align at the specified line
    Line(usize, LineAlign),
    /// Align at the top (ascender) of the first line
    Top,
    /// Align at the center, that is (top + bottom) / 2
    Center,
    /// Align at the bottom (descender) of the last line
    Bottom,
}

impl Default for Align {
    fn default() -> Self {
        Align::Line(0, Default::default())
    }
}

impl From<LineAlign> for Align {
    fn from(value: LineAlign) -> Self {
        Self::Line(0, value)
    }
}

/// A direction for horizontal text layout.
/// Direction refers to left to right, or right to left text.
/// The mixed directions take into account bidirectional text and refer to
/// the main direction of the text.
/// (see https://www.w3.org/International/articles/inline-bidi-markup/uba-basics#context)
#[derive(Debug, Clone, Copy, Default)]
pub enum Direction {
    /// The main direction is the one of the first encountered script.
    /// This is the default.
    #[default]
    Mixed,
    /// The main direction of the script is Left to Right.
    /// Note that Right to Left script will still be rendered right to left,
    /// but in case of mixed script, the main direction is left to right.
    MixedLTR,
    /// The main direction of the script is Right to Left.
    /// Note that Left to Right script will still be rendered left to right,
    /// but in case of mixed script, the main direction is right to left.
    MixedRTL,
    /// Left to right text. (no bidirectional algorithm applied)
    LTR,
    /// Right to left text. (no bidirectional algorithm applied)
    RTL,
}

/// Direction for vertical text layout
#[derive(Debug, Clone, Copy, Default)]
pub enum VerDirection {
    /// Top to bottom
    #[default]
    TTB,
    /// Bottom to top
    BTT,
}

/// Direction of progression of successive columns for vertical text.
/// While horizontal text lines always progress from top to bottom,
/// vertical text columns can progress either left to right or right to left.
#[derive(Debug, Clone, Copy, Default)]
pub enum VerProgression {
    /// Progression is determined by the main direction of the script.
    /// That is, Hebrew and Arabic progress right to left, while others progress left to right.
    #[default]
    PerScript,
    /// Progression is left to right
    LTR,
    /// Progression is right to left
    RTL,
}

/// Layout options for rich text
#[derive(Debug, Clone, Copy)]
pub enum Layout {
    /// Horizontal text layout options
    Horizontal(Align, TypeAlign, Direction),
    /// Vertical text layout options
    Vertical(TypeAlign, VerDirection, VerProgression),
}

impl Default for Layout {
    fn default() -> Self {
        Layout::Horizontal(Align::default(), TypeAlign::default(), Direction::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// A set of properties to be applied to a text span.
/// If a property is `None`, value is inherited from the parent span.
#[derive(Debug, Clone, PartialEq)]
pub struct TextOptProps {
    font_family: Option<Vec<font::Family>>,
    font_weight: Option<font::Weight>,
    font_width: Option<font::Width>,
    font_style: Option<font::Style>,
    font_size: Option<f32>,
    fill: Option<Color>,
    stroke: Option<Color>,
    underline: Option<bool>,
    strikeout: Option<bool>,
}

impl TextOptProps {
    fn affect_shape(&self) -> bool {
        self.font_family.is_some()
            || self.font_weight.is_some()
            || self.font_width.is_some()
            || self.font_style.is_some()
            || self.font_size.is_some()
    }
}

/// A set of resolved properties for a text span
#[derive(Debug, Clone)]
pub struct TextProps {
    font: font::Font,
    font_size: f32,
    fill: Option<Color>,
    stroke: Option<Color>,
    underline: bool,
    strikeout: bool,
}

impl TextProps {
    pub fn new(font_size: f32) -> TextProps {
        TextProps {
            font: font::Font::default(),
            font_size,
            fill: Some(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }),
            stroke: None,
            underline: false,
            strikeout: false,
        }
    }

    fn apply_opts(&mut self, opts: &TextOptProps) {
        if let Some(font_family) = &opts.font_family {
            self.font = self.font.clone().with_families(font_family.clone());
        }
        if let Some(font_weight) = opts.font_weight {
            self.font = self.font.clone().with_weight(font_weight);
        }
        if let Some(font_width) = opts.font_width {
            self.font = self.font.clone().with_width(font_width);
        }
        if let Some(font_style) = opts.font_style {
            self.font = self.font.clone().with_style(font_style);
        }
        if let Some(font_size) = opts.font_size {
            self.font_size = font_size;
        }
        if let Some(fill) = opts.fill {
            self.fill = Some(fill);
        }
        if let Some(stroke) = opts.stroke {
            self.stroke = Some(stroke);
        }
        if let Some(underline) = opts.underline {
            self.underline = underline;
        }
        if let Some(strikeout) = opts.strikeout {
            self.strikeout = strikeout;
        }
    }
}

/// A text span
#[derive(Debug, Clone)]
struct TextSpan {
    start: usize,
    end: usize,
    props: TextOptProps,
}
#[derive(Debug, Clone)]
struct TextLine {
    start: usize,
    end: usize,
    shapes: Vec<ShapeSpan>,
    main_dir: rustybuzz::Direction,
}

#[derive(Debug, Clone)]
struct ShapeSpan {
    start: usize,
    end: usize,
    spans: Vec<PropsSpan>,
    face_id: fontdb::ID,
    glyphs: Vec<Glyph>,
    metrics: font::ScaledMetrics,
}

impl ShapeSpan {
    fn x_advance(&self) -> f32 {
        self.glyphs.iter().map(|g| g.x_advance as f32).sum()
    }
}

#[derive(Debug, Clone)]
struct PropsSpan {
    start: usize,
    end: usize,
    props: TextProps,
}

#[derive(Debug, Clone, Copy)]
struct Glyph {
    id: ttf::GlyphId,
    cluster: usize,
    x_advance: f32,
    y_advance: f32,
    x_offset: f32,
    y_offset: f32,
    ts: tiny_skia::Transform,
}

#[derive(Debug, Clone)]
pub struct RichTextLayout {
    text: String,
    lines: Vec<TextLine>,
}

impl RichTextLayout {
    fn empty() -> Self {
        Self {
            text: String::new(),
            lines: Vec::new(),
        }
    }
}
