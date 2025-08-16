pub use fontdb::{Database, ID};
use ttf_parser as ttf;

use crate::style;

pub trait DatabaseExt {
    fn has_char(&self, id: ID, c: char) -> bool;
    fn has_chars<C>(&self, id: ID, chars: C) -> bool
    where
        C: Iterator<Item = char>;
}

impl DatabaseExt for Database {
    fn has_char(&self, id: ID, c: char) -> bool {
        let res = self.with_face_data(id, |data, index| -> Option<bool> {
            let face = ttf::Face::parse(data, index).ok()?;
            face.glyph_index(c)?;
            Some(true)
        });
        res == Some(Some(true))
    }

    fn has_chars<C>(&self, id: ID, chars: C) -> bool
    where
        C: Iterator<Item = char>,
    {
        let res = self.with_face_data(id, |data, index| -> Option<bool> {
            let face = ttf::Face::parse(data, index).ok()?;
            for c in chars {
                face.glyph_index(c)?;
            }
            Some(true)
        });
        res == Some(Some(true))
    }
}

pub fn select_face(db: &Database, font: &style::Font) -> Option<ID> {
    let families = parse_font_family(font.family().as_str());
    let weight = fontdb::Weight(font.weight().0);
    let stretch = match font.width() {
        style::Width::UltraCondensed => fontdb::Stretch::UltraCondensed,
        style::Width::ExtraCondensed => fontdb::Stretch::ExtraCondensed,
        style::Width::Condensed => fontdb::Stretch::Condensed,
        style::Width::SemiCondensed => fontdb::Stretch::SemiCondensed,
        style::Width::Normal => fontdb::Stretch::Normal,
        style::Width::SemiExpanded => fontdb::Stretch::SemiExpanded,
        style::Width::Expanded => fontdb::Stretch::Expanded,
        style::Width::ExtraExpanded => fontdb::Stretch::ExtraExpanded,
        style::Width::UltraExpanded => fontdb::Stretch::UltraExpanded,
    };
    let style = match font.style() {
        style::Style::Normal => fontdb::Style::Normal,
        style::Style::Italic => fontdb::Style::Italic,
        style::Style::Oblique => fontdb::Style::Oblique,
    };
    let query = fontdb::Query {
        families: &families,
        weight,
        stretch,
        style,
    };
    db.query(&query)
}

pub fn select_face_fallback(db: &Database, c: char, already_tried: &[ID]) -> Option<ID> {
    let base_face = db.face(already_tried[0])?;

    for face in db.faces() {
        if already_tried.contains(&face.id) {
            continue;
        }
        if face.style != base_face.style {
            continue;
        }
        if face.weight != base_face.weight {
            continue;
        }
        if face.stretch != base_face.stretch {
            continue;
        }
        if !db.has_char(face.id, c) {
            continue;
        }
        return Some(face.id);
    }
    None
}

pub(crate) fn apply_variations(face: &mut ttf::Face, style: &style::Font) {
    if face.is_variable() && face.weight().to_number() != style.weight().to_number() {
        let _ = face.set_variation(ttf::Tag::from_bytes(b"wght"), style.weight().to_var_value());
    }
    if face.is_variable() && face.width().to_number() != style.width().to_number() {
        let _ = face.set_variation(ttf::Tag::from_bytes(b"wdth"), style.width().to_var_value());
    }
}

/// A font that has been resolved, but not scaled
#[derive(Debug, Clone, Copy)]
pub(crate) struct FaceMetrics {
    // all values in font units
    units_per_em: u16,
    ascent: i16,
    descent: i16,
    x_height: i16,
    cap_height: i16,
    line_gap: i16,
}

impl FaceMetrics {
    pub(crate) fn scale(&self, size: f32) -> f32 {
        size / self.units_per_em as f32
    }

    pub(crate) fn ascent(&self, size: f32) -> f32 {
        self.ascent as f32 * self.scale(size)
    }

    pub(crate) fn descent(&self, size: f32) -> f32 {
        self.descent as f32 * self.scale(size)
    }

    pub(crate) fn x_height(&self, size: f32) -> f32 {
        self.x_height as f32 * self.scale(size)
    }

    pub(crate) fn cap_height(&self, size: f32) -> f32 {
        self.cap_height as f32 * self.scale(size)
    }

    pub(crate) fn height(&self, size: f32) -> f32 {
        (self.ascent - self.descent) as f32 * self.scale(size)
    }

    pub(crate) fn line_gap(&self, size: f32) -> f32 {
        self.line_gap as f32 * self.scale(size)
    }
}

pub(crate) fn face_metrics(face: &ttf::Face) -> FaceMetrics {
    let units_per_em = face.units_per_em();
    let ascent = face.ascender();
    let descent = face.descender();
    let x_height = face
        .x_height()
        .unwrap_or(((ascent - descent) as f32 * 0.45) as i16);
    let cap_height = face
        .capital_height()
        .unwrap_or(((ascent - descent) as f32 * 0.8) as i16);
    let line_gap = face.line_gap();

    FaceMetrics {
        units_per_em,
        ascent,
        descent,
        x_height,
        cap_height,
        line_gap,
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
