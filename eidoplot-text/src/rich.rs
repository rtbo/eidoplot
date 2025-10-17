use crate::{
    font,
    fontdb, BBox,
    Error,
};
use ttf_parser as ttf;

mod boundaries;
mod builder;
mod render;

use boundaries::Boundaries;
pub use render::render_rich_text;


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

/// Space between two columns of vertical space.
/// This value is a factor of the font em-box side size
/// E.g. if the em-box after scaling is 40px wide, a value of 0.5 will yield
/// to an inter-space of 20px. (0.5 is the default value)
#[derive(Debug, Clone, Copy)]
pub struct InterColumn(pub f32);

impl Default for InterColumn {
    fn default() -> Self {
        InterColumn(0.5)
    }
}


/// Layout options for rich text
#[derive(Debug, Clone, Copy)]
pub enum Layout {
    /// Horizontal text layout options
    Horizontal(Align, TypeAlign, Direction),
    /// Vertical text layout options
    Vertical(TypeAlign, VerDirection, VerProgression, InterColumn),
}

impl Default for Layout {
    fn default() -> Self {
        Layout::Horizontal(Align::default(), TypeAlign::default(), Direction::default())
    }
}

/// A builder struct for rich text
#[derive(Debug, Clone)]
pub struct RichTextBuilder {
    text: String,
    root_props: TextProps,
    layout: Layout,
    spans: Vec<TextSpan>,
}

impl RichTextBuilder {
    /// Create a new RichTextBuilder
    pub fn new(text: String, root_props: TextProps) -> RichTextBuilder {
        RichTextBuilder {
            text,
            root_props,
            layout: Layout::default(),
            spans: vec![],
        }
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    /// Add a new text span
    pub fn add_span(&mut self, start: usize, end: usize, props: TextOptProps) {
        assert!(start <= end);
        assert!(
            self.text.is_char_boundary(start) && self.text.is_char_boundary(end),
            "start and end must be on char boundaries"
        );
        self.spans.push(TextSpan { start, end, props });
    }

    /// Create a RichText from this builder
    pub fn done(self, fontdb: &fontdb::Database) -> Result<RichText, Error> {
        self.done_impl(fontdb)
    }
}

#[derive(Debug, Clone)]
pub struct RichText {
    text: String,
    lines: Vec<LineSpan>,
    bbox: BBox,
}

impl RichText {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn bbox(&self) -> BBox {
        self.bbox
    }

    fn empty() -> Self {
        Self {
            text: String::new(),
            lines: Vec::new(),
            bbox: BBox::EMPTY,
        }
    }

    #[cfg(debug_assertions)]
    pub fn assert_flat_coverage(&self) {
        let len = self.text.len();
        let mut cursor = 0;
        for l in self.lines.iter() {
            assert_eq!(l.start, cursor);
            cursor = l.end;
            if cursor == len {
                // last line might not end with a newline
                break;
            }
            if self.text.as_bytes()[cursor] == b'\r' {
                cursor += 1;
            }
            assert_eq!(self.text.as_bytes()[cursor], b'\n', "expected end of line, found {}", self.text[cursor..].chars().next().unwrap());
            cursor += 1;
            l.assert_flat_coverage();
        }
        assert_eq!(cursor, len);
    }
}

/// A RGBA color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red component
    pub r: u8,
    /// Green component
    pub g: u8,
    /// Blue component
    pub b: u8,
    /// Alpha component
    pub a: u8,
}

/// A set of properties to be applied to a text span.
/// If a property is `None`, value is inherited from the parent span.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextOptProps {
    pub font_family: Option<Vec<font::Family>>,
    pub font_weight: Option<font::Weight>,
    pub font_width: Option<font::Width>,
    pub font_style: Option<font::Style>,
    pub font_size: Option<f32>,
    pub fill: Option<Color>,
    pub stroke: Option<(Color, f32)>,
    pub underline: Option<bool>,
    pub strikeout: Option<bool>,
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
    font_size: f32,
    font: font::Font,
    fill: Option<Color>,
    outline: Option<(Color, f32)>,
    underline: bool,
    strikeout: bool,
}

impl TextProps {
    pub fn new(font_size: f32) -> TextProps {
        TextProps {
            font_size,
            font: font::Font::default(),
            fill: Some(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }),
            outline: None,
            underline: false,
            strikeout: false,
        }
    }

    pub fn with_font(mut self, font: font::Font) -> Self {
        self.font = font;
        self
    }

    pub fn with_fill(mut self, fill: Option<Color>) -> Self {
        self.fill = fill;
        self
    }

    pub fn with_outline(mut self, stroke: Option<(Color, f32)>) -> Self {
        self.outline = stroke;
        self
    }

    pub fn with_underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn with_strikeout(mut self) -> Self {
        self.strikeout = true;
        self
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
            self.outline = Some(stroke);
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
struct LineSpan {
    start: usize,
    end: usize,
    shapes: Vec<ShapeSpan>,
    main_dir: rustybuzz::Direction,
    bbox: BBox,
}

impl LineSpan {
    #[cfg(debug_assertions)]
    fn assert_flat_coverage(&self) {
        let mut cursor = self.start;
        for s in self.shapes.iter() {
            assert_eq!(s.start, cursor);
            cursor = s.end;
            s.assert_flat_coverage();
        }
        assert_eq!(cursor, self.end);
    }
}

#[derive(Debug, Clone)]
struct ShapeSpan {
    start: usize,
    end: usize,
    spans: Vec<PropsSpan>,
    face_id: fontdb::ID,
    glyphs: Vec<Glyph>,
    metrics: font::ScaledMetrics,
    y_baseline: f32,
    bbox: BBox,
}

impl ShapeSpan {
    #[cfg(debug_assertions)]
    fn assert_flat_coverage(&self) {
        let mut cursor = self.start;
        for s in self.spans.iter() {
            assert_eq!(s.start, cursor);
            cursor = s.end;
        }
        assert_eq!(cursor, self.end);
    }
}

#[derive(Debug, Clone)]
struct PropsSpan {
    start: usize,
    end: usize,
    props: TextProps,
    bbox: BBox,
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
