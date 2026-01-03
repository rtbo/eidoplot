//! Module that contains a simple single line text layout and rendering engine

use plotive_base::geom;
use ttf_parser as ttf;

use crate::bidi::{self, BidiAlgo};
use crate::font::{self, DatabaseExt};
use crate::{Error, Font, ScriptDir, fontdb};

/// Horizontal alignment
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Align {
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

/// Vertical alignment for a single line of text
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VerAlign {
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

/// A single line of text
#[derive(Debug, Clone)]
pub struct LineText {
    text: String,
    align: (Align, VerAlign),
    font_size: f32,
    font: Font,
    bbox: Option<geom::Rect>,
    main_dir: ScriptDir,
    metrics: font::ScaledMetrics,
    pub(crate) shapes: Vec<Shape>,
}

impl LineText {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn align(&self) -> (Align, VerAlign) {
        self.align
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn font(&self) -> &Font {
        &self.font
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

    pub fn main_dir(&self) -> ScriptDir {
        self.main_dir
    }

    pub fn metrics(&self) -> font::ScaledMetrics {
        self.metrics
    }

    fn new_empty(font: Font) -> Self {
        Self {
            text: String::new(),
            align: (Default::default(), Default::default()),
            font_size: 1.0,
            font,
            bbox: None,
            main_dir: ScriptDir::LeftToRight,
            metrics: font::ScaledMetrics::null(),
            shapes: Vec::new(),
        }
    }

    /// Create a new text line
    ///
    /// This function will run the unicode bidirectional algorithm and shape the text.
    /// The glyphs are laid out so that the origin point (0, 0) correspond to the provided
    /// alignment options.
    /// The final position on the text is defined by the transform provided to the render function.
    pub fn new(
        text: String,
        align: (Align, VerAlign),
        font_size: f32,
        font: Font,
        db: &fontdb::Database,
    ) -> Result<Self, Error> {
        let default_lev = match crate::script_is_rtl(&text) {
            Some(false) => Some(unicode_bidi::LTR_LEVEL),
            Some(true) => Some(unicode_bidi::RTL_LEVEL),
            None => None,
        };
        let mut bidi = BidiAlgo::Yep { default_lev };
        let bidi_runs = bidi.visual_runs(&text, 0);
        if bidi_runs.is_empty() {
            return Ok(LineText::new_empty(font.clone()));
        }
        let main_dir = match default_lev {
            Some(lev) if lev.is_ltr() => ScriptDir::LeftToRight,
            Some(lev) if lev.is_rtl() => ScriptDir::RightToLeft,
            _ => match bidi_runs[0].dir {
                rustybuzz::Direction::LeftToRight => ScriptDir::LeftToRight,
                rustybuzz::Direction::RightToLeft => ScriptDir::RightToLeft,
                _ => unreachable!(),
            },
        };

        let mut shapes = Vec::with_capacity(bidi_runs.len());
        let mut ctx = Ctx { buffer: None };
        for run in &bidi_runs {
            let shape = Shape::shape_run(&text, run, font_size, &font, db, &mut ctx)?;
            shapes.push(shape);
        }

        let (align, ver_align) = align;

        let metrics = shapes.metrics();

        let mut y_cursor = match ver_align {
            VerAlign::Bottom => metrics.descent,
            VerAlign::Baseline => 0.0,
            VerAlign::Middle => metrics.x_height / 2.0,
            VerAlign::Hanging => metrics.cap_height,
            VerAlign::Top => metrics.ascent,
        };

        let width = shapes.width();

        let x_start = match (align, main_dir) {
            (Align::Start, ScriptDir::LeftToRight)
            | (Align::End, ScriptDir::RightToLeft)
            | (Align::Left, _) => 0.0,
            (Align::Start, ScriptDir::RightToLeft)
            | (Align::End, ScriptDir::LeftToRight)
            | (Align::Right, _) => -width,
            (Align::Center, _) => -width / 2.0,
        };

        let top = y_cursor - metrics.ascent;
        let bottom = y_cursor - metrics.descent;

        let mut x_cursor = x_start;

        let y_flip = geom::Transform::from_scale(1.0, -1.0);

        for shape in shapes.iter_mut() {
            let scale_ts = geom::Transform::from_scale(shape.metrics.scale, shape.metrics.scale);
            for glyph in shape.glyphs.iter_mut() {
                let x = x_cursor + glyph.x_offset;
                let y = y_cursor - glyph.y_offset;
                let pos_ts = geom::Transform::from_translate(x, y);
                glyph.ts = y_flip.post_concat(scale_ts).post_concat(pos_ts);
                x_cursor += glyph.x_advance;
                y_cursor -= glyph.y_advance;
            }
        }

        Ok(LineText {
            text,
            align: (align, ver_align),
            font_size,
            font: font.clone(),
            bbox: Some(geom::Rect::from_trbl(top, x_cursor, bottom, x_start)),
            main_dir,
            metrics,
            shapes,
        })
    }
}

/// A shaped text run
#[derive(Debug, Clone)]
pub(crate) struct Shape {
    pub(crate) face_id: fontdb::ID,
    pub(crate) metrics: font::ScaledMetrics,
    pub(crate) glyphs: Vec<Glyph>,
}

impl Shape {
    fn width(&self) -> f32 {
        self.glyphs.iter().map(|g| g.x_advance).sum()
    }
}

trait ShapesExt {
    fn metrics(&self) -> font::ScaledMetrics;
    fn width(&self) -> f32;
}

impl ShapesExt for [Shape] {
    fn metrics(&self) -> font::ScaledMetrics {
        let mut metrics = self[0].metrics;
        for s in self.iter().skip(1) {
            metrics.ascent = metrics.ascent.min(s.metrics.ascent);
            metrics.descent = metrics.descent.max(s.metrics.descent);
            metrics.x_height += metrics.x_height;
            metrics.cap_height = metrics.cap_height.max(s.metrics.cap_height);
        }
        metrics.x_height /= self.len() as f32;
        metrics
    }

    fn width(&self) -> f32 {
        let mut w = 0.0;
        for s in self {
            w += s.width();
        }
        w
    }
}

/// A glyph in a shaped text run
#[derive(Debug, Clone, Copy)]
pub(crate) struct Glyph {
    pub(crate) id: ttf::GlyphId,
    x_offset: f32,
    y_offset: f32,
    x_advance: f32,
    y_advance: f32,
    pub(crate) ts: geom::Transform,
}

#[derive(Debug)]
struct Ctx {
    buffer: Option<rustybuzz::UnicodeBuffer>,
}

impl Shape {
    fn shape_run(
        text: &str,
        run: &bidi::BidiRun,
        font_size: f32,
        font: &font::Font,
        db: &fontdb::Database,
        ctx: &mut Ctx,
    ) -> Result<Self, Error> {
        let face_id = db
            .select_face_for_str(font, text)
            .or_else(|| db.select_face(&font))
            .ok_or_else(|| Error::NoSuchFont(font.clone()))?;

        let mut buffer = ctx
            .buffer
            .take()
            .unwrap_or_else(|| rustybuzz::UnicodeBuffer::new());
        buffer.push_str(&text[run.start..run.end]);
        if run.start != 0 {
            buffer.set_pre_context(&text[..run.start]);
        }
        if run.end != text.len() {
            buffer.set_post_context(&text[run.end..]);
        }

        buffer.set_direction(run.dir);
        buffer.guess_segment_properties();

        let (shape, metrics) = db
            .with_face_data(face_id, |data, index| -> Result<_, Error> {
                let face = ttf::Face::parse(data, index)?;
                let metrics = font::face_metrics(&face).scaled(font_size);
                let mut hbface = rustybuzz::Face::from_face(face);
                font::apply_hb_variations(&mut hbface, &font);

                Ok((rustybuzz::shape(&hbface, &[], buffer), metrics))
            })
            .expect("should be a valid face id")?;

        let mut glyphs = Vec::with_capacity(shape.len());
        for (i, p) in shape.glyph_infos().iter().zip(shape.glyph_positions()) {
            glyphs.push(Glyph {
                id: ttf::GlyphId(i.glyph_id as u16),
                x_advance: p.x_advance as f32 * metrics.scale,
                y_advance: p.y_advance as f32 * metrics.scale,
                x_offset: p.x_offset as f32 * metrics.scale,
                y_offset: p.y_offset as f32 * metrics.scale,
                ts: tiny_skia::Transform::identity(),
            })
        }

        ctx.buffer = Some(shape.clear());

        Ok(Shape {
            face_id,
            glyphs,
            metrics,
        })
    }
}

pub fn render_line_text_with<R>(line: &LineText, db: &font::Database, mut render_fn: R)
where
    R: FnMut(&geom::Path),
{
    for shape in line.shapes.iter() {
        db.with_face_data(shape.face_id, |data, index| {
            let mut face = ttf::Face::parse(data, index).unwrap();
            font::apply_ttf_variations(&mut face, line.font());

            // the path builder for the entire string
            let mut str_pb = geom::PathBuilder::new();
            // the path builder for each glyph
            let mut gl_pb = geom::PathBuilder::new();

            for gl in &shape.glyphs {
                {
                    let mut builder = crate::Outliner(&mut gl_pb);
                    face.outline_glyph(gl.id, &mut builder);
                }

                if let Some(path) = gl_pb.finish() {
                    let path = path.transform(gl.ts).unwrap();
                    str_pb.push_path(&path);

                    gl_pb = path.clear();
                } else {
                    gl_pb = geom::PathBuilder::new();
                }
            }

            if let Some(path) = str_pb.finish() {
                render_fn(&path);
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct RenderOptions<'a> {
    pub fill: Option<tiny_skia::Paint<'a>>,
    pub outline: Option<(tiny_skia::Paint<'a>, tiny_skia::Stroke)>,
    pub mask: Option<&'a tiny_skia::Mask>,
    pub transform: geom::Transform,
}

pub fn render_line_text(
    line: &LineText,
    opts: &RenderOptions<'_>,
    db: &font::Database,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    let render_fn = |path: &geom::Path| {
        if let Some(paint) = opts.fill.as_ref() {
            pixmap.fill_path(
                &path,
                &paint,
                tiny_skia::FillRule::Winding,
                opts.transform,
                opts.mask,
            );
        }
        if let Some((paint, stroke)) = opts.outline.as_ref() {
            pixmap.stroke_path(&path, &paint, &stroke, opts.transform, opts.mask);
        }
    };
    render_line_text_with(line, db, render_fn);
}
