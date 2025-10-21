use crate::{BBox, Error, font, fontdb};
use ttf_parser as ttf;

mod boundaries;
mod builder;
mod render;

use boundaries::Boundaries;
pub use render::render_rich_text;

/// Typographic alignment, possibly depending on the script direction.
#[derive(Debug, Clone, Copy, Default)]
pub enum Align {
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
pub enum VerAlign {
    /// Align at the specified line
    Line(usize, LineAlign),
    /// Align at the top (ascender) of the first line
    Top,
    /// Align at the center, that is (top + bottom) / 2
    Center,
    /// Align at the bottom (descender) of the last line
    Bottom,
}

impl Default for VerAlign {
    fn default() -> Self {
        VerAlign::Line(0, Default::default())
    }
}

impl From<LineAlign> for VerAlign {
    fn from(value: LineAlign) -> Self {
        Self::Line(0, value)
    }
}

/// Horizontal alignement of vertical text
/// This will not affect the placement of glyphs relative to each other,
/// but will dictate how they are aligned relative to the reference X coordinate.
/// The default is center.
#[derive(Debug, Clone, Copy, Default)]
pub enum HorAlign {
    Left,
    #[default]
    Center,
    Right,
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
    /// Right to Left script will still be rendered right to left,
    /// but in case of mixed script, the main direction is left to right.
    MixedLTR,
    /// The main direction of the script is Right to Left.
    /// Left to Right script will still be rendered left to right,
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
    Horizontal(Align, VerAlign, Direction),
    /// Vertical text layout options
    Vertical(Align, HorAlign, VerDirection, VerProgression, InterColumn),
}

impl Default for Layout {
    fn default() -> Self {
        Layout::Horizontal(Default::default(), Default::default(), Default::default())
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
    layout: Layout,
    lines: Vec<LineSpan>,
    bbox: BBox,
}

impl RichText {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn lines(&self) -> &[LineSpan] {
        &self.lines
    }

    pub fn bbox(&self) -> BBox {
        self.bbox
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.bbox.width()
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.bbox.height()
    }

    pub fn visual_bbox(&self) -> BBox {
        if self.lines.is_empty() {
            return BBox::NULL;
        }
        let mut bbox = BBox::EMPTY;
        for l in &self.lines {
            bbox = BBox::unite(&bbox, &l.visual_bbox());
        }
        bbox
    }

