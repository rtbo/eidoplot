use std::sync::Arc;

use crate::style;

use rustybuzz::ttf_parser;

#[derive(Debug, Clone)]
pub struct FontDb(Arc<fontdb::Database>);

impl FontDb {
    pub fn new(fontdb: Arc<fontdb::Database>) -> Self {
        Self(fontdb)
    }
}

impl FontDb {
    /// Compute the width of a label
    pub fn label_width(&self, label: &str, font: &style::Font) -> f32 {
        self.max_labels_width(&[label], font)
    }

    pub fn max_labels_width<I, L>(&self, labels: I, font: &style::Font) -> f32
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
        let id = self.0.query(&query).expect("Should find a face");
        self.0
            .with_face_data(id, |data, index| {
                let face = ttf_parser::Face::parse(data, index).unwrap();
                let units_per_em = face.units_per_em() as f32;
                let hbf = rustybuzz::Face::from_face(face);
                let scale = font.size() / units_per_em;
                let mut max_w = f32::NAN;
                let mut buffer = rustybuzz::UnicodeBuffer::new();
                for lbl in labels {
                    let lbl = lbl.as_ref();
                    buffer.push_str(lbl);
                    let glyph_buffer = rustybuzz::shape(&hbf, &[], buffer);
                    let w: i32 = glyph_buffer
                        .glyph_positions()
                        .iter()
                        .map(|gp| gp.x_advance)
                        .sum();
                    max_w = max_w.max(w as f32 * scale);
                    buffer = glyph_buffer.clear();
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
