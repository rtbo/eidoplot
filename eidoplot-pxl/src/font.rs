use ttf_parser as ttf;
use eidoplot::{geom, render, style, font};

use font::DatabaseExt as _;

pub use fontdb::Database;

pub trait DatabaseExt {
    fn outline_text(
        &self,
        text: &str,
        font: &style::Font,
        path_builder_reuse: Option<geom::PathBuilder>,
    ) -> OutlinedText;
}

impl DatabaseExt for Database {
    fn outline_text(
        &self,
        text: &str,
        font: &style::Font,
        path_builder_reuse: Option<geom::PathBuilder>,
    ) -> OutlinedText {
        let id = self.query_face(font).expect("Should find a face");

        self.with_face_data(id, |data, index| {
                let mut face = ttf::Face::parse(data, index).unwrap();
                if face.is_variable() {
                    let _ = face.set_variation(
                        ttf::Tag::from_bytes(b"wght"),
                        font.weight().0 as f32,
                    );
                    let _ = face.set_variation(
                        ttf::Tag::from_bytes(b"wdth"),
                        font::width_to_percent(font.width()),
                    );
                }
                let units_per_em = face.units_per_em() as f32;
                let scale = font.size() / units_per_em;
                let ts_scale = geom::Transform::from_scale(scale, scale);

                let face = rustybuzz::Face::from_face(face);
                let mut buffer = rustybuzz::UnicodeBuffer::new();
                buffer.push_str(text);
                let glyph_buffer = rustybuzz::shape(&face, &[], buffer);
                let gps = glyph_buffer.glyph_positions();
                let gis = glyph_buffer.glyph_infos();
                assert_eq!(gps.len(), gis.len());

                // the path builder for the entire string
                let mut str_pb = path_builder_reuse.unwrap_or(geom::PathBuilder::new());
                // the path builder for each glyph
                let mut gl_pb = geom::PathBuilder::new();

                let mut x_cursor = 0.0;
                let mut y_cursor = 0.0;

                for (gp, gi) in gps.iter().zip(gis.iter()) {

                    let glyph_id = ttf::GlyphId(gi.glyph_id as u16);

                    let tx = gp.x_offset as f32 + x_cursor;
                    let ty = gp.y_offset as f32 + y_cursor;

                    {
                        let mut builder = Outliner(&mut gl_pb);
                        face.outline_glyph(glyph_id, &mut builder);
                    }

                    if let Some(path) = gl_pb.finish() {
                        let ts_glyph = geom::Transform::from_translate(tx, ty);
                        let transform = ts_scale.pre_concat(ts_glyph);

                        // print_transform_info("text", &ts_text);
                        // print_transform_info("scale", &ts_scale);
                        // print_transform_info("glyph", &ts_glyph);
                        // print_transform_info("result", &transform);
                        // println!("");

                        let path = path.transform(transform).unwrap();
                        str_pb.push_path(&path);

                        gl_pb = path.clear();
                    } else {
                        gl_pb = geom::PathBuilder::new();
                    }

                    x_cursor += gp.x_advance as f32;
                    y_cursor += gp.y_advance as f32;
                }

                // println!("Finished {:?}", text);

                let path = str_pb.finish().unwrap();
                let xh = face
                    .x_height()
                    .map(|v| v as f32)
                    .unwrap_or_else(|| face.ascender() as f32 * 0.6);
                let cap = face
                    .capital_height()
                    .map(|v| v as f32)
                    .unwrap_or_else(|| face.ascender() as f32 * 0.8);

                OutlinedText {
                    path,
                    w: x_cursor * scale,
                    xh: xh * scale,
                    cap: cap * scale,
                }
            })
            .expect("Should find face data")
    }
}

pub struct OutlinedText {
    path: geom::Path,
    w: f32,
    xh: f32,
    cap: f32,
}

impl OutlinedText {
    pub fn anchor_transform(&self, anchor: render::TextAnchor) -> geom::Transform {
        let ts_flip = geom::Transform::from_scale(1.0, -1.0);

        let ts_point = geom::Transform::from_translate(anchor.pos.x(), anchor.pos.y());

        let ts_align = match anchor.align {
            render::TextAlign::Start => geom::Transform::identity(),
            render::TextAlign::Center => geom::Transform::from_translate(-self.w / 2.0, 0.0),
            render::TextAlign::End => geom::Transform::from_translate(-self.w, 0.0),
        };

        let ts_baseline = match anchor.baseline {
            render::TextBaseline::Base => geom::Transform::identity(),
            render::TextBaseline::Center => geom::Transform::from_translate(0.0, self.xh / 2.0),
            render::TextBaseline::Hanging => geom::Transform::from_translate(0.0, self.cap),
        };

        //ts_point.pre_concat(ts_align).pre_concat(ts_baseline)
        ts_point
            .pre_concat(ts_align)
            .pre_concat(ts_baseline)
            .pre_concat(ts_flip)
    }

    /// Recycles the path builder allocation
    pub fn into_path(self) -> geom::Path {
        self.path
    }
}

struct Outliner<'a>(&'a mut geom::PathBuilder);

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
