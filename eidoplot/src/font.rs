use std::sync::Arc;

use rustybuzz::ttf_parser;

use crate::{style};

pub use fontdb::{Database, ID};

/// Loads fonts that are bundled with eidoplot
/// and returns an Arc to the database.
pub fn bundled_db() -> Arc<Database> {
    const FONTDB_FAMILY_SANS: &str = "Noto Sans";
    const FONTDB_FAMILY_SERIF: &str = "Noto Serif";
    const FONTDB_FAMILY_MONO: &str = "Noto Mono";

    let res_dir = crate::resource_folder();

    let mut db = Database::new();

    // db.load_system_fonts();

    db.load_fonts_dir(&res_dir);
    db.set_sans_serif_family(FONTDB_FAMILY_SANS);
    db.set_serif_family(FONTDB_FAMILY_SERIF);
    db.set_monospace_family(FONTDB_FAMILY_MONO);

    Arc::new(db)
}

pub trait DatabaseExt {
    fn query_face(&self, font: &style::Font) -> Option<ID>;

    fn label_width(&self, label: &str, font: &style::Font) -> f32 {
        self.max_labels_width(&[label], font)
    }

    fn max_labels_width<I, L>(&self, labels: I, font: &style::Font) -> f32
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>;

}

impl DatabaseExt for Database {
    fn query_face(&self, font: &style::Font) -> Option<ID> {
        let families = parse_font_family(font.family().as_str());
        let weight = fontdb::Weight(font.weight().0);
        let stretch = match font.width() {
            style::font::Width::UltraCondensed => fontdb::Stretch::UltraCondensed,
            style::font::Width::ExtraCondensed => fontdb::Stretch::ExtraCondensed,
            style::font::Width::Condensed => fontdb::Stretch::Condensed,
            style::font::Width::SemiCondensed => fontdb::Stretch::SemiCondensed,
            style::font::Width::Normal => fontdb::Stretch::Normal,
            style::font::Width::SemiExpanded => fontdb::Stretch::SemiExpanded,
            style::font::Width::Expanded => fontdb::Stretch::Expanded,
            style::font::Width::ExtraExpanded => fontdb::Stretch::ExtraExpanded,
            style::font::Width::UltraExpanded => fontdb::Stretch::UltraExpanded,
        };
        let style = match font.style() {
            style::font::Style::Normal => fontdb::Style::Normal,
            style::font::Style::Italic => fontdb::Style::Italic,
            style::font::Style::Oblique => fontdb::Style::Oblique,
        };
        let query = fontdb::Query {
            families: &families,
            weight,
            stretch,
            style,
        };
        self.query(&query)
    }

    fn max_labels_width<I, L>(&self, labels: I, font: &style::Font) -> f32
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>,
    {
        // FIXME: error mgmt
        let id = self.query_face(font).expect("Should find a face");

        self.with_face_data(id, |data, index| {
            let mut face = ttf_parser::Face::parse(data, index).unwrap();
            if face.is_variable() {
                let _ = face
                    .set_variation(ttf_parser::Tag::from_bytes(b"wght"), font.weight().0 as f32);
                let _ = face.set_variation(
                    ttf_parser::Tag::from_bytes(b"wdth"),
                    width_to_percent(font.width()),
                );
            }
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

pub fn width_to_percent(font: style::font::Width) -> f32 {
    match font {
        style::font::Width::UltraCondensed => 50.0,
        style::font::Width::ExtraCondensed => 62.5,
        style::font::Width::Condensed => 75.0,
        style::font::Width::SemiCondensed => 87.5,
        style::font::Width::Normal => 100.0,
        style::font::Width::SemiExpanded => 112.5,
        style::font::Width::Expanded => 125.0,
        style::font::Width::ExtraExpanded => 150.0,
        style::font::Width::UltraExpanded => 200.0,
    }
}

fn parse_font_family(input: &str) -> Vec<fontdb::Family<'_>> {
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
