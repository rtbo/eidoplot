use super::{
    Align, Boundaries, Direction, Error, Glyph, Layout, LineAlign, PropsSpan, RichTextLayout,
    ShapeSpan, TextLine, TextOptProps, TextProps, TextSpan, TypeAlign, VerDirection,
};
use crate::bidi::BidiAlgo;
use crate::font::{self, DatabaseExt};
use crate::rich::VerProgression;
use crate::{BBox, fontdb};

use tiny_skia::Transform;
use ttf_parser as ttf;

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

impl TextOptProps {
    #[allow(dead_code)]
    fn desc(&self) -> String {
        let mut s = String::new();
        if self.font_family.is_some() {
            s.push_str("family ");
        }
        if self.font_weight.is_some() {
            s.push_str("font-weight ");
        }
        if self.font_width.is_some() {
            s.push_str("font-width ");
        }
        if self.font_style.is_some() {
            s.push_str("font-style ");
        }
        if self.font_size.is_some() {
            s.push_str("font-size ");
        }
        if self.fill.is_some() {
            s.push_str("fill ");
        }
        if self.stroke.is_some() {
            s.push_str("stroke ");
        }
        if self.underline.is_some() {
            s.push_str("underline ");
        }
        s
    }
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

    fn pop_opts(&mut self, opts: &TextOptProps) {
        for i in (0..self.stack.len()).rev() {
            if &self.stack[i] == opts {
                self.stack.remove(i);
                break;
            }
        }
    }
}
#[derive(Debug)]
enum Justify {
    Nope,
    Ws { added_gap: f32 },
    Glyph { fact: f32 },
}

impl ShapeSpan {
    fn x_advance(&self) -> f32 {
        self.glyphs.iter().map(|g| g.x_advance as f32).sum()
    }
}

// implementation specific to vertical text
impl ShapeSpan {
    fn col_y_advance(&self) -> f32 {
        self.glyphs.iter().map(|g| g.y_advance as f32).sum()
    }
}

impl TextLine {
    fn metrics(&self) -> font::ScaledMetrics {
        let mut metrics = font::ScaledMetrics::null();
        for s in &self.shapes {
            metrics.scale = metrics.scale.max(s.metrics.scale);
            metrics.ascent = metrics.ascent.max(s.metrics.ascent);
            metrics.descent = metrics.descent.max(s.metrics.descent);
            metrics.x_height = metrics.x_height.max(s.metrics.x_height);
            metrics.cap_height = metrics.cap_height.max(s.metrics.cap_height);
            metrics.line_gap = metrics.line_gap.max(s.metrics.line_gap);
        }
        metrics
    }