    fn empty() -> Self {
        Self {
            text: String::new(),
            layout: Layout::default(),
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
            assert_eq!(
                self.text.as_bytes()[cursor],
                b'\n',
                "expected end of line, found {}",
                self.text[cursor..].chars().next().unwrap()
            );
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

    pub fn with_outline(mut self, stroke: (Color, f32)) -> Self {
        self.outline = Some(stroke);
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

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn font(&self) -> &font::Font {
        &self.font
    }

    pub fn fill(&self) -> Option<Color> {
        self.fill
    }

    pub fn outline(&self) -> Option<(Color, f32)> {
        self.outline
    }

    pub fn underline(&self) -> bool {
        self.underline
    }

    pub fn strikeout(&self) -> bool {
        self.strikeout
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

/// A line of rich text
#[derive(Debug, Clone)]
pub struct LineSpan {
    start: usize,
    end: usize,
    shapes: Vec<ShapeSpan>,
    main_dir: rustybuzz::Direction,
    bbox: BBox,
}

impl LineSpan {
    /// Byte index where the line starts in the text
    pub fn start(&self) -> usize {
        self.start
    }

    /// Byte index where the line ends in the text
    pub fn end(&self) -> usize {
        self.end
    }

    /// The text shapes in this line
    pub fn shapes(&self) -> &[ShapeSpan] {
        &self.shapes
    }

    /// The main text direction of this line
    pub fn main_dir(&self) -> rustybuzz::Direction {
        self.main_dir
    }

    /// Bounding box of the line
    pub fn bbox(&self) -> BBox {
        self.bbox
    }

    /// The total height of this line including the gap to the next one
    pub fn total_height(&self) -> f32 {
        self.height() + self.gap()
    }

    /// The vertical gap from this line to the next.
    /// Can be zero if the font includes this in the height
    pub fn gap(&self) -> f32 {
        self.shapes
            .iter()
            .map(|s| s.metrics.line_gap)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    pub fn height(&self) -> f32 {
        self.shapes
            .iter()
            .map(|s| s.metrics.height())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    pub fn ascent(&self) -> f32 {
        self.shapes
            .iter()
            .map(|s| s.metrics.ascent)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    pub fn descent(&self) -> f32 {
        self.shapes
            .iter()
            .map(|s| s.metrics.descent)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// The maximum capital height of this line.
    /// If there are multiple shape sizes, the average is returned
    pub fn cap_height(&self) -> f32 {
        self.shapes
            .iter()
            .map(|s| s.metrics.cap_height)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// The x-height of this line.
    /// If there are multiple shape sizes, the average is returned
    pub fn x_height(&self) -> f32 {
        if self.shapes.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.shapes.iter().map(|s| s.metrics.x_height).sum();
        sum / (self.shapes.len() as f32)
    }

    /// Visual bounding box of the line
    pub fn visual_bbox(&self) -> BBox {
        if self.shapes.is_empty() {
            return BBox::NULL;
        }
        let mut bbox = BBox::EMPTY;
        for s in &self.shapes {
            bbox = BBox::unite(&bbox, &s.visual_bbox());
        }
        bbox
    }

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

/// A shape of text in a line
/// A shape is a sequence of glyphs that share the same properties:
///   - font
///   - font size
///   - script direction
#[derive(Debug, Clone)]
pub struct ShapeSpan {
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
    /// Byte index where the shape starts in the text
    pub fn start(&self) -> usize {
        self.start
    }

    /// Byte index where the shape ends in the text
    pub fn end(&self) -> usize {
        self.end
    }

    /// The font of this shape
    pub fn font(&self) -> &font::Font {
        &self.spans[0].props.font
    }

    /// The font of this shape
    pub fn font_size(&self) -> f32 {
        self.spans[0].props.font_size
    }

    /// The text spans in this shape
    pub fn spans(&self) -> &[PropsSpan] {
        &self.spans
    }

    /// The metrics of this shape
    pub fn metrics(&self) -> font::ScaledMetrics {
        self.metrics
    }

    /// The bounding box of this shape
    pub fn bbox(&self) -> BBox {
        self.bbox
    }

    /// The visual bounding box of this shape
    pub fn visual_bbox(&self) -> BBox {
        if self.glyphs.is_empty() {
            return BBox::NULL;
        }
        let mut bbox = BBox::EMPTY;
        for g in &self.glyphs {
            bbox = BBox::unite(&bbox, &g.visual_bbox());
        }
        bbox
    }

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

/// A span of text with the same properties
#[derive(Debug, Clone)]
pub struct PropsSpan {
    start: usize,
    end: usize,
    props: TextProps,
    bbox: BBox,
}

impl PropsSpan {
    /// Byte index where the span starts in the text
    pub fn start(&self) -> usize {
        self.start
    }

    /// Byte index where the span ends in the text
    pub fn end(&self) -> usize {
        self.end
    }

    /// The properties of this span
    pub fn props(&self) -> &TextProps {
        &self.props
    }

    /// Bounding box of the span
    pub fn bbox(&self) -> BBox {
        self.bbox
    }
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
    rect: ttf::Rect,
}

impl Glyph {
    fn visual_bbox(&self) -> BBox {
        let mut tl_br = [
            tiny_skia_path::Point {
                x: self.rect.x_min as f32,
                y: self.rect.y_max as f32,
            },
            tiny_skia_path::Point {
                x: self.rect.x_max as f32,
                y: self.rect.y_min as f32,
            },
        ];
        self.ts.map_points(&mut tl_br);
        BBox {
            top: tl_br[0].y,
            right: tl_br[1].x,
            bottom: tl_br[1].y,
            left: tl_br[0].x,
        }
    }
}
