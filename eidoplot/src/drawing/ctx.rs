use std::sync::Arc;

use rustybuzz::ttf_parser;

use crate::{render, style};

pub struct Ctx<'a, S> {
    surface: &'a mut S,
    fontdb: Arc<fontdb::Database>,
}

impl<'a, S> Ctx<'a, S> {
    pub fn new(surface: &'a mut S, fontdb: Arc<fontdb::Database>) -> Ctx<'a, S> {
        Ctx { surface, fontdb }
    }
}

impl<'a, S> render::Surface for Ctx<'a, S>
where
    S: render::Surface,
{
    type Error = S::Error;

    fn prepare(&mut self, size: crate::geom::Size) -> Result<(), Self::Error> {
        self.surface.prepare(size)
    }

    fn fill(&mut self, fill: style::Fill) -> Result<(), Self::Error> {
        self.surface.fill(fill)
    }

    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), Self::Error> {
        self.surface.draw_rect(rect)
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), Self::Error> {
        self.surface.draw_path(path)
    }

    fn draw_text(&mut self, text: &render::Text) -> Result<(), Self::Error> {
        self.surface.draw_text(text)
    }

    fn push_clip(&mut self, clip: &render::Clip) -> Result<(), Self::Error> {
        self.surface.push_clip(clip)
    }

    fn pop_clip(&mut self) -> Result<(), Self::Error> {
        self.surface.pop_clip()
    }
}

impl<'a, S> Ctx<'a, S> {
    pub fn max_labels_width<I, L>(&self, font: &style::Font, labels: I) -> f32
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>,
    {
        let families = parse_font_family(font.family().as_str());
        let query = fontdb::Query {
            families: &families,
            ..Default::default()
        };
        // FIXME: error mgmt
        let id = self.fontdb.query(&query).expect("Should find a face");
        self.fontdb
            .with_face_data(id, |data, index| {
                let face = ttf_parser::Face::parse(data, index).unwrap();
                let units_per_em = face.units_per_em() as f32;
                let hbf = rustybuzz::Face::from_face(face);
                let scale = font.size() / units_per_em;
                let mut max_w = f32::NAN;
                for lbl in labels {
                    let lbl = lbl.as_ref();
                    let mut buffer = rustybuzz::UnicodeBuffer::new();
                    buffer.push_str(lbl);
                    let glyph_buffer = rustybuzz::shape(&hbf, &[], buffer);
                    let w: i32 = glyph_buffer
                        .glyph_positions()
                        .iter()
                        .map(|gp| gp.x_advance)
                        .sum();
                    max_w = max_w.max(w as f32 * scale);
                }
                max_w
            })
            .expect("Should find face data")
    }
}

fn parse_font_family(input: &str) -> Vec<fontdb::Family> {
    let mut families = Vec::new();
    let parts = input.split(',').map(|s| s.trim());

    for part in parts {
        let part = part.trim();
        let family = match part {
            "serif" => fontdb::Family::Serif,
            "sans-serif" => fontdb::Family::SansSerif,
            "cursive" => fontdb::Family::Cursive,
            "fantasy" => fontdb::Family::Fantasy,
            "monospace" => fontdb::Family::Monospace,
            _ => {
                // Remove surrounding quotes if present
                let name = part.trim_matches('\'').trim_matches('"');
                fontdb::Family::Name(name)
            }
        };
        families.push(family);
    }

    families
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_font_family() {
        let input = "'Noto Sans', 'Open Sans', sans-serif";
        let expected = vec![
            fontdb::Family::Name("Noto Sans"),
            fontdb::Family::Name("Open Sans"),
            fontdb::Family::SansSerif,
        ];
        assert_eq!(parse_font_family(input), expected);
    }
}
