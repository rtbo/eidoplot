use tiny_skia_path::{PathBuilder, Transform};
use ttf_parser as ttf;

use crate::shape;
use crate::font::{self, Font};

#[derive(Debug, Clone, Copy, Default)]
pub enum HorAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum LineVerAlign {
    /// Align the bottom of the descender
    Bottom,
    /// Align the baseline
    #[default]
    Baseline,
    /// Align the middle of the x-height
    Middle,
    /// Align at capital height
    Hanging,
    /// Align at the top of the ascender
    Top,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LineAlign(pub HorAlign, pub LineVerAlign);

impl LineAlign {
    pub fn hor(&self) -> HorAlign {
        self.0
    }

    pub fn ver(&self) -> LineVerAlign {
        self.1
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextVerAlign {
    Line(usize, LineVerAlign),
    Top,
    Center,
    Bottom,
}

impl Default for TextVerAlign {
    fn default() -> Self {
        TextVerAlign::Line(0, LineVerAlign::default())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextAlign {
    pub hor: HorAlign,
    pub ver: TextVerAlign,
    pub justify: bool,
}

#[derive(Debug, Clone)]
struct Glyph {
    id: Option<ttf::GlyphId>,
    ts: Transform,
}

pub fn render_text(
    text: &shape::Text,
    transform: tiny_skia_path::Transform,
    align: TextAlign,
    font_size: f32,
    db: &font::Database,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    let lines = text.lines();
    if lines.is_empty() {
        return;
    }

    let fst = &lines[0];
    let lst = &lines[lines.len() - 1];

    let mut y_cursor = match align.ver {
        TextVerAlign::Top => -fst.ascent(font_size),
        TextVerAlign::Bottom => {
            text.baseline_of_line(lines.len() - 1, font_size) - lst.descent(font_size)
        }
        TextVerAlign::Center => {
            let top = -fst.ascent(font_size);
            let bottom = text.baseline_of_line(lines.len() - 1, font_size) - lst.descent(font_size);
            (top + bottom) / 2.0
        }
        TextVerAlign::Line(line, align) => {
            let baseline = text.baseline_of_line(line, font_size);
            match align {
                LineVerAlign::Bottom => baseline - lst.descent(font_size),
                LineVerAlign::Baseline => baseline,
                LineVerAlign::Middle => baseline - lst.x_height(font_size) / 2.0,
                LineVerAlign::Hanging => baseline - lst.cap_height(font_size),
                LineVerAlign::Top => baseline - lst.ascent(font_size),
            }
        }
    };

    let line_align = LineAlign(align.hor, LineVerAlign::Baseline);

    let justify = if align.justify {
        Some(text.width(font_size))
    } else {
        None
    };

    for (i, line) in lines.iter().enumerate() {
        if i != 0 {
            y_cursor -= line.height(font_size);
        }
        render_line_at_y(
            y_cursor, line, transform, line_align, font_size, justify, db, pixmap,
        );
        y_cursor -= line.gap(font_size);
    }
}

pub fn render_line(
    line: &shape::Line,
    transform: tiny_skia_path::Transform,
    align: LineAlign,
    font_size: f32,
    justify: Option<f32>,
    db: &font::Database,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    render_line_at_y(0.0, line, transform, align, font_size, justify, db, pixmap);
}

fn render_line_at_y(
    y_start: f32,
    line: &shape::Line,
    transform: tiny_skia_path::Transform,
    align: LineAlign,
    font_size: f32,
    justify: Option<f32>,
    db: &font::Database,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    let width = line.width(font_size);

    let (width, justify_gap) = match justify {
        Some(justify) => {
            if justify <= width {
                (width, None)
            } else {
                let adv_count = line.glyphs().iter().filter(|g| g.has_x_advance()).count();
                let gap = if adv_count > 1 {
                    (justify - width) / (adv_count as f32 - 1.0)
                } else {
                    0.0
                };
                (justify, Some(gap)) 
            }
        },
        None => (width, None),
    };

    let mut x_cursor = match (align.hor(), line.rtl()) {
        (HorAlign::Start, false) | (HorAlign::End, true) => 0.0,
        (HorAlign::Center, _) => -width / 2.0,
        (HorAlign::Start, true) | (HorAlign::End, false) => -width,
    };

    let mut y_cursor = y_start
        + match align.ver() {
            LineVerAlign::Bottom => -line.descent(font_size),
            LineVerAlign::Baseline => 0.0,
            LineVerAlign::Middle => -line.x_height(font_size) / 2.0,
            LineVerAlign::Hanging => -line.cap_height(font_size),
            LineVerAlign::Top => -line.ascent(font_size),
        };

    // grouping by font-id in order to avoid loading the same font on every glyph
    let mut runs = Vec::new();
    let mut run_start = 0;
    let mut glyphs_buf = Vec::with_capacity(line.glyphs().len());
    let mut font_id = None;

    for (gi, sh_gl) in line.glyphs().iter().enumerate() {
        let scale = sh_gl.scale(font_size);
        let scale_ts = Transform::from_scale(scale, scale);

        let x = x_cursor + sh_gl.x_offset(font_size);
        let y = y_cursor + sh_gl.y_offset(font_size);
        let pos_ts = Transform::from_translate(x, y);

        let gl = Glyph {
            id: sh_gl.id(),
            ts: scale_ts.post_concat(pos_ts),
        };
        if let Some(font_id) = font_id {
            if sh_gl.font_id() != font_id {
                runs.push((run_start, gi, font_id));
                run_start = gi;
            }
        }

        glyphs_buf.push(gl);
        font_id = Some(sh_gl.font_id());

        x_cursor += sh_gl.x_advance(font_size);
        y_cursor += sh_gl.y_advance(font_size);
        if let Some(jg) = justify_gap {
            if sh_gl.has_x_advance() {
                x_cursor += jg;
            }
        }
    }
    if run_start < line.glyphs().len() {
        runs.push((run_start, line.glyphs().len(), font_id.unwrap()));
    }

    for r in runs {
        render_glyphs(
            &glyphs_buf[r.0..r.1],
            transform,
            r.2,
            line.font(),
            db,
            pixmap,
        );
    }
}

fn render_glyphs(
    glyphs: &[Glyph],
    transform: tiny_skia_path::Transform,
    font_id: font::ID,
    font: &Font,
    db: &font::Database,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    println!("rendering with {:?}", db.face(font_id).unwrap());

    db.with_face_data(font_id, |data, index| {
        let mut face = ttf::Face::parse(data, index).unwrap();
        font::apply_variations(&mut face, font);

        // the path builder for the entire string
        let mut str_pb = PathBuilder::new();
        // the path builder for each glyph
        let mut gl_pb = PathBuilder::new();

        for gl in glyphs {
            let Some(glyph_id) = gl.id else {
                continue;
            };

            {
                let mut builder = Outliner(&mut gl_pb);
                face.outline_glyph(glyph_id, &mut builder);
            }

            if let Some(path) = gl_pb.finish() {
                let path = path.transform(gl.ts).unwrap();
                str_pb.push_path(&path);

                gl_pb = path.clear();
            } else {
                gl_pb = PathBuilder::new();
            }
        }
        let paint = tiny_skia::Paint::default();
        if let Some(path) = str_pb.finish() {
            let path = path.transform(Transform::from_scale(1.0, -1.0)).unwrap();
            pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, transform, None);
        }
    })
    .unwrap();
}

struct Outliner<'a>(&'a mut PathBuilder);

impl ttf::OutlineBuilder for Outliner<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.0.close();
    }
}
