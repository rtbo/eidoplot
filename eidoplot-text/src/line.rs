//! Module that contains a simple single line text layout and rendering engine

use std::fmt;

use crate::{
    BBox,
    bidi::{self, BidiAlgo},
    fontdb,
    font::{self, DatabaseExt},
};
use tiny_skia_path::Transform;
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
pub enum Baseline {
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

/// Options for the layout of text line
pub struct Options {
    /// The horizontal alignment
    align: Align,
    /// The vertical alignment
    baseline: Baseline,
    /// The font to use
    font: font::Font,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            align: Align::default(),
            baseline: Baseline::default(),
            font: font::Font::default(),
        }
    }
}

/// A single line of text
#[derive(Debug, Clone)]
pub struct Line {
    text: String,
    bbox: BBox,
    shapes: Vec<Shape>,
}

impl Line {
    pub fn text(&self) -> &str {
        &self.text
    }

    fn new_empty() -> Self {
        Self {
            text: String::new(),
            bbox: BBox::EMPTY,
            shapes: Vec::new(),
        }
    }
}

/// A shaped text run
#[derive(Debug, Clone)]
struct Shape {
    start: usize,
    end: usize,
    face_id: fontdb::ID,
    metrics: font::ScaledMetrics,
    glyphs: Vec<Glyph>,
}

impl Shape {
    fn width(&self) -> f32 {
        self.glyphs.iter().map(|g| g.x_advance).sum()
    }
}

trait ShapesExt {
    fn descent(&self) -> f32;
    fn ascent(&self) -> f32;
    fn x_height(&self) -> f32;
    fn cap_height(&self) -> f32;
    fn width(&self) -> f32;
}

impl ShapesExt for [Shape] {
    fn descent(&self) -> f32 {
        self.iter()
            .map(|s| s.metrics.descent)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn ascent(&self) -> f32 {
        self.iter()
            .map(|s| s.metrics.ascent)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn x_height(&self) -> f32 {
        let sum: f32 = self.iter().map(|s| s.metrics.x_height).sum();
        sum / self.len() as f32
    }

    fn cap_height(&self) -> f32 {
        let sum: f32 = self.iter().map(|s| s.metrics.cap_height).sum();
        sum / self.len() as f32
    }

    fn width(&self) -> f32 {
        let w = 0.0;
        for s in self {
            w + s.width();
        }
        w
    }
}

/// A glyph in a shaped text run
#[derive(Debug, Clone, Copy)]
struct Glyph {
    id: ttf::GlyphId,
    cluster: usize,
    x_offset: f32,
    y_offset: f32,
    x_advance: f32,
    y_advance: f32,
    ts: Transform,
}

#[derive(Debug)]
struct Ctx {
    buffer: Option<rustybuzz::UnicodeBuffer>,
}

impl Line {
    /// Create a new text line
    ///
    /// This function will run the unicode bidirectional algorithm and shape the text.
    /// The glyphs are laid out so that the origin point (0, 0) correspond to the provided
    /// alignment options.
    /// The final position on the text is defined by the transform provided to the render function.
    pub fn new(
        text: String,
        font_size: f32,
        opts: &Options,
        db: &fontdb::Database,
    ) -> Result<Self, Error> {
        let mut bidi = BidiAlgo::Yep { default_lev: None };
        let bidi_runs = bidi.visual_runs(&text, 0);
        if bidi_runs.is_empty() {
            return Ok(Line::new_empty());
        }
        let main_dir = bidi_runs[0].dir;

        let mut shapes = Vec::with_capacity(bidi_runs.len());
        let mut ctx = Ctx { buffer: None };
        for run in &bidi_runs {
            let shape = Shape::shape_run(&text, run, font_size, &opts.font, db, &mut ctx)?;
            shapes.push(shape);
        }

        let mut y_cursor = match opts.baseline {
            Baseline::Bottom => shapes.descent(),
            Baseline::Baseline => 0.0,
            Baseline::Middle => shapes.x_height() / 2.0,
            Baseline::Hanging => shapes.cap_height(),
            Baseline::Top => shapes.ascent(),
        };

        let width = shapes.width();

        let x_start = match (opts.align, main_dir) {
            (Align::Start, rustybuzz::Direction::LeftToRight)
            | (Align::End, rustybuzz::Direction::RightToLeft)
            | (Align::Left, _) => 0.0,
            (Align::Start, rustybuzz::Direction::RightToLeft)
            | (Align::End, rustybuzz::Direction::LeftToRight)
            | (Align::Right, _) => -width,
            (Align::Center, _) => -width / 2.0,
            _ => unreachable!(),
        };

        let top = y_cursor - shapes.ascent();
        let bottom = y_cursor - shapes.descent();

        let mut x_cursor = x_start;

        let y_flip = Transform::from_scale(1.0, -1.0);

        for shape in shapes.iter_mut() {
            let scale_ts = Transform::from_scale(shape.metrics.scale, shape.metrics.scale);
            for glyph in shape.glyphs.iter_mut() {
                let x = x_cursor + glyph.x_offset;
                let y = y_cursor - glyph.y_offset;
                let pos_ts = Transform::from_translate(x, y);
                glyph.ts = y_flip.post_concat(scale_ts).post_concat(pos_ts);
                x_cursor += glyph.x_advance;
                y_cursor -= glyph.y_advance;
            }
        }

        Ok(Line {
            text,
            bbox: BBox {
                top,
                right: x_cursor,
                bottom,
                left: x_start,
            },
            shapes,
        })
    }
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
        buffer.push_str(text);
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
                cluster: i.cluster as usize + run.start,
                x_advance: p.x_advance as f32 * metrics.scale,
                y_advance: p.y_advance as f32 * metrics.scale,
                x_offset: p.x_offset as f32 * metrics.scale,
                y_offset: p.y_offset as f32 * metrics.scale,
                ts: tiny_skia::Transform::identity(),
            })
        }

        ctx.buffer = Some(shape.clear());

        Ok(Shape {
            start: run.start,
            end: run.end,
            face_id,
            glyphs,
            metrics,
        })
    }
}
