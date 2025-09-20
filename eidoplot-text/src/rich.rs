use crate::{
    font::{self, DatabaseExt},
    fontdb,
};
use std::fmt;
use ttf_parser as ttf;

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

/// Typographic alignment.
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

/// Vertical alignment for a single line of text
#[derive(Debug, Clone, Copy, Default)]
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

#[derive(Debug, Clone, Copy)]
pub enum Layout {
    Horizontal(TypeAlign, LineAlign, Direction),
    Vertical(TypeAlign, VerDirection, VerProgression),
}

impl Default for Layout {
    fn default() -> Self {
        Layout::Horizontal(TypeAlign::default(), LineAlign::default(), Direction::default())
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

/// A builder struct for rich text
#[derive(Debug, Clone)]
pub struct RichTextBuilder {
    text: String,
    init_props: TextProps,
    layout: Layout,
    spans: Vec<TextSpan>,
}

impl RichTextBuilder {
    /// Create a new RichTextBuilder
    pub fn new(text: String, init_props: TextProps) -> RichTextBuilder {
        RichTextBuilder {
            text,
            init_props,
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

    /// Create a RichTextLayout from this builder
    pub fn shape_and_layout(self, fontdb: &fontdb::Database) -> Result<RichTextLayout, Error> {
        let bidi_algo = match &self.layout {
            Layout::Horizontal(_, _, Direction::Mixed) => BidiAlgo::Yep { default_lev: None },
            Layout::Horizontal(_, _, Direction::MixedLTR) => BidiAlgo::Yep {
                default_lev: Some(unicode_bidi::LTR_LEVEL),
            },
            Layout::Horizontal(_, _, Direction::MixedRTL) => BidiAlgo::Yep {
                default_lev: Some(unicode_bidi::RTL_LEVEL),
            },
            Layout::Horizontal(_, _, Direction::LTR) => BidiAlgo::Nope(rustybuzz::Direction::LeftToRight),
            Layout::Horizontal(_, _, Direction::RTL) => BidiAlgo::Nope(rustybuzz::Direction::RightToLeft),
            Layout::Vertical(_, VerDirection::TTB, _) => BidiAlgo::Nope(rustybuzz::Direction::TopToBottom),
            Layout::Vertical(_, VerDirection::BTT, _) => BidiAlgo::Nope(rustybuzz::Direction::BottomToTop),
        };

        let resolver = PropsResolver::new(self.init_props.clone());

        let mut ctx = BuilderCtx {
            resolver,
            bidi_algo,
            buffer: None,
        };

        // spliting lines while keeping track of byte indices
        // str.lines() isn't suitable because it splits either on \n or \r\n, without knowing which
        let mut lines = Vec::new();
        let mut line_start = 0;
        for line in self.text.split_inclusive('\n') {
            let eol_len = if line.len() > 1 && line.as_bytes()[line.len() - 2] == b'\r' {
                2
            } else {
                1
            };

            lines.push(self.shape_line(
                line_start,
                line_start + line.len() - eol_len,
                fontdb,
                &mut ctx,
            )?);

            line_start += line.len();
        }

        self.build_layout(lines, fontdb)
    }
}

#[derive(Debug)]
struct BuilderCtx {
    resolver: PropsResolver,
    bidi_algo: BidiAlgo,
    buffer: Option<rustybuzz::UnicodeBuffer>,
}

#[derive(Debug)]
struct PropsResolver {
    init_props: TextProps,
    stack: Vec<TextOptProps>,
}

impl PropsResolver {
    fn new(init_props: TextProps) -> PropsResolver {
        PropsResolver {
            init_props,
            stack: Vec::new(),
        }
    }

    fn resolved(&self) -> TextProps {
        let mut props = self.init_props.clone();
        for opts in self.stack.iter() {
            props.apply_opts(opts);
        }
        props
    }

    fn push_opts(&mut self, opts: TextOptProps) {
        self.stack.push(opts);
    }

    fn pop_opts(&mut self, opts: TextOptProps) {
        for i in (0..self.stack.len()).rev() {
            if &self.stack[i] == &opts {
                self.stack.remove(i);
                break;
            }
        }
    }
}

#[derive(Debug)]
struct BidiRun {
    start: usize,
    end: usize,
    dir: rustybuzz::Direction,
}

#[derive(Debug)]
enum BidiAlgo {
    Nope(rustybuzz::Direction),
    Yep {
        default_lev: Option<unicode_bidi::Level>,
    },
}

impl BidiAlgo {
    fn start_dir(&self) -> rustybuzz::Direction {
        match self {
            BidiAlgo::Nope(dir) => *dir,
            BidiAlgo::Yep { default_lev } => {
                if let Some(lev) = default_lev {
                    if lev.is_rtl() {
                        rustybuzz::Direction::RightToLeft
                    } else {
                        rustybuzz::Direction::LeftToRight
                    }
                } else {
                    rustybuzz::Direction::LeftToRight
                }
            }
        }
    }

    fn visual_runs(&mut self, text: &str, start: usize) -> Vec<BidiRun> {
        match self {
            BidiAlgo::Nope(dir) => vec![BidiRun {
                start,
                end: start + text.len(),
                dir: *dir,
            }],
            BidiAlgo::Yep { default_lev } => {
                let bidi = unicode_bidi::BidiInfo::new(text, *default_lev);
                let mut res_runs = Vec::new();

                for para in &bidi.paragraphs {
                    let line = para.range.clone();
                    let (levels, runs) = bidi.visual_runs(para, line);
                    for run in runs {
                        let lev = levels[run.start];
                        let dir = if lev.is_rtl() {
                            rustybuzz::Direction::RightToLeft
                        } else {
                            rustybuzz::Direction::LeftToRight
                        };
                        if default_lev.is_none() {
                            // assign for following lines
                            *default_lev = Some(lev);
                        }
                        res_runs.push(BidiRun {
                            start: start + run.start,
                            end: start + run.end,
                            dir,
                        })
                    }
                }

                if res_runs.is_empty() {
                    let dir = if let Some(lev) = default_lev {
                        if lev.is_rtl() {
                            rustybuzz::Direction::RightToLeft
                        } else {
                            rustybuzz::Direction::LeftToRight
                        }
                    } else {
                        rustybuzz::Direction::LeftToRight
                    };
                    res_runs.push(BidiRun {
                        start,
                        end: start + text.len(),
                        dir,
                    });
                }
                res_runs
            }
        }
    }
}

#[derive(Debug)]
struct TextLine {
    start: usize,
    end: usize,
    shapes: Vec<ShapeSpan>,
}

impl TextLine {
    fn metrics(&self) -> font::ScaledMetrics {
        let mut metrics = font::ScaledMetrics::null();
        for s in &self.shapes {
            metrics.ascent = metrics.ascent.max(s.metrics.ascent);
            metrics.descent = metrics.descent.max(s.metrics.descent);
            metrics.x_height = metrics.x_height.max(s.metrics.x_height);
            metrics.cap_height = metrics.cap_height.max(s.metrics.cap_height);
            metrics.line_gap = metrics.line_gap.max(s.metrics.line_gap);
        }
        metrics
    }
}

#[derive(Debug)]
struct ShapeSpan {
    start: usize,
    end: usize,
    spans: Vec<PropsSpan>,
    face_id: fontdb::ID,
    glyphs: Vec<Glyph>,
    metrics: font::ScaledMetrics,
}

#[derive(Debug, Clone)]
struct PropsSpan {
    start: usize,
    end: usize,
    props: TextProps,
}

#[derive(Debug)]
struct Glyph {
    id: ttf::GlyphId,
    cluster: usize,
    x_advance: i32,
    y_advance: i32,
    x_offset: i32,
    y_offset: i32,
    ts: tiny_skia::Transform,
}

#[derive(Debug, Clone)]
pub struct RichTextLayout {
    text: String,
}

impl RichTextLayout {
    fn empty() -> Self {
        Self {
            text: String::new(),
        }
    }
}

impl RichTextBuilder {
    fn shape_line(
        &self,
        start: usize,
        end: usize,
        fontdb: &fontdb::Database,
        ctx: &mut BuilderCtx,
    ) -> Result<TextLine, Error> {
        debug_assert!(self.text.is_char_boundary(start) && self.text.is_char_boundary(end));
        let line_txt = &self.text[start..end];

        // We create a flat list of shapes. Each of the following change is a shape boundary:
        //  - a change of font property
        //  - a change of text direction (LTR or RTL)
        //  - a paragraph separator (unlikely to happen as lines are already split)

        let mut cur_dir = ctx.bidi_algo.start_dir();
        let bidi_runs = ctx.bidi_algo.visual_runs(line_txt, start);

        let mut cur_start = start;
        let mut shapes = Vec::new();
        for (i, _) in line_txt.char_indices() {
            let i = i + start;
            let mut bump_start = false;
            for run in bidi_runs.iter() {
                if (i == run.start || i == run.end) && run.start != run.end {
                    shapes.push(self.shape_span(cur_start, i, cur_dir, fontdb, ctx)?);
                    cur_dir = run.dir;
                    bump_start = true;
                }
            }
            for span in self.spans.iter().filter(|s| s.props.affect_shape()) {
                if (i == span.start || i == span.end) && span.start != span.end {
                    shapes.push(self.shape_span(cur_start, i, cur_dir, fontdb, ctx)?);
                    if i == span.start {
                        ctx.resolver.push_opts(span.props.clone());
                    }
                    if i == span.end {
                        ctx.resolver.pop_opts(span.props.clone());
                    }
                    bump_start = true;
                }
            }
            if bump_start {
                cur_start = i;
            }
        }
        if cur_start != end {
            shapes.push(self.shape_span(cur_start, end, cur_dir, fontdb, ctx)?);
        }

        Ok(TextLine { start, end, shapes })
    }

    fn shape_span(
        &self,
        start: usize,
        end: usize,
        dir: rustybuzz::Direction,
        fontdb: &fontdb::Database,
        ctx: &mut BuilderCtx,
    ) -> Result<ShapeSpan, Error> {
        debug_assert!(self.text.is_char_boundary(start) && self.text.is_char_boundary(end));

        let mut cur_start = start;
        let mut props_spans = Vec::new();
        let txt = &self.text[start..end];
        for (i, _) in txt.char_indices() {
            let i = i + start;
            for span in self.spans.iter().filter(|s| !s.props.affect_shape()) {
                if (i == span.start || i == span.end) && span.start != span.end {
                    props_spans.push(PropsSpan {
                        start: cur_start,
                        end: i,
                        props: ctx.resolver.resolved(),
                    });
                    if i == span.start {
                        ctx.resolver.push_opts(span.props.clone());
                    }
                    if i == span.end {
                        ctx.resolver.pop_opts(span.props.clone());
                    }
                    cur_start = i;
                }
            }
        }
        if cur_start != end {
            props_spans.push(PropsSpan {
                start: cur_start,
                end,
                props: ctx.resolver.resolved(),
            });
        }
        // font and font_size are identical in all the subspans
        let shape_props = ctx.resolver.resolved();
        let face_id = fontdb
            .select_face_for_str(&shape_props.font, txt)
            .or_else(|| fontdb.select_face(&shape_props.font))
            .ok_or(Error::NoSuchFont(shape_props.font.clone()))?;

        let mut buffer = ctx
            .buffer
            .take()
            .unwrap_or_else(|| rustybuzz::UnicodeBuffer::new());
        buffer.push_str(txt);
        if start != 0 {
            buffer.set_pre_context(&self.text[..start]);
        }
        if end != self.text.len() {
            buffer.set_post_context(&self.text[end..]);
        }
        buffer.set_direction(dir);
        buffer.guess_segment_properties();

        let (shape, metrics) = fontdb
            .with_face_data(
                face_id,
                |data, index| -> Result<_, Error> {
                    let face = ttf::Face::parse(data, index)?;
                    let metrics = font::face_metrics(&face).scaled(shape_props.font_size);
                    let mut hbface = rustybuzz::Face::from_face(face);
                    font::apply_hb_variations(&mut hbface, &shape_props.font);

                    let kern = rustybuzz::Feature::new(ttf::Tag::from_bytes(b"kern"), 1, ..);
                    Ok((rustybuzz::shape(&hbface, &[kern], buffer), metrics))
                },
            )
            .expect("should be a valid face id")?;

        let mut glyphs = Vec::with_capacity(shape.len());
        for (i, p) in shape.glyph_infos().iter().zip(shape.glyph_positions()) {
            glyphs.push(Glyph {
                id: ttf::GlyphId(i.glyph_id as u16),
                cluster: i.cluster as usize + start,
                x_advance: p.x_advance,
                y_advance: p.y_advance,
                x_offset: p.x_offset,
                y_offset: p.y_offset,
                ts: tiny_skia::Transform::identity(),
            })
        }

        ctx.buffer = Some(shape.clear());

        Ok(ShapeSpan {
            start,
            end,
            spans: props_spans,
            face_id,
            glyphs,
            metrics,
        })
    }

    fn build_layout(self, lines: Vec<TextLine>, fontdb: &fontdb::Database) -> Result<RichTextLayout, Error> {
        if lines.is_empty() {
            return Ok(RichTextLayout::empty());
        }

        match self.layout {
            Layout::Horizontal(..) => self.build_horizontal_layout(lines, fontdb),
            Layout::Vertical(..) => self.build_vertical_layout(lines, fontdb),
        }
    }

    fn build_horizontal_layout(self, lines: Vec<TextLine>, fontdb: &fontdb::Database) -> Result<RichTextLayout, Error> {
        let Layout::Horizontal(type_align, line_align, direction) = self.layout else {
            unreachable!()
        };

        let lines_len = lines.len();

        let fst_metrics = lines[0].metrics();
        let lst_metrics = lines[lines_len - 1].metrics();
        todo!()
    }

    fn build_vertical_layout(self, lines: Vec<TextLine>, fontdb: &fontdb::Database) -> Result<RichTextLayout, Error> {
        let Layout::Vertical(type_align, direction, prograssion) = self.layout else {
            unreachable!()
        };

        todo!()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let mut builder =
            RichTextBuilder::new("Some RICH text string".to_string(), TextProps::new(12.0));
        //builder.add_span(0, 5, TextOptProps::new().bold());
    }
}