    fn em_size(&self) -> f32 {
        self.shapes
            .iter()
            .map(|s| s.metrics.em_size)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn ascent(&self) -> f32 {
        self.shapes
            .iter()
            .map(|s| s.metrics.ascent)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn descent(&self) -> f32 {
        self.shapes
            .iter()
            .map(|s| s.metrics.descent)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn gap(&self) -> f32 {
        self.shapes
            .iter()
            .map(|s| s.metrics.line_gap)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn height(&self) -> f32 {
        self.shapes
            .iter()
            .map(|s| s.metrics.height())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn x_advance(&self) -> f32 {
        self.shapes.iter().map(|s| s.x_advance()).sum()
    }
}

// This implementation gathers method specific to vertical text
impl TextLine {
    /// The column width if this TextLine is a vertical text column
    fn col_width(&self) -> f32 {
        self.shapes
            .iter()
            .map(|s| s.x_advance())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn col_height(&self) -> f32 {
        self.shapes.iter().map(|s| s.col_y_advance()).sum()
    }
}

trait Lines {
    fn baseline(&self, idx: usize) -> f32;
}

impl Lines for [TextLine] {
    fn baseline(&self, idx: usize) -> f32 {
        let mut h = 0.0;
        let mut l = 0;
        while l < idx {
            h += self[l].gap() + self[l].height();
            l += 1;
        }
        h
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

impl VerProgression {
    fn from_script(text: &str) -> VerProgression {
        if crate::script_is_rtl(text).unwrap_or(false) {
            VerProgression::RTL
        } else {
            VerProgression::LTR
        }
    }
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

    /// Create a RichTextLayout from this builder
    pub fn shape_and_layout(self, fontdb: &fontdb::Database) -> Result<RichTextLayout, Error> {
        if self.text.is_empty() {
            return Ok(RichTextLayout::empty());
        }

        let bidi_algo = match &self.layout {
            Layout::Horizontal(_, _, Direction::Mixed) => BidiAlgo::Yep { default_lev: None },
            Layout::Horizontal(_, _, Direction::MixedLTR) => BidiAlgo::Yep {
                default_lev: Some(unicode_bidi::LTR_LEVEL),
            },
            Layout::Horizontal(_, _, Direction::MixedRTL) => BidiAlgo::Yep {
                default_lev: Some(unicode_bidi::RTL_LEVEL),
            },
            Layout::Horizontal(_, _, Direction::LTR) => {
                BidiAlgo::Nope(rustybuzz::Direction::LeftToRight)
            }
            Layout::Horizontal(_, _, Direction::RTL) => {
                BidiAlgo::Nope(rustybuzz::Direction::RightToLeft)
            }
            Layout::Vertical(_, VerDirection::TTB, _, _) => {
                BidiAlgo::Nope(rustybuzz::Direction::TopToBottom)
            }
            Layout::Vertical(_, VerDirection::BTT, _, _) => {
                BidiAlgo::Nope(rustybuzz::Direction::BottomToTop)
            }
        };

        let resolver = PropsResolver::new(self.root_props.clone());

        let mut ctx = BuilderCtx {
            resolver,
            bidi_algo,
            buffer: None,
        };

        // spliting lines while keeping track of byte indices
        // str.lines() isn't suitable because it splits either on \n or \r\n, without knowing which
        let mut lines = Vec::new();
        let mut line_start = 0;
        let mut was_cr = false;
        for (i, c) in self.text.char_indices() {
            match c {
                '\r' => {
                    was_cr = true;
                }
                '\n' => {
                    lines.push(self.shape_line(
                        line_start,
                        if was_cr { i - 1 } else { i },
                        if was_cr { 2 } else { 1 },
                        fontdb,
                        &mut ctx,
                    )?);
                    line_start = i + 1;
                    was_cr = false;
                }
                '\u{85}' => {
                    lines.push(self.shape_line(line_start, i, 2, fontdb, &mut ctx)?);
                    line_start = i + 2;
                    was_cr = false;
                }
                '\u{2028}' | '\u{2029}' => {
                    lines.push(self.shape_line(line_start, i, 3, fontdb, &mut ctx)?);
                    line_start = i + 3;
                    was_cr = false;
                }
                _ => {
                    was_cr = false;
                }
            }
        }
        if line_start < self.text.len() {
            lines.push(self.shape_line(line_start, self.text.len(), 0, fontdb, &mut ctx)?);
        }
        self.build_layout(lines)
    }

    fn shape_line(
        &self,
        start: usize,
        end: usize,
        _eol: usize,
        fontdb: &fontdb::Database,
        ctx: &mut BuilderCtx,
    ) -> Result<TextLine, Error> {
        debug_assert!(self.text.is_char_boundary(start) && self.text.is_char_boundary(end));
        let line_txt = &self.text[start..end];

        // We create a flat list of shapes. Each of the following change is a shape boundary:
        //  - a change of font property
        //  - a change of text direction (LTR or RTL)
        //  - a paragraph separator (unlikely to happen as lines are already split)

        let main_dir = ctx.bidi_algo.start_dir();
        let mut cur_dir = main_dir;
        let bidi_runs = ctx.bidi_algo.visual_runs(line_txt, start);

        let mut boundaries = Boundaries::new(start, end);
        for run in bidi_runs.iter() {
            boundaries.check_in(run.start);
            boundaries.check_in(run.end);
        }
        for span in self.spans.iter().filter(|s| s.props.affect_shape()) {
            boundaries.check_in(span.start);
            boundaries.check_in(span.end);
        }

        let boundaries = boundaries.into_iter();
        let mut shapes = Vec::with_capacity(boundaries.len());

        for (span_start, span_end) in boundaries {
            for run in bidi_runs.iter() {
                if span_start == run.start {
                    cur_dir = run.dir;
                }
            }
            shapes.push(self.shape_span(span_start, span_end, cur_dir, fontdb, ctx)?);
        }

        Ok(TextLine {
            start,
            end,
            shapes,
            main_dir,
            bbox: BBox::EMPTY,
        })
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

        let txt = &self.text[start..end];

        let mut boundaries = Boundaries::new(start, end);
        for span in self.spans.iter() {
            boundaries.check_in(span.start);
            boundaries.check_in(span.end);
        }
        let boundaries = boundaries.into_iter();
        let mut props_spans = Vec::with_capacity(boundaries.len());

        for (span_start, span_end) in boundaries {
            for span in self.spans.iter() {
                if span.start == span_start {
                    ctx.resolver.push_opts(span.props.clone());
                }
            }
            props_spans.push(PropsSpan {
                start: span_start,
                end: span_end,
                props: ctx.resolver.resolved(),
                bbox: BBox::EMPTY,
            });
            for span in self.spans.iter() {
                if span.end == span_end {
                    ctx.resolver.pop_opts(&span.props);
                }
            }
        }

        // shape_props is only interested in the font and font_size,
        // which are all the same for the subspans within the shape
        let shape_props = &props_spans.first().unwrap().props;
        let face_id = fontdb
            .select_face_for_str(&shape_props.font, txt)
            .or_else(|| fontdb.select_face(&shape_props.font))
            .ok_or_else(|| Error::NoSuchFont(shape_props.font.clone()))?;

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
            .with_face_data(face_id, |data, index| -> Result<_, Error> {
                let face = ttf::Face::parse(data, index)?;
                let metrics = font::face_metrics(&face).scaled(shape_props.font_size);
                let mut hbface = rustybuzz::Face::from_face(face);
                font::apply_hb_variations(&mut hbface, &shape_props.font);

                Ok((rustybuzz::shape(&hbface, &[], buffer), metrics))
            })
            .expect("should be a valid face id")?;

        let mut glyphs = Vec::with_capacity(shape.len());
        for (i, p) in shape.glyph_infos().iter().zip(shape.glyph_positions()) {
            glyphs.push(Glyph {
                id: ttf::GlyphId(i.glyph_id as u16),
                cluster: i.cluster as usize + start,
                x_advance: p.x_advance as f32 * metrics.scale,
                y_advance: p.y_advance as f32 * metrics.scale,
                x_offset: p.x_offset as f32 * metrics.scale,
                y_offset: p.y_offset as f32 * metrics.scale,
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
            y_baseline: f32::NAN,
            bbox: BBox::EMPTY,
        })
    }

    fn build_layout(self, mut lines: Vec<TextLine>) -> Result<RichTextLayout, Error> {
        if lines.is_empty() {
            return Ok(RichTextLayout::empty());
        }

        match self.layout {
            Layout::Horizontal(..) => self.build_horizontal_layout(&mut lines)?,
            Layout::Vertical(..) => self.build_vertical_layout(&mut lines)?,
        }

        let bbox = lines
            .iter()
            .map(|l| l.bbox)
            .reduce(|a, b| BBox::unite(&a, &b));
        let bbox = bbox.unwrap_or_default();

        Ok(RichTextLayout {
            text: self.text,
            lines,
            bbox,
        })
    }

    fn build_horizontal_layout(&self, lines: &mut Vec<TextLine>) -> Result<(), Error> {
        let Layout::Horizontal(align, type_align, _) = self.layout else {
            unreachable!()
        };

        let lines_len = lines.len();

        // y-cursor must be placed at the baseline of the first line
        let mut y_cursor = match align {
            Align::Top => lines[0].ascent(),
            Align::Bottom => lines[lines_len - 1].descent() - lines.baseline(lines_len - 1),
            Align::Center => {
                let top = lines[0].ascent();
                let bottom = lines[lines_len - 1].descent() - lines.baseline(lines_len - 1);
                (top + bottom) / 2.0
            }
            Align::Line(line, align) => {
                let baseline = lines.baseline(line);
                let lst_metrics = lines[lines_len - 1].metrics();
                match align {
                    LineAlign::Bottom => lst_metrics.descent - baseline,
                    LineAlign::Baseline => -baseline,
                    LineAlign::Middle => lst_metrics.x_height / 2.0 - baseline,
                    LineAlign::Hanging => lst_metrics.cap_height - baseline,
                    LineAlign::Top => lst_metrics.ascent - baseline,
                }
            }
        };

        for lidx in 0..lines_len {
            if lidx != 0 {
                y_cursor += lines[lidx].height();
            }

            self.layout_horizontal_line(&mut lines[lidx], y_cursor, type_align);

            y_cursor += lines[lidx].gap();
        }

        Ok(())
    }

    fn layout_horizontal_line(&self, line: &mut TextLine, y_baseline: f32, type_align: TypeAlign) {
        let ws = self.text[line.start..line.end]
            .chars()
            .filter(|c| c.is_whitespace())
            .count();
        let width = line.x_advance();
        let (width, justify) = match type_align {
            TypeAlign::Justify(sz) => {
                let sz = sz.max(width);
                let justify = if ws > 0 {
                    Justify::Ws {
                        added_gap: (sz - width) / ws as f32,
                    }
                } else {
                    Justify::Glyph { fact: sz / width }
                };
                (sz, justify)
            }
            _ => (width, Justify::Nope),
        };

        let x_start = match (type_align, line.main_dir) {
            (TypeAlign::Start, rustybuzz::Direction::LeftToRight)
            | (TypeAlign::End, rustybuzz::Direction::RightToLeft)
            | (TypeAlign::Left, _) => 0.0,
            (TypeAlign::Start, rustybuzz::Direction::RightToLeft)
            | (TypeAlign::End, rustybuzz::Direction::LeftToRight)
            | (TypeAlign::Right, _) => -width,
            (TypeAlign::Center, _) => -width / 2.0,
            _ => unreachable!(),
        };

        let top = y_baseline - line.ascent();
        let bottom = y_baseline - line.descent();

        let mut x_cursor = x_start;
        let mut y_cursor = y_baseline;

        let y_flip = Transform::from_scale(1.0, -1.0);
        for shape in line.shapes.iter_mut() {
            let shape_start = x_cursor;
            let scale_ts = Transform::from_scale(shape.metrics.scale, shape.metrics.scale);
            for glyph in shape.glyphs.iter_mut() {
                let x = x_cursor + glyph.x_offset;
                let y = y_cursor - glyph.y_offset;
                let pos_ts = Transform::from_translate(x, y);
                glyph.ts = y_flip.post_concat(scale_ts).post_concat(pos_ts);
                let glyph_start = x_cursor;
                x_cursor += match justify {
                    Justify::Nope => glyph.x_advance,
                    Justify::Glyph { fact } => glyph.x_advance * fact,
                    Justify::Ws { added_gap } => {
                        let is_ws = self.text[glyph.cluster..]
                            .chars()
                            .next()
                            .unwrap()
                            .is_whitespace();
                        if is_ws {
                            glyph.x_advance + added_gap
                        } else {
                            glyph.x_advance
                        }
                    }
                };
                y_cursor -= glyph.y_advance;
                for s in shape.spans.iter_mut() {
                    if s.start <= glyph.cluster && glyph.cluster < s.end {
                        s.bbox = BBox::unite(
                            &s.bbox,
                            &BBox {
                                top,
                                right: x_cursor,
                                bottom,
                                left: glyph_start,
                            },
                        );
                    }
                }
            }
            shape.y_baseline = y_baseline;
            shape.bbox = BBox {
                top,
                right: x_cursor,
                bottom,
                left: shape_start,
            };
        }
        line.bbox = BBox {
            top: y_baseline - line.ascent(),
            right: x_cursor,
            bottom: y_baseline - line.descent(),
            left: x_start,
        };
    }

    fn build_vertical_layout(&self, cols: &mut Vec<TextLine>) -> Result<(), Error> {
        let Layout::Vertical(type_align, _, progression, inter_col) = self.layout else {
            unreachable!()
        };

        let progression = match progression {
            VerProgression::PerScript => VerProgression::from_script(&self.text),
            progression => progression,
        };

        let move_x = |x_cursor: &mut f32, value: f32| match progression {
            VerProgression::LTR => *x_cursor += value,
            VerProgression::RTL => *x_cursor -= value,
            VerProgression::PerScript => unreachable!(),
        };

        let mut x_cursor = 0.0;

        for (idx, col) in cols.iter_mut().enumerate() {
            if idx != 0 {
                move_x(&mut x_cursor, col.col_width());
            }

            self.layout_vertical_column(col, x_cursor, type_align);

            move_x(&mut x_cursor, col.em_size() * inter_col.0);
        }

        Ok(())
    }

    fn layout_vertical_column(&self, col: &mut TextLine, x_leftline: f32, type_align: TypeAlign) {
        let ws = self.text[col.start..col.end]
            .chars()
            .filter(|c| c.is_whitespace())
            .count();
        let height = col.col_height();
        let (height, justify) = match type_align {
            TypeAlign::Justify(sz) => {
                let sz = sz.max(height);
                let justify = if ws > 0 {
                    Justify::Ws {
                        added_gap: (sz - height) / ws as f32,
                    }
                } else {
                    Justify::Glyph { fact: sz / height }
                };
                (sz, justify)
            }
            _ => (height, Justify::Nope),
        };

        let y_start = match (type_align, col.main_dir) {
            (TypeAlign::Start, rustybuzz::Direction::TopToBottom)
            | (TypeAlign::End, rustybuzz::Direction::BottomToTop)
            | (TypeAlign::Left, _) => 0.0,
            (TypeAlign::Start, rustybuzz::Direction::BottomToTop)
            | (TypeAlign::End, rustybuzz::Direction::TopToBottom)
            | (TypeAlign::Right, _) => -height,
            (TypeAlign::Center, _) => -height / 2.0,
            _ => unreachable!(),
        };

        let left = x_leftline;
        let right = x_leftline + col.col_width();

        let mut x_cursor = x_leftline;
        let mut y_cursor = y_start;

        let y_flip = Transform::from_scale(1.0, -1.0);
        for shape in col.shapes.iter_mut() {
            let shape_start = x_cursor;
            let scale_ts = Transform::from_scale(shape.metrics.scale, shape.metrics.scale);
            for glyph in shape.glyphs.iter_mut() {
                let x = x_cursor + glyph.x_offset;
                let y = y_cursor - glyph.y_offset;
                let pos_ts = Transform::from_translate(x, y);
                glyph.ts = y_flip.post_concat(scale_ts).post_concat(pos_ts);
                let glyph_start = y_cursor;
                y_cursor -= match justify {
                    Justify::Nope => glyph.y_advance,
                    Justify::Glyph { fact } => glyph.y_advance * fact,
                    Justify::Ws { added_gap } => {
                        let is_ws = self.text[glyph.cluster..]
                            .chars()
                            .next()
                            .unwrap()
                            .is_whitespace();
                        if is_ws {
                            glyph.y_advance + added_gap
                        } else {
                            glyph.y_advance
                        }
                    }
                };
                x_cursor += glyph.x_advance;
                for s in shape.spans.iter_mut() {
                    if s.start <= glyph.cluster && glyph.cluster < s.end {
                        s.bbox = BBox::unite(
                            &s.bbox,
                            &BBox {
                                top: glyph_start,
                                right,
                                bottom: x_cursor,
                                left,
                            },
                        );
                    }
                }
            }
            // y_baseline is only used for underline and strikeout
            // vertical underline is not supported, vertical strikeout doesn't use y_baseline
            shape.y_baseline = f32::NAN;
            shape.bbox = BBox {
                top: shape_start,
                right,
                bottom: x_cursor,
                left,
            };
            col.bbox = BBox::unite(&col.bbox, &shape.bbox);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn underline_span() {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        let mut builder =
            RichTextBuilder::new("Some RICH\ntext string".to_string(), TextProps::new(12.0));
        builder.add_span(
            5,
            9,
            TextOptProps {
                underline: Some(true),
                ..Default::default()
            },
        );
        let text = builder.shape_and_layout(&db).unwrap();
        assert_eq!(text.lines.len(), 2);
        assert_eq!(text.lines[0].shapes.len(), 1);
        assert_eq!(text.lines[1].shapes.len(), 1);
        assert_eq!(text.lines[0].shapes[0].spans.len(), 2);
        assert_eq!(text.lines[1].shapes[0].spans.len(), 1);
        assert_eq!(text.lines[0].shapes[0].spans[0].props.underline, false);
        assert_eq!(text.lines[0].shapes[0].spans[1].props.underline, true);
        assert_eq!(text.lines[1].shapes[0].spans[0].props.underline, false);
    }
}
