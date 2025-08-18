use std::fmt;

pub use fontdb::{Database, ID};
use ttf_parser as ttf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Family {
    SansSerif,
    Serif,
    Monospace,
    Cursive,
    Fantasy,
    Named(String),
}

#[derive(Debug, PartialEq)]
pub enum FamilyError {
    UnclosedQuote,
    EmptyFamilyName,
    InvalidCharacterInName,
}

impl fmt::Display for FamilyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FamilyError::UnclosedQuote => write!(f, "unclosed quote"),
            FamilyError::EmptyFamilyName => write!(f, "empty family name"),
            FamilyError::InvalidCharacterInName => write!(f, "invalid character in family name"),
        }
    }
}

impl std::error::Error for FamilyError {}

pub fn parse_font_families(input: &str) -> Result<Vec<Family>, FamilyError> {
    let mut families = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '\0';

    for c in input.chars() {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            q if q == quote_char && in_quotes => {
                in_quotes = false;
                quote_char = '\0';
            }
            ',' if !in_quotes => {
                let family = current.trim();
                if !family.is_empty() {
                    families.push(parse_single_font_family(family)?);
                } else if !current.is_empty() {
                    return Err(FamilyError::EmptyFamilyName);
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }

    // Gestion de la dernière famille
    if in_quotes {
        return Err(FamilyError::UnclosedQuote);
    }
    let family = current.trim();
    if !family.is_empty() {
        families.push(parse_single_font_family(family)?);
    } else if !current.is_empty() {
        return Err(FamilyError::EmptyFamilyName);
    }

    Ok(families)
}

fn parse_single_font_family(family: &str) -> Result<Family, FamilyError> {
    let family = family.trim();
    if family.is_empty() {
        return Err(FamilyError::EmptyFamilyName);
    }

    // Vérification des caractères invalides (hors guillemets, déjà gérés)
    if family.contains(['"', '\'']) {
        return Err(FamilyError::InvalidCharacterInName);
    }

    match family.to_ascii_lowercase().as_str() {
        "sans-serif" => Ok(Family::SansSerif),
        "serif" => Ok(Family::Serif),
        "monospace" => Ok(Family::Monospace),
        "cursive" => Ok(Family::Cursive),
        "fantasy" => Ok(Family::Fantasy),
        _ => Ok(Family::Named(family.to_string())),
    }
}

/// Specifies the weight of glyphs in the font, their degree of blackness or stroke thickness.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct Weight(pub u16);

impl Default for Weight {
    fn default() -> Weight {
        Weight::NORMAL
    }
}

impl Weight {
    /// Thin weight (100), the thinnest value.
    pub const THIN: Weight = Weight(100);
    /// Extra light weight (200).
    pub const EXTRA_LIGHT: Weight = Weight(200);
    /// Light weight (300).
    pub const LIGHT: Weight = Weight(300);
    /// Normal (400).
    pub const NORMAL: Weight = Weight(400);
    /// Medium weight (500, higher than normal).
    pub const MEDIUM: Weight = Weight(500);
    /// Semibold weight (600).
    pub const SEMIBOLD: Weight = Weight(600);
    /// Bold weight (700).
    pub const BOLD: Weight = Weight(700);
    /// Extra-bold weight (800).
    pub const EXTRA_BOLD: Weight = Weight(800);
    /// Black weight (900), the thickest value.
    pub const BLACK: Weight = Weight(900);

    pub fn to_number(&self) -> u16 {
        self.0
    }

    /// Returns a numeric representation of a weight suitable for font variations
    pub fn to_var_value(&self) -> f32 {
        self.0 as f32
    }
}

/// Allows italic or oblique faces to be selected.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Style {
    /// A face that is neither italic not obliqued.
    Normal,
    /// A form that is generally cursive in nature.
    Italic,
    /// A typically-sloped version of the regular face.
    Oblique,
}

impl Default for Style {
    fn default() -> Style {
        Style::Normal
    }
}

/// A face [width](https://docs.microsoft.com/en-us/typography/opentype/spec/os2#uswidthclass).
#[allow(missing_docs)]
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Width {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl Width {
    /// Returns a numeric representation of a width.
    pub fn to_number(self) -> u16 {
        match self {
            Width::UltraCondensed => 1,
            Width::ExtraCondensed => 2,
            Width::Condensed => 3,
            Width::SemiCondensed => 4,
            Width::Normal => 5,
            Width::SemiExpanded => 6,
            Width::Expanded => 7,
            Width::ExtraExpanded => 8,
            Width::UltraExpanded => 9,
        }
    }

    pub fn to_percent(self) -> f32 {
        match self {
            Width::UltraCondensed => 50.0,
            Width::ExtraCondensed => 62.5,
            Width::Condensed => 75.0,
            Width::SemiCondensed => 87.5,
            Width::Normal => 100.0,
            Width::SemiExpanded => 112.5,
            Width::Expanded => 125.0,
            Width::ExtraExpanded => 150.0,
            Width::UltraExpanded => 200.0,
        }
    }

    /// Get the width as a value suitable for font variations
    pub fn to_var_value(self) -> f32 {
        self.to_percent()
    }
}

impl Default for Width {
    fn default() -> Self {
        Width::Normal
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Font {
    families: Vec<Family>,
    weight: Weight,
    width: Width,
    style: Style,
}

impl Default for Font {
    fn default() -> Self {
        Font::new(vec![Family::Serif])
    }
}

impl Font {
    pub fn new(families: Vec<Family>) -> Self {
        Font {
            families,
            weight: Weight::NORMAL,
            width: Width::Normal,
            style: Style::Normal,
        }
    }

    pub fn with_families(self, families: Vec<Family>) -> Self {
        Font { families, ..self }
    }

    pub fn with_weight(self, weight: Weight) -> Self {
        Font { weight, ..self }
    }

    pub fn with_width(self, width: Width) -> Self {
        Font { width, ..self }
    }

    pub fn with_style(self, style: Style) -> Self {
        Font { style, ..self }
    }

    pub fn families(&self) -> &[Family] {
        &self.families
    }

    pub fn weight(&self) -> Weight {
        self.weight
    }

    pub fn width(&self) -> Width {
        self.width
    }

    pub fn style(&self) -> Style {
        self.style
    }
}

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

fn to_fontdb_family(family: &Family) -> fontdb::Family<'_> {
    match family {
        Family::Named(name) => fontdb::Family::Name(name.as_str()),
        Family::SansSerif => fontdb::Family::SansSerif,
        Family::Serif => fontdb::Family::Serif,
        Family::Monospace => fontdb::Family::Monospace,
        Family::Cursive => fontdb::Family::Cursive,
        Family::Fantasy => fontdb::Family::Fantasy,
    } 
}

pub fn select_face(db: &Database, font: &Font) -> Option<ID> {
    let families: Vec<_> = font.families.iter().map(to_fontdb_family).collect();
    let weight = fontdb::Weight(font.weight().0);
    let stretch = match font.width() {
        Width::UltraCondensed => fontdb::Stretch::UltraCondensed,
        Width::ExtraCondensed => fontdb::Stretch::ExtraCondensed,
        Width::Condensed => fontdb::Stretch::Condensed,
        Width::SemiCondensed => fontdb::Stretch::SemiCondensed,
        Width::Normal => fontdb::Stretch::Normal,
        Width::SemiExpanded => fontdb::Stretch::SemiExpanded,
        Width::Expanded => fontdb::Stretch::Expanded,
        Width::ExtraExpanded => fontdb::Stretch::ExtraExpanded,
        Width::UltraExpanded => fontdb::Stretch::UltraExpanded,
    };
    let style = match font.style() {
        Style::Normal => fontdb::Style::Normal,
        Style::Italic => fontdb::Style::Italic,
        Style::Oblique => fontdb::Style::Oblique,
    };
    let query = fontdb::Query {
        families: families.as_slice(),
        weight,
        stretch,
        style,
    };
    println!("Performing query {:?}", query);
    let res = db.query(&query);
    println!("Obtained result: {:#?}", res.map(|id| db.face(id).unwrap()));
    res
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

pub(crate) fn apply_variations(face: &mut ttf::Face, font: &Font) {
    if face.is_variable() && face.weight().to_number() != font.weight().to_number() {
        let _ = face.set_variation(ttf::Tag::from_bytes(b"wght"), font.weight().to_var_value());
    }
    if face.is_variable() && face.width().to_number() != font.width().to_number() {
        let _ = face.set_variation(ttf::Tag::from_bytes(b"wdth"), font.width().to_var_value());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_font_family_simple() {
        let input = "Arial, sans-serif, monospace";
        let expected = Ok(vec![
            Family::Named("Arial".to_string()),
            Family::SansSerif,
            Family::Monospace,
        ]);
        assert_eq!(parse_font_families(input), expected);
    }

    #[test]
    fn test_parse_font_family_with_quotes() {
        let input = r#""Times New Roman", 'Courier New', fantasy"#;
        let expected = Ok(vec![
            Family::Named("Times New Roman".to_string()),
            Family::Named("Courier New".to_string()),
            Family::Fantasy,
        ]);
        assert_eq!(parse_font_families(input), expected);
    }

    #[test]
    fn test_parse_font_family_with_comma_in_name() {
        let input = r#"Arial, "Times New Roman, Bold", serif"#;
        let expected = Ok(vec![
            Family::Named("Arial".to_string()),
            Family::Named("Times New Roman, Bold".to_string()),
            Family::Serif,
        ]);
        assert_eq!(parse_font_families(input), expected);
    }

    #[test]
    fn test_parse_font_family_empty() {
        let input = "";
        let expected = Ok(vec![]);
        assert_eq!(parse_font_families(input), expected);
    }

    #[test]
    fn test_parse_font_family_unclosed_quote() {
        let input = r#"Arial, "Times New Roman"#;
        let expected = Err(FamilyError::UnclosedQuote);
        assert_eq!(parse_font_families(input), expected);
    }

    #[test]
    fn test_parse_font_family_empty_name() {
        let input = "Arial, , monospace";
        let expected = Err(FamilyError::EmptyFamilyName);
        assert_eq!(parse_font_families(input), expected);
    }

    #[test]
    fn test_parse_font_family_invalid_character() {
        let input = r#"Arial, "Times" New Roman"#;
        let expected = Err(FamilyError::InvalidCharacterInName);
        assert_eq!(parse_font_families(input), expected);
    }

    #[test]
    fn test_parse_font_family_all_keywords() {
        let input = "serif, sans-serif, monospace, cursive, fantasy";
        let expected = Ok(vec![
            Family::Serif,
            Family::SansSerif,
            Family::Monospace,
            Family::Cursive,
            Family::Fantasy,
        ]);
        assert_eq!(parse_font_families(input), expected);
    }
}