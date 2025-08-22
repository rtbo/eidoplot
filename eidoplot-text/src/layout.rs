use tiny_skia_path::Transform;
use ttf_parser as ttf;

use crate::font::{self, Font};
use crate::shape::{self, Direction, MainDirection, TextShape};

/// Bounding box of text layout.
/// It is expressed relatively to the anchor (or left of anchor when [Anchor::Window] is used)
#[derive(Debug, Clone, Copy)]
pub struct BBox {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl BBox {
    const fn unite(bb1: &BBox, bb2: &BBox) -> BBox {
        BBox {
            top: bb1.top.min(bb2.top),
            right: bb1.right.max(bb2.right),
            bottom: bb1.bottom.max(bb2.bottom),
            left: bb1.left.min(bb2.left),
        }
    }

    const EMPTY: BBox = BBox {
        top: f32::MAX,
        right: f32::MIN,
        bottom: f32::MIN,
        left: f32::MAX,
    };

    pub const fn is_empty(&self) -> bool {
        self.top >= self.bottom || self.left >= self.right
    }

    pub const fn translate(self, x: f32, y: f32) -> BBox {
        BBox {
            top: self.top + y,
            right: self.right + x,
            bottom: self.bottom + y,
            left: self.left + x,
        }
    }

    pub const fn width(&self) -> f32 {
        self.right - self.left
    }

    pub const fn height(&self) -> f32 {
        self.bottom - self.top
    }

    pub fn transform(self, transform: &Transform) -> BBox {
        let mut top_left = tiny_skia_path::Point {
            x: self.left,
            y: self.top,
        };
        let mut bottom_right = tiny_skia_path::Point {
            x: self.right,
            y: self.bottom,
        };
        transform.map_point(&mut top_left);
        transform.map_point(&mut bottom_right);
        BBox {
            top: top_left.y,
            right: bottom_right.x,
            bottom: bottom_right.y,
            left: top_left.x,
        }
    }
}

impl Default for BBox {
    fn default() -> Self {
        BBox::EMPTY
    }
}

/// Horizontal alignment
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HorAlign {
    /// Align the start of the text (left or right depending on the direction)
    #[default]
    Start,
    /// Left align the text (independently of the direction)
    Left,
    /// Center align the text
    Center,
    /// Align the end of the text (left or right depending on the direction)
    End,
    /// Right align the text (independently of the direction)
    Right,
}

/// Anchor where to align the text 
/// By default it is a point (0, 0) 
/// Keep in mind that the anchor is relative and that a transform applies on top of it
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    /// Anchor at a point
    /// The LTR text with HorAlign::Start will start at this point and span to the right
    /// The RTL text with HorAlign::Start will start at this point and span to the left
    Point,
    /// Anchor in a horizontal window defined by its width
    /// The following cases will be align at the left of the window and span to the right:
    ///     - Any text with [HorAlign::Left]
    ///     - LTR text with [HorAlign::Start]
    ///     - RTL text with [HorAlign::End]
    /// The following cases will be align at the right of the window and span to the left:
    ///     - Any text with [HorAlign::Right]
    ///     - LTR text with [HorAlign::End]
    ///     - RTL text with [HorAlign::Start]
    /// Centered text will be centered in the window
    /// No check is made that the text fits in the window, and no shrinking is done
    Window(f32),
}

impl Default for Anchor {
    fn default() -> Self {
        Anchor::Point
    }
}

/// Vertical alignment for a single line of text
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineVerAlign {
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

/// Vertical alignment for a whole text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerAlign {
    /// Align at the specified line
    Line(usize, LineVerAlign),
    /// Align at the top (ascender) of the first line
    Top,
    /// Align at the center, that is (top + bottom) / 2
    Center,
    /// Align at the bottom (descender) of the last line
    Bottom,
}

impl Default for VerAlign {
    fn default() -> Self {
        VerAlign::Line(0, LineVerAlign::default())
    }
}

