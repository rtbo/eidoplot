use std::sync::Arc;

use rustybuzz::ttf_parser;

pub struct Ctx<'a, S> {
    pub surface: &'a mut S,
    pub fontdb: Arc<fontdb::Database>,
}

impl<'a, S> Ctx<'a, S> {
    pub fn calculate_string_width(&self, face_id: fontdb::ID, text: &str, font_sz: f32) -> f32 {
        self.fontdb.with_face_data(face_id, |data, index| {
            let face = ttf_parser::Face::parse(data, index).unwrap();
            let units_per_em = face.units_per_em() as f32;
            let hbf  = rustybuzz::Face::from_face(face);
            let mut buffer = rustybuzz::UnicodeBuffer::new();
            buffer.push_str(text);
            let glyph_buffer = rustybuzz::shape(&hbf, &[], buffer);
            let scale = font_sz / units_per_em;
            glyph_buffer.glyph_positions().iter().map(|gp| gp.x_advance as f32 * scale).sum()
        }).unwrap()
    }
}