use plotive_base::{Color, ColorU8, color, geom};
use ttf_parser as ttf;

use crate::{Error, font, fontdb, line};

mod boundaries;
mod builder;
mod parse;
mod render;

use boundaries::Boundaries;
pub use parse::{
    ParseRichTextError, ParsedRichText, parse_rich_text, parse_rich_text_with_classes,
};
pub use render::{RichPrimitive, render_rich_text, render_rich_text_with};

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
    /// For vertical layout, this is the same as [Start](Self::Start).
    Left,
    /// Text is right aligned.
    /// For vertical layout, this is the same as [`End`].
    Right,
    /// The text is justified on both ends.
    /// The parameter is the total width of the text (or height for vertical text)
    Justify(f32),
}

/// Vertical alignment for a whole horizontal text, possibly considering multiple lines
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerAlign {
    /// Align at the specified line
    Line(usize, line::VerAlign),
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

impl From<line::VerAlign> for VerAlign {
    fn from(value: line::VerAlign) -> Self {
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
pub struct RichTextBuilder<C>
where
    C: Clone + PartialEq,
{
    text: String,
    root_props: TextProps<C>,
    layout: Layout,
    spans: Vec<TextSpan<C>>,
}

impl<C> RichTextBuilder<C>
where
    C: Clone + PartialEq,
{
    /// Create a new RichTextBuilder
    pub fn new(text: String, root_props: TextProps<C>) -> RichTextBuilder<C> {
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
    pub fn add_span(&mut self, start: usize, end: usize, props: TextOptProps<C>) {
        assert!(start <= end);
        assert!(
            self.text.is_char_boundary(start) && self.text.is_char_boundary(end),
            "start and end must be on char boundaries"
        );
        self.spans.push(TextSpan { start, end, props });
    }

    /// Create a RichText from this builder
    pub fn done(self, fontdb: &fontdb::Database) -> Result<RichText<C>, Error> {
        self.done_impl(fontdb)
    }
}

#[derive(Debug, Clone)]
pub struct RichText<C = ColorU8>
where
    C: Clone,
{
    text: String,
    layout: Layout,
    lines: Vec<LineSpan<C>>,
    bbox: Option<geom::Rect>,
}

impl<C> RichText<C>
where
    C: Clone,
{
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn lines(&self) -> &[LineSpan<C>] {
        &self.lines
    }

    pub fn bbox(&self) -> Option<&geom::Rect> {
        self.bbox.as_ref()
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.bbox.map_or(0.0, |bbox| bbox.width())
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.bbox.map_or(0.0, |bbox| bbox.height())
    }

    pub fn visual_bbox(&self) -> Option<geom::Rect> {
        if self.lines.is_empty() {
            return None;
        }
        let mut bbox = None;
        for l in &self.lines {
            bbox = geom::Rect::unite_opt(bbox.as_ref(), l.visual_bbox().as_ref());
        }
        bbox
    }

    /// Convert this RichText to another color type using the provided mapping function
    pub fn to_other_color<D, M>(&self, color_map: M) -> RichText<D>
    where
        D: Clone,
        M: Fn(&C) -> D,
    {
        RichText {
            text: self.text.clone(),
            layout: self.layout,
            lines: self
                .lines
                .iter()
                .map(|l| l.to_other_color(&color_map))
                .collect(),
            bbox: self.bbox,
        }
    }

    fn empty() -> Self {
        Self {
            text: String::new(),
            layout: Layout::default(),
            lines: Vec::new(),
            bbox: None,
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

/// A set of properties to be applied to a text span.
/// If a property is `None`, value is inherited from the parent span.
#[derive(Debug, Clone, PartialEq)]
pub struct TextOptProps<C> {
    pub font_family: Option<Vec<font::Family>>,
    pub font_weight: Option<font::Weight>,
    pub font_width: Option<font::Width>,
    pub font_style: Option<font::Style>,
    pub font_size: Option<f32>,
    pub fill: Option<C>,
    pub stroke: Option<(C, f32)>,
    pub underline: Option<bool>,
    pub strikeout: Option<bool>,
}

impl<C> Default for TextOptProps<C> {
    fn default() -> Self {
        TextOptProps {
            font_family: None,
            font_weight: None,
            font_width: None,
            font_style: None,
            font_size: None,
            fill: None,
            stroke: None,
            underline: None,
            strikeout: None,
        }
    }
}

impl<C> TextOptProps<C> {
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
pub struct TextProps<C>
where
    C: Clone,
{
    font_size: f32,
    font: font::Font,
    fill: Option<C>,
    outline: Option<(C, f32)>,
    underline: bool,
    strikeout: bool,
}

impl<C> TextProps<C>
where
    C: Clone,
{
    /// Convert this TextProps to another color type using the provided mapping function
    pub fn to_other_color<D, M>(&self, color_map: M) -> TextProps<D>
    where
        D: Clone,
        M: Fn(&C) -> D,
    {
        TextProps {
            font_size: self.font_size,
            font: self.font.clone(),
            fill: self.fill.as_ref().map(|c| color_map(c)),
            outline: self.outline.as_ref().map(|(c, w)| (color_map(c), *w)),
            underline: self.underline,
            strikeout: self.strikeout,
        }
    }
}

/// A color that has meaning for the foreground
/// (e.g. a font color)
pub trait Foreground {
    fn foreground() -> Self;
}

impl Foreground for ColorU8 {
    fn foreground() -> Self {
        color::BLACK
    }
}

impl<C> TextProps<C>
where
    C: Color + Foreground,
{
    pub fn new(font_size: f32) -> TextProps<C> {
        TextProps {
            font_size,
            font: font::Font::default(),
            fill: Some(C::foreground()),
            outline: None,
            underline: false,
            strikeout: false,
        }
    }
}

impl<C> TextProps<C>
where
    C: Clone,
{
    pub fn with_font(mut self, font: font::Font) -> Self {
        self.font = font;
        self
    }

    pub fn with_fill(mut self, fill: Option<C>) -> Self {
        self.fill = fill;
        self
    }

    pub fn with_outline(mut self, stroke: (C, f32)) -> Self {
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

    pub fn fill(&self) -> Option<C> {
        self.fill.clone()
    }

    pub fn outline(&self) -> Option<(C, f32)> {
        self.outline.clone()
    }

    pub fn underline(&self) -> bool {
        self.underline
    }

    pub fn strikeout(&self) -> bool {
        self.strikeout
    }

    fn apply_opts(&mut self, opts: &TextOptProps<C>) {
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
        if let Some(fill) = opts.fill.as_ref() {
            self.fill = Some(fill.clone());
        }
        if let Some(stroke) = opts.stroke.as_ref() {
            self.outline = Some(stroke.clone());
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
struct TextSpan<C> {
    start: usize,
    end: usize,
    props: TextOptProps<C>,
}

/// A line of rich text
#[derive(Debug, Clone)]
pub struct LineSpan<C>
where
    C: Clone,
{
    start: usize,
    end: usize,
    shapes: Vec<ShapeSpan<C>>,
    main_dir: rustybuzz::Direction,
    bbox: Option<geom::Rect>,
}

impl<C> LineSpan<C>
where
    C: Clone,
{
    /// Convert this LineSpan to another color type using the provided mapping function
    pub fn to_other_color<D, M>(&self, color_map: M) -> LineSpan<D>
    where
        D: Clone,
        M: Fn(&C) -> D,
    {
        LineSpan {
            start: self.start,
            end: self.end,
            shapes: self
                .shapes
                .iter()
                .map(|s| s.to_other_color(&color_map))
                .collect(),
            main_dir: self.main_dir,
            bbox: self.bbox,
        }
    }

    /// Byte index where the line starts in the text
    pub fn start(&self) -> usize {
        self.start
    }

    /// Byte index where the line ends in the text
    pub fn end(&self) -> usize {
        self.end
    }

    /// The text shapes in this line
    pub fn shapes(&self) -> &[ShapeSpan<C>] {
        &self.shapes
    }

    /// The main text direction of this line
    pub fn main_dir(&self) -> rustybuzz::Direction {
        self.main_dir
    }

    /// Bounding box of the line
    pub fn bbox(&self) -> Option<geom::Rect> {
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
    pub fn visual_bbox(&self) -> Option<geom::Rect> {
        if self.shapes.is_empty() {
            return None;
        }
        let mut bbox = None;
        for s in &self.shapes {
            bbox = geom::Rect::unite_opt(bbox.as_ref(), Some(&s.visual_bbox()));
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
pub struct ShapeSpan<C>
where
    C: Clone,
{
    start: usize,
    end: usize,
    spans: Vec<PropsSpan<C>>,
    face_id: fontdb::ID,
    glyphs: Vec<Glyph>,
    metrics: font::ScaledMetrics,
    y_baseline: f32,
    bbox: Option<geom::Rect>,
}

impl<C> ShapeSpan<C>
where
    C: Clone,
{
    /// Convert this ShapeSpan to another color type using the provided mapping function
    pub fn to_other_color<D, M>(&self, color_map: M) -> ShapeSpan<D>
    where
        D: Clone,
        M: Fn(&C) -> D,
    {
        ShapeSpan {
            start: self.start,
            end: self.end,
            spans: self
                .spans
                .iter()
                .map(|s| s.to_other_color(&color_map))
                .collect(),
            face_id: self.face_id,
            glyphs: self.glyphs.clone(),
            metrics: self.metrics,
            y_baseline: self.y_baseline,
            bbox: self.bbox,
        }
    }

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
    pub fn spans(&self) -> &[PropsSpan<C>] {
        &self.spans
    }

    /// The metrics of this shape
    pub fn metrics(&self) -> font::ScaledMetrics {
        self.metrics
    }

    /// The bounding box of this shape
    pub fn bbox(&self) -> geom::Rect {
        // no empty shapes are built
        self.bbox.unwrap()
    }

    /// The visual bounding box of this shape
    pub fn visual_bbox(&self) -> geom::Rect {
        // no empty shapes are built
        assert!(!self.glyphs.is_empty());

        let mut bbox = None;
        for g in &self.glyphs {
            match bbox {
                Some(ref mut b) => {
                    *b = geom::Rect::unite(b, &g.visual_bbox());
                }
                None => {
                    bbox = Some(g.visual_bbox());
                }
            }
        }
        bbox.unwrap()
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
pub struct PropsSpan<C>
where
    C: Clone,
{
    start: usize,
    end: usize,
    props: TextProps<C>,
    bbox: Option<geom::Rect>,
}

impl<C> PropsSpan<C>
where
    C: Clone,
{
    /// Convert this PropSpan to another color type using the provided mapping function
    pub fn to_other_color<D, M>(&self, color_map: M) -> PropsSpan<D>
    where
        D: Clone,
        M: Fn(&C) -> D,
    {
        PropsSpan {
            start: self.start,
            end: self.end,
            props: self.props.to_other_color(color_map),
            bbox: self.bbox,
        }
    }

    /// Byte index where the span starts in the text
    pub fn start(&self) -> usize {
        self.start
    }

    /// Byte index where the span ends in the text
    pub fn end(&self) -> usize {
        self.end
    }

    /// The properties of this span
    pub fn props(&self) -> &TextProps<C> {
        &self.props
    }

    /// Bounding box of the span
    pub fn bbox(&self) -> geom::Rect {
        // no empty spans are built
        self.bbox.unwrap()
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
    fn visual_bbox(&self) -> geom::Rect {
        let mut tl_br = [
            geom::Point {
                x: self.rect.x_min as f32,
                y: self.rect.y_max as f32,
            },
            geom::Point {
                x: self.rect.x_max as f32,
                y: self.rect.y_min as f32,
            },
        ];
        self.ts.map_points(&mut tl_br);
        geom::Rect::from_trbl(tl_br[0].y, tl_br[1].x, tl_br[1].y, tl_br[0].x)
    }
}
