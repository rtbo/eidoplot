use tiny_skia_path::PathBuilder;
use ttf_parser as ttf;

use crate::font::{self, Font};
use crate::layout::{Glyph, TextLayout};

pub fn render_text<F>(text: &TextLayout, db: &font::Database, mut render_func: F)
where
    F: FnMut(&[Glyph], &Font, font::ID, &font::Database),
{
    let font = text.font();

    for lines in text.lines.iter() {
        for run in lines.runs.iter() {
            render_func(&run.glyphs, font, run.font_id, db);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Options<'a> {
    pub fill: Option<tiny_skia::Paint<'a>>,
    pub outline: Option<(tiny_skia::Paint<'a>, tiny_skia::Stroke)>,
    pub mask: Option<&'a tiny_skia::Mask>,
    pub transform: tiny_skia_path::Transform,
}

pub fn render_text_tiny_skia(
    text: &TextLayout,
    opts: &Options<'_>,
    db: &font::Database,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    render_text(text, db, |glyphs, font, font_id, db| {
        render_glyphs_tiny_skia(glyphs, font, font_id, db, opts, pixmap)
    });
}

fn render_glyphs_tiny_skia(
    glyphs: &[Glyph],
    font: &Font,
    font_id: font::ID,
    db: &font::Database,
    render_opts: &Options<'_>,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    db.with_face_data(font_id, |data, index| {
        let mut face = ttf::Face::parse(data, index).unwrap();
        font::apply_variations(&mut face, font);

        // the path builder for the entire string
        let mut str_pb = PathBuilder::new();
        // the path builder for each glyph
        let mut gl_pb = PathBuilder::new();

        for gl in glyphs {
            {
                let mut builder = Outliner(&mut gl_pb);
                face.outline_glyph(gl.id, &mut builder);
            }

            if let Some(path) = gl_pb.finish() {
                let path = path.transform(gl.ts).unwrap();
                str_pb.push_path(&path);

                gl_pb = path.clear();
            } else {
                gl_pb = PathBuilder::new();
            }
        }

        if let Some(path) = str_pb.finish() {
            if let Some(paint) = render_opts.fill.as_ref() {
                pixmap.fill_path(
                    &path,
                    &paint,
                    tiny_skia::FillRule::Winding,
                    render_opts.transform,
                    render_opts.mask,
                );
            }
            if let Some((paint, stroke)) = render_opts.outline.as_ref() {
                pixmap.stroke_path(
                    &path,
                    &paint,
                    &stroke,
                    render_opts.transform,
                    render_opts.mask,
                );
            }
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