/// Convert a LineVerAlign to a VerAlign at line 0 (or for a single line of text)
/// 
/// # Example
/// ```
/// use eidoplot_text::{LineVerAlign, VerAlign};
/// let align: VerAlign = LineVerAlign::Hanging.into();
/// assert_eq!(align, VerAlign::Line(0, LineVerAlign::Hanging));
/// ```
impl From<LineVerAlign> for VerAlign {
    fn from(l: LineVerAlign) -> Self {
        VerAlign::Line(0, l)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Options {
    pub anchor: Anchor,
    pub hor_align: HorAlign,
    pub ver_align: VerAlign,
    /// Justify the text horizontally
    /// For point anchor, justifies to the maximum line width (equals text width)
    /// For window anchor, justifies to the window width (overrules hor_align)
    pub hor_justify: bool,
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub id: ttf::GlyphId,
    pub ts: tiny_skia_path::Transform,
    dbg: shape::GlyphDbg,
}

impl Glyph {
    /// Sequence of characters for debugging.
    /// This function is always available, but the Option is filled
    /// only when the debug assertions are enabled.
    pub fn dbg_chars(&self) -> Option<&str> {
        self.dbg.chars()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GlyphRun {
    pub(crate) font_id: font::ID,
    pub(crate) glyphs: Vec<Glyph>,
}

#[derive(Debug, Clone)]
pub(crate) struct LineLayout {
    pub(crate) runs: Vec<GlyphRun>,
    bbox: BBox,
    direction: Direction,
}

#[derive(Debug, Clone)]
pub struct TextLayout {
    pub(crate) lines: Vec<LineLayout>,
    text: String,
    font: Font,
    font_size: f32,
    opts: Options,
    bbox: BBox,
}

impl TextLayout {
    pub fn from_shape(text_shape: &TextShape, font_size: f32, opts: &Options) -> Self {
        match &text_shape.lines {
            shape::Lines::SingleFont(lines) => {
                layout_text(lines, text_shape.text(), text_shape.font(), font_size, opts)
            }
            shape::Lines::Fallback(lines) => {
                layout_text(lines, text_shape.text(), text_shape.font(), font_size, opts)
            }
        }
    }

    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    pub fn font(&self) -> &Font {
        &self.font
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn lines_len(&self) -> usize {
        self.lines.len()
    }

    pub fn bbox(&self) -> BBox {
        self.bbox
    }


    pub fn line_bbox(&self, lidx: usize) -> BBox {
        self.lines[lidx].bbox
    }

    pub fn direction(&self) -> Direction {
        self.lines.first().map(|l| l.direction).unwrap_or_default()
    }

    pub fn line_direction(&self, lidx: usize) -> Direction {
        self.lines[lidx].direction
    }

    /// Options used to build this layout
    pub fn options(&self) -> &Options {
        &self.opts
    }
}

#[derive(Debug, Clone, Copy)]
struct ScaledGlyph {
    /// The id of the glyph in the font face
    id: ttf::GlyphId,
    /// The scaled x-offset of the glyph
    x_offset: f32,
    /// The scaled y-offset of the glyph
    y_offset: f32,
    /// The scaled x-advance of the glyph
    x_advance: f32,
    /// The scaled y-advance of the glyph
    y_advance: f32,
}

impl From<(shape::Glyph, f32)> for ScaledGlyph {
    fn from((glyph, scale): (shape::Glyph, f32)) -> Self {
        Self {
            id: glyph.id,
            x_offset: glyph.x_offset as f32 * scale,
            y_offset: glyph.y_offset as f32 * scale,
            x_advance: glyph.x_advance as f32 * scale,
            y_advance: glyph.y_advance as f32 * scale,
        }
    }
}
trait LineTrait {
    fn glyphs_len(&self) -> usize;
    fn glyph(&self, gidx: usize) -> Option<shape::Glyph>;
}

trait LinesTrait {
    type Line: LineTrait + shape::MainDirection;

    fn lines(&self) -> &[Self::Line];

    fn line_glyph_font_id(&self, lidx: usize, gidx: usize) -> Option<font::ID>;
    fn line_glyph_scale(&self, lidx: usize, gidx: usize, font_size: f32) -> Option<f32>;
    fn line_scaled_metrics(&self, idx: usize, font_size: f32) -> Option<font::ScaledFaceMetrics>;
    /// Y value of the baseline of a line.
    /// Relatively to the baseline of the first line
    fn line_scaled_baseline(&self, lidx: usize, font_size: f32) -> f32;
    fn line_scaled_height(&self, lidx: usize, font_size: f32) -> f32;
    fn line_scaled_gap(&self, lidx: usize, font_size: f32) -> f32;
    fn line_scaled_width(&self, lidx: usize, font_size: f32) -> f32;
    fn scaled_width(&self, font_size: f32) -> f32 {
        (0..self.lines().len())
            .map(|l| self.line_scaled_width(l, font_size))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }
}

mod single_font {
    use crate::font;
    use crate::shape;
    use crate::shape::single_font::{Line, Lines};

    impl super::LineTrait for Line {
        fn glyphs_len(&self) -> usize {
            self.glyphs.len()
        }

        fn glyph(&self, gidx: usize) -> Option<shape::Glyph> {
            Some(self.glyphs[gidx])
        }
    }

    impl super::LinesTrait for Lines {
        type Line = Line;

        fn lines(&self) -> &[Line] {
            &self.lines
        }

        fn line_glyph_font_id(&self, _lidx: usize, _gidx: usize) -> Option<font::ID> {
            Some(self.font)
        }

        fn line_glyph_scale(&self, _lidx: usize, _gidx: usize, font_size: f32) -> Option<f32> {
            Some(self.metrics.scale(font_size))
        }

        fn line_scaled_metrics(
            &self,
            _idx: usize,
            font_size: f32,
        ) -> Option<font::ScaledFaceMetrics> {
            Some(self.metrics.scaled(font_size))
        }

        fn line_scaled_baseline(&self, lidx: usize, font_size: f32) -> f32 {
            let scale = self.metrics.scale(font_size);
            let gap = self.metrics.line_gap;
            let height = self.metrics.height();
            (gap + height) as f32 * scale * lidx as f32
        }

        fn line_scaled_height(&self, _lidx: usize, font_size: f32) -> f32 {
            self.metrics.scaled_height(font_size)
        }

        fn line_scaled_gap(&self, _lidx: usize, font_size: f32) -> f32 {
            self.metrics.scaled_line_gap(font_size)
        }

        fn line_scaled_width(&self, lidx: usize, font_size: f32) -> f32 {
            let mut x_advance = 0;
            for glyph in &self.lines[lidx].glyphs {
                x_advance += glyph.x_advance;
            }
            x_advance as f32 * self.metrics.scale(font_size)
        }
    }
}

mod fallback_font {
    use crate::font;
    use crate::shape;
    use crate::shape::fallback::{Glyph, Line, Lines};

    fn glyph_scaled_height(glyph: &Glyph, font_size: f32) -> f32 {
        match glyph {
            Glyph::Missing(..) => 0.0,
            Glyph::Resolved(_, _, metrics) => metrics.scaled_height(font_size),
        }
    }

    fn glyph_scaled_line_gap(glyph: &Glyph, font_size: f32) -> f32 {
        match glyph {
            Glyph::Missing(..) => 0.0,
            Glyph::Resolved(_, _, metrics) => metrics.scaled_line_gap(font_size),
        }
    }

    fn glyph_scaled_x_advance(glyph: &Glyph, font_size: f32) -> f32 {
        // FIXME: advance of replacement char?
        match glyph {
            Glyph::Missing(..) => 0.0,
            Glyph::Resolved(glyph, _, metrics) => glyph.x_advance as f32 * metrics.scale(font_size),
        }
    }

    fn line_scaled_height(line: &Line, font_size: f32) -> f32 {
        line.glyphs
            .iter()
            .map(|g| glyph_scaled_height(g, font_size))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn line_scaled_gap(line: &Line, font_size: f32) -> f32 {
        line.glyphs
            .iter()
            .map(|g| glyph_scaled_line_gap(g, font_size))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    impl super::LineTrait for Line {
        fn glyphs_len(&self) -> usize {
            self.glyphs.len()
        }

        fn glyph(&self, gidx: usize) -> Option<shape::Glyph> {
            match &self.glyphs[gidx] {
                Glyph::Missing(..) => None,
                Glyph::Resolved(glyph, _, _) => Some(*glyph),
            }
        }
    }

    impl super::LinesTrait for Lines {
        type Line = Line;

        fn lines(&self) -> &[Line] {
            &self.lines
        }

        fn line_glyph_font_id(&self, line_idx: usize, gidx: usize) -> Option<font::ID> {
            match &self.lines[line_idx].glyphs[gidx] {
                Glyph::Missing(..) => None,
                Glyph::Resolved(_, id, _) => Some(*id),
            }
        }

        fn line_glyph_scale(&self, lidx: usize, gidx: usize, font_size: f32) -> Option<f32> {
            match &self.lines[lidx].glyphs[gidx] {
                Glyph::Missing(..) => None,
                Glyph::Resolved(_, _, metrics) => Some(metrics.scale(font_size)),
            }
        }

        fn line_scaled_metrics(
            &self,
            idx: usize,
            font_size: f32,
        ) -> Option<font::ScaledFaceMetrics> {
            let line = &self.lines[idx];
            if line.glyphs.is_empty() {
                return None;
            }

            let mut metrics = font::ScaledFaceMetrics::null();

            for g in &line.glyphs {
                match g {
                    Glyph::Missing(..) => {}
                    Glyph::Resolved(_, _, m) => {
                        let m = m.scaled(font_size);
                        metrics.ascent = metrics.ascent.max(m.ascent);
                        metrics.descent = metrics.descent.max(m.descent);
                        metrics.x_height = metrics.x_height.max(m.x_height);
                        metrics.cap_height = metrics.cap_height.max(m.cap_height);
                        metrics.line_gap = metrics.line_gap.max(m.line_gap);
                    }
                }
            }
            Some(metrics)
        }

        fn line_scaled_baseline(&self, lidx: usize, font_size: f32) -> f32 {
            let mut h = 0.0;
            let mut l = 0;
            while l < lidx {
                h += line_scaled_gap(&self.lines[l], font_size);
                h += line_scaled_height(&self.lines[l + 1], font_size);
                l += 1;
            }
            h
        }

        fn line_scaled_height(&self, lidx: usize, font_size: f32) -> f32 {
            line_scaled_height(&self.lines[lidx], font_size)
        }

        fn line_scaled_gap(&self, lidx: usize, font_size: f32) -> f32 {
            line_scaled_gap(&self.lines[lidx], font_size)
        }

        fn line_scaled_width(&self, lidx: usize, font_size: f32) -> f32 {
            self.lines[lidx]
                .glyphs
                .iter()
                .map(|g| glyph_scaled_x_advance(g, font_size))
                .sum()
        }
    }
}

fn layout_text<L>(lines: &L, text: &str, font: &Font, font_size: f32, opts: &Options) -> TextLayout
where
    L: LinesTrait,
{
    if lines.lines().is_empty() {
        todo!()
    }

    let lines_len = lines.lines().len();

    let fst_metrics = lines.line_scaled_metrics(0, font_size).unwrap();
    let lst_metrics = lines.line_scaled_metrics(lines_len - 1, font_size).unwrap();

    // y-cursor must be placed at the baseline of the first line
    let mut y_cursor = match opts.ver_align {
        VerAlign::Top => fst_metrics.ascent,
        VerAlign::Bottom => {
            lst_metrics.descent - lines.line_scaled_baseline(lines_len - 1, font_size)
        }
        VerAlign::Center => {
            let top = fst_metrics.ascent;
            let bottom = lst_metrics.descent - lines.line_scaled_baseline(lines_len - 1, font_size);
            (top + bottom) / 2.0
        }
        VerAlign::Line(line, align) => {
            let baseline = lines.line_scaled_baseline(line, font_size);
            match align {
                LineVerAlign::Bottom => lst_metrics.descent - baseline,
                LineVerAlign::Baseline => -baseline,
                LineVerAlign::Middle => lst_metrics.x_height / 2.0 - baseline,
                LineVerAlign::Hanging => lst_metrics.cap_height - baseline,
                LineVerAlign::Top => lst_metrics.ascent - baseline,
            }
        }
    };

    let justify = if opts.hor_justify {
        match opts.anchor {
            Anchor::Point => Some(lines.scaled_width(font_size)),
            Anchor::Window(width) => Some(width),
        }
    } else {
        None
    };

    let line_align = LineAlign {
        anchor: opts.anchor,
        hor_align: opts.hor_align,
        justify,
    };

    let mut line_vec = Vec::with_capacity(lines_len);

    for lidx in 0..lines_len {
        if lidx != 0 {
            y_cursor += lines.line_scaled_height(lidx, font_size);
        }

        let l = layout_line_at_baseline(y_cursor, lines, lidx, font_size, line_align);
        line_vec.push(l);

        y_cursor += lines.line_scaled_gap(lidx, font_size);
    }

    let bbox = line_vec
        .iter()
        .map(|l| l.bbox)
        .reduce(|a, b| BBox::unite(&a, &b));
    let bbox = bbox.unwrap_or_default();

    TextLayout {
        lines: line_vec,
        text: text.to_string(),
        font: font.clone(),
        font_size,
        bbox,
        opts: *opts,
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct LineAlign {
    hor_align: HorAlign,
    anchor: Anchor,
    justify: Option<f32>,
}

fn layout_line_at_baseline<T>(
    y_baseline: f32,
    text: &T,
    lidx: usize,
    font_size: f32,
    align: LineAlign,
) -> LineLayout
where
    T: LinesTrait,
{
    let line = &text.lines()[lidx];
    let width = text.line_scaled_width(lidx, font_size);

    let (width, justify_fact) = match align.justify {
        Some(justify) => {
            if justify <= width {
                (width, 1.0)
            } else {
                (justify, justify / width)
            }
        }
        None => (width, 1.0),
    };

    let (anchor_left, anchor_right) = match align.anchor {
        Anchor::Point => (0.0, 0.0),
        Anchor::Window(width) => (0.0, width),
    };

    let x_start = match (align.hor_align, line.main_direction()) {
        (HorAlign::Start, Direction::LTR)
        | (HorAlign::End, Direction::RTL)
        | (HorAlign::Left, _) => anchor_left,
        (HorAlign::Center, _) => (anchor_left + anchor_right) / 2.0 - width / 2.0,
        (HorAlign::Start, Direction::RTL)
        | (HorAlign::End, Direction::LTR)
        | (HorAlign::Right, _) => anchor_right - width,
    };

    let mut x_cursor = x_start;
    let mut y_cursor = y_baseline;

    let mut glyph_runs = Vec::new();
    let mut glyphs = Vec::new();
    let mut font_id = None;

    let y_flip: Transform = Transform::from_scale(1.0, -1.0);

    for gidx in 0..line.glyphs_len() {
        let Some(gl_font_id) = text.line_glyph_font_id(lidx, gidx) else {
            continue;
        };
        let shape_gl = line.glyph(gidx).unwrap();
        let scale = text.line_glyph_scale(lidx, gidx, font_size).unwrap();

        let scale_ts = Transform::from_scale(scale, scale);
        let scaled_gl = ScaledGlyph::from((shape_gl, scale));

        let x = x_cursor + scaled_gl.x_offset;
        let y = y_cursor + scaled_gl.y_offset;
        let pos_ts = Transform::from_translate(x, y);

        let gl = Glyph {
            id: scaled_gl.id,
            ts: y_flip.post_concat(scale_ts).post_concat(pos_ts),
            dbg: shape_gl.dbg,
        };

        if let Some(font_id) = font_id {
            if gl_font_id != font_id {
                glyph_runs.push(GlyphRun {
                    font_id: gl_font_id,
                    glyphs: glyphs,
                });
                glyphs = vec![gl];
            } else {
                glyphs.push(gl);
            }
        } else {
            glyphs.push(gl);
        }

        font_id = Some(gl_font_id);

        x_cursor += scaled_gl.x_advance * justify_fact;
        y_cursor += scaled_gl.y_advance;
    }

    if glyphs.len() > 0 {
        glyph_runs.push(GlyphRun {
            font_id: font_id.unwrap(),
            glyphs,
        });
    }

    let bbox = text.line_scaled_metrics(lidx, font_size).map(|m| BBox {
        left: x_start,
        right: x_cursor,
        top: y_baseline - m.ascent,
        bottom: y_baseline - m.descent,
    });
    let bbox = bbox.unwrap_or_default();

    LineLayout {
        runs: glyph_runs,
        bbox,
        direction: line.main_direction(),
    }
}
