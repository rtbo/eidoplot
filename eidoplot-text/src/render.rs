use tiny_skia_path::{PathBuilder, Transform};
use ttf_parser as ttf;

use crate::{font, shape, style};

#[derive(Debug, Clone, Copy, Default)]
pub enum HorAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum VerAlign {
    #[default]
    Baseline,
    Middle,
    Hanging,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Align(pub HorAlign, pub VerAlign);

impl Align {
    pub fn hor(&self) -> HorAlign {
        self.0
    }

    pub fn ver(&self) -> VerAlign {
        self.1
    }
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub id: Option<ttf::GlyphId>,
    pub ts: Transform,
}

pub fn render_line(
    line: &shape::Line,
    transform: tiny_skia_path::Transform,
    align: Align,
    font_size: f32,
    justify: Option<f32>,
    db: &font::Database,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    let width = line.width(font_size);

    let mut x_cursor = match align.hor() {
        HorAlign::Start => 0.0,
        HorAlign::Center => -width / 2.0,
        HorAlign::End => -width,
    };

    let mut y_cursor = match align.ver() {
        VerAlign::Baseline => 0.0,
        VerAlign::Middle => -line.x_height(font_size) / 2.0,
        VerAlign::Hanging => -line.cap_height(font_size),
    };

    println!("origin is ({}, {})", x_cursor, y_cursor);

    let justify_gap = justify.map(|j| justify_gap(line, width, j));

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

        println!("translating glyph {:?} at ({}, {})", sh_gl.id(), x, y);

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
            line.style(),
            db,
            pixmap,
        );
    }
}

fn justify_gap(line: &shape::Line, width: f32, justify: f32) -> f32 {
    if justify <= width {
        return 0.0;
    }
    let adv_count = line.glyphs().iter().filter(|g| g.has_x_advance()).count();
    if adv_count > 1 {
        (justify - width) / (adv_count as f32 - 1.0)
    } else {
        0.0
    }
}

fn render_glyphs(
    glyphs: &[Glyph],
    transform: tiny_skia_path::Transform,
    font_id: font::ID,
    style: &style::Font,
    db: &font::Database,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    db.with_face_data(font_id, |data, index| {
        let mut face = ttf::Face::parse(data, index).unwrap();
        font::apply_variations(&mut face, style);

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
