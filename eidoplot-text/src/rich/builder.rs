use super::{
    Direction, Error, Layout, RichTextLayout, TextOptProps, TextProps, TextSpan, VerDirection,
    TextLine, ShapeSpan, PropsSpan, Align, TypeAlign, LineAlign, Glyph,
};
use crate::font::{self, DatabaseExt};
use crate::fontdb;

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
enum Justify {
    Nope,
    Ws { added_gap: f32 },
    Glyph { fact: f32 },
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
            Layout::Vertical(_, VerDirection::TTB, _) => {
                BidiAlgo::Nope(rustybuzz::Direction::TopToBottom)
            }
            Layout::Vertical(_, VerDirection::BTT, _) => {
                BidiAlgo::Nope(rustybuzz::Direction::BottomToTop)
            }
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

        self.build_layout(lines)
    }

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

        let main_dir = ctx.bidi_algo.start_dir();
        let mut cur_dir = main_dir;
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

        Ok(TextLine {
            start,
            end,
            shapes,
            main_dir,
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
            .with_face_data(face_id, |data, index| -> Result<_, Error> {
                let face = ttf::Face::parse(data, index)?;
                let metrics = font::face_metrics(&face).scaled(shape_props.font_size);
                let mut hbface = rustybuzz::Face::from_face(face);
                font::apply_hb_variations(&mut hbface, &shape_props.font);

                let kern = rustybuzz::Feature::new(ttf::Tag::from_bytes(b"kern"), 1, ..);
                Ok((rustybuzz::shape(&hbface, &[kern], buffer), metrics))
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
        Ok(RichTextLayout {
            text: self.text,
            lines,
        })
    }

    fn build_horizontal_layout(&self, lines: &mut Vec<TextLine>) -> Result<(), Error> {
        let Layout::Horizontal(align, type_align, _) = self.layout else {
            unreachable!()
        };

        let lines_len = lines.len();

        let fst_metrics = lines[0].metrics();
        let lst_metrics = lines[lines_len - 1].metrics();

        // y-cursor must be placed at the baseline of the first line
        let mut y_cursor = match align {
            Align::Top => fst_metrics.ascent,
            Align::Bottom => lst_metrics.descent - lines.baseline(lines_len - 1),
            Align::Center => {
                let top = fst_metrics.ascent;
                let bottom = lst_metrics.descent - lines.baseline(lines_len - 1);
                (top + bottom) / 2.0
            }
            Align::Line(line, align) => {
                let baseline = lines.baseline(line);
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

            let metrics = if lidx == 0 {
                fst_metrics
            } else if lidx == lines_len - 1 {
                lst_metrics
            } else {
                lines[lidx].metrics()
            };

            self.layout_horizontal_line(&mut lines[lidx], y_cursor, &metrics, type_align);

            y_cursor += metrics.line_gap;
        }

        Ok(())
    }

    fn layout_horizontal_line(
        &self,
        line: &mut TextLine,
        y_baseline: f32,
        metrics: &font::ScaledMetrics,
        type_align: TypeAlign,
    ) {
        let ws = self.text[line.start..line.end]
            .chars()
            .filter(|c| c.is_whitespace())
            .count();
        let width = line.x_advance();
        let (width, justify) = match type_align {
            TypeAlign::Justify(wid) => {
                let wid = wid.max(width);
                let justify = if ws > 0 {
                    Justify::Ws {
                        added_gap: (wid - width) / ws as f32,
                    }
                } else {
                    Justify::Glyph { fact: wid / width }
                };
                (wid, justify)
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

        let mut x_cursor = x_start;
        let mut y_cursor = y_baseline;

        let y_flip = Transform::from_scale(1.0, -1.0);
        for shape in line.shapes.iter_mut() {
            let scale_ts = Transform::from_scale(metrics.scale, metrics.scale);
            for glyph in shape.glyphs.iter_mut() {
                let x = x_cursor + glyph.x_offset;
                let y = y_cursor + glyph.y_offset;
                let pos_ts = Transform::from_translate(x, y);
                glyph.ts = y_flip.post_concat(scale_ts).post_concat(pos_ts);
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
                y_cursor += glyph.y_advance;
            }
        }
    }

    fn build_vertical_layout(&self, lines: &mut Vec<TextLine>) -> Result<(), Error> {
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