use std::{fmt, str};

pub use fontdb::{Database, ID};
use ttf_parser as ttf;

use crate::fontdb;

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
    let mut had_quotes = false;

    for c in input.chars() {
        match c {
            '"' | '\'' if !in_quotes => {
                if !current.trim().is_empty() {
                    // quote char in the middle of the name
                    return Err(FamilyError::InvalidCharacterInName);
                }
                in_quotes = true;
                had_quotes = true;
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
                had_quotes = false;
                current.clear();
            }
            _ => {
                if !c.is_whitespace() && had_quotes && !in_quotes {
                    // quote char in the middle of the name
                    return Err(FamilyError::InvalidCharacterInName);
                }
                current.push(c)
            }
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

pub fn font_families_to_string(families: &[Family]) -> String {
    let mut result = String::new();
    for (i, f) in families.iter().enumerate() {
        if i != 0 {
            result.push(',');
        }
        match f {
            Family::SansSerif => result.push_str("sans-serif"),
            Family::Serif => result.push_str("serif"),
            Family::Monospace => result.push_str("monospace"),
            Family::Cursive => result.push_str("cursive"),
            Family::Fantasy => result.push_str("fantasy"),
            Family::Named(name) => {
                if name.chars().any(char::is_whitespace) {
                    result.push('"');
                    result.push_str(name);
                    result.push('"');
                } else {
                    result.push_str(name);
                }
            }
        }
    }
    result
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

    fn to_fontdb(self) -> fontdb::Weight {
        fontdb::Weight(self.0)
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

impl Style {
    fn to_fontdb(self) -> fontdb::Style {
        match self {
            Style::Normal => fontdb::Style::Normal,
            Style::Italic => fontdb::Style::Italic,
            Style::Oblique => fontdb::Style::Oblique,
        }
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

    fn to_fontdb(self) -> fontdb::Stretch {
        match self {
            Width::UltraCondensed => fontdb::Stretch::UltraCondensed,
            Width::ExtraCondensed => fontdb::Stretch::ExtraCondensed,
            Width::Condensed => fontdb::Stretch::Condensed,
            Width::SemiCondensed => fontdb::Stretch::SemiCondensed,
            Width::Normal => fontdb::Stretch::Normal,
            Width::SemiExpanded => fontdb::Stretch::SemiExpanded,
            Width::Expanded => fontdb::Stretch::Expanded,
            Width::ExtraExpanded => fontdb::Stretch::ExtraExpanded,
            Width::UltraExpanded => fontdb::Stretch::UltraExpanded,
        }
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

impl str::FromStr for Font {
    type Err = FamilyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let families = parse_font_families(s)?;
        Ok(Font::new(families))
    }
}

impl Default for Font {
    fn default() -> Self {
        Font::new(vec![Family::SansSerif])
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

    /// Perform a CSS-like query for the provided font spec.
    fn select_face(&self, font: &Font) -> Option<ID>;

    /// Perform a CSS-like query for the provided font spec with a filter to ensure
    /// that the returned face can render all characters of the string
    fn select_face_for_str(&self, font: &Font, s: &str) -> Option<ID>;

    fn select_face_fallback(&self, s: &str, already_tried: &[ID]) -> Option<ID>;
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

    fn select_face(&self, font: &Font) -> Option<ID> {
        let families: Vec<_> = font.families.iter().map(to_fontdb_family).collect();
        let query = fontdb::Query {
            families: families.as_slice(),
            weight: font.weight().to_fontdb(),
            stretch: font.width().to_fontdb(),
            style: font.style().to_fontdb(),
        };

        self.query(&query)
    }

    fn select_face_for_str(&self, font: &Font, s: &str) -> Option<ID> {
        // same as query implementation of fontdb with the additional unicode_ranges filter
        let ur = unicode_ranges_for_str(s);

        for family in &font.families {
            let fdbfamily = to_fontdb_family(family);
            let name = self.family_name(&fdbfamily);
            let candidates: Vec<_> = self
                .faces()
                .filter(|f| {
                    if let Some(fur) = f.unicode_ranges {
                        unicode_ranges_contains(fur, ur)
                    } else {
                        false
                    }
                })
                .filter(|face| face.families.iter().any(|family| family.0 == name))
                .collect();

            if !candidates.is_empty() {
                let query = fontdb::Query {
                    families: &[fdbfamily],
                    weight: fontdb::Weight(font.weight().0),
                    stretch: font.width().to_fontdb(),
                    style: font.style().to_fontdb(),
                };
                if let Some(index) = find_best_match(&candidates, &query) {
                    return Some(candidates[index].id);
                }
            }
        }
        None
    }

    fn select_face_fallback(&self, s: &str, already_tried: &[ID]) -> Option<ID> {
        let base_face = self.face(already_tried[0])?;

        for face in self.faces() {
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
            if !self.has_chars(face.id, s.chars()) {
                continue;
            }
            return Some(face.id);
        }
        None
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

pub fn unicode_ranges_for_str(s: &str) -> ttf::UnicodeRanges {
    let mut ranges = 0u128;
    for c in s.chars() {
        let index = char_range_index(c);
        ranges |= 1 << index;
    }
    ttf::UnicodeRanges(ranges)
}

/// Check whether `ranges` contains all the characters in `sample`
pub fn unicode_ranges_contains(ranges: ttf::UnicodeRanges, sample: ttf::UnicodeRanges) -> bool {
    (ranges.0 & sample.0) == sample.0
}

// Function copied from ttf-parser
fn char_range_index(c: char) -> i8 {
    match c as u32 {
        0x0000..=0x007F => 0,
        0x0080..=0x00FF => 1,
        0x0100..=0x017F => 2,
        0x0180..=0x024F => 3,
        0x0250..=0x02AF => 4,
        0x1D00..=0x1DBF => 4,
        0x02B0..=0x02FF => 5,
        0xA700..=0xA71F => 5,
        0x0300..=0x036F => 6,
        0x1DC0..=0x1DFF => 6,
        0x0370..=0x03FF => 7,
        0x2C80..=0x2CFF => 8,
        0x0400..=0x052F => 9,
        0x2DE0..=0x2DFF => 9,
        0xA640..=0xA69F => 9,
        0x0530..=0x058F => 10,
        0x0590..=0x05FF => 11,
        0xA500..=0xA63F => 12,
        0x0600..=0x06FF => 13,
        0x0750..=0x077F => 13,
        0x07C0..=0x07FF => 14,
        0x0900..=0x097F => 15,
        0x0980..=0x09FF => 16,
        0x0A00..=0x0A7F => 17,
        0x0A80..=0x0AFF => 18,
        0x0B00..=0x0B7F => 19,
        0x0B80..=0x0BFF => 20,
        0x0C00..=0x0C7F => 21,
        0x0C80..=0x0CFF => 22,
        0x0D00..=0x0D7F => 23,
        0x0E00..=0x0E7F => 24,
        0x0E80..=0x0EFF => 25,
        0x10A0..=0x10FF => 26,
        0x2D00..=0x2D2F => 26,
        0x1B00..=0x1B7F => 27,
        0x1100..=0x11FF => 28,
        0x1E00..=0x1EFF => 29,
        0x2C60..=0x2C7F => 29,
        0xA720..=0xA7FF => 29,
        0x1F00..=0x1FFF => 30,
        0x2000..=0x206F => 31,
        0x2E00..=0x2E7F => 31,
        0x2070..=0x209F => 32,
        0x20A0..=0x20CF => 33,
        0x20D0..=0x20FF => 34,
        0x2100..=0x214F => 35,
        0x2150..=0x218F => 36,
        0x2190..=0x21FF => 37,
        0x27F0..=0x27FF => 37,
        0x2900..=0x297F => 37,
        0x2B00..=0x2BFF => 37,
        0x2200..=0x22FF => 38,
        0x2A00..=0x2AFF => 38,
        0x27C0..=0x27EF => 38,
        0x2980..=0x29FF => 38,
        0x2300..=0x23FF => 39,
        0x2400..=0x243F => 40,
        0x2440..=0x245F => 41,
        0x2460..=0x24FF => 42,
        0x2500..=0x257F => 43,
        0x2580..=0x259F => 44,
        0x25A0..=0x25FF => 45,
        0x2600..=0x26FF => 46,
        0x2700..=0x27BF => 47,
        0x3000..=0x303F => 48,
        0x3040..=0x309F => 49,
        0x30A0..=0x30FF => 50,
        0x31F0..=0x31FF => 50,
        0x3100..=0x312F => 51,
        0x31A0..=0x31BF => 51,
        0x3130..=0x318F => 52,
        0xA840..=0xA87F => 53,
        0x3200..=0x32FF => 54,
        0x3300..=0x33FF => 55,
        0xAC00..=0xD7AF => 56,
        // Ignore Non-Plane 0 (57), since this is not a real range.
        0x10900..=0x1091F => 58,
        0x4E00..=0x9FFF => 59,
        0x2E80..=0x2FDF => 59,
        0x2FF0..=0x2FFF => 59,
        0x3400..=0x4DBF => 59,
        0x20000..=0x2A6DF => 59,
        0x3190..=0x319F => 59,
        0xE000..=0xF8FF => 60,
        0x31C0..=0x31EF => 61,
        0xF900..=0xFAFF => 61,
        0x2F800..=0x2FA1F => 61,
        0xFB00..=0xFB4F => 62,
        0xFB50..=0xFDFF => 63,
        0xFE20..=0xFE2F => 64,
        0xFE10..=0xFE1F => 65,
        0xFE30..=0xFE4F => 65,
        0xFE50..=0xFE6F => 66,
        0xFE70..=0xFEFF => 67,
        0xFF00..=0xFFEF => 68,
        0xFFF0..=0xFFFF => 69,
        0x0F00..=0x0FFF => 70,
        0x0700..=0x074F => 71,
        0x0780..=0x07BF => 72,
        0x0D80..=0x0DFF => 73,
        0x1000..=0x109F => 74,
        0x1200..=0x139F => 75,
        0x2D80..=0x2DDF => 75,
        0x13A0..=0x13FF => 76,
        0x1400..=0x167F => 77,
        0x1680..=0x169F => 78,
        0x16A0..=0x16FF => 79,
        0x1780..=0x17FF => 80,
        0x19E0..=0x19FF => 80,
        0x1800..=0x18AF => 81,
        0x2800..=0x28FF => 82,
        0xA000..=0xA48F => 83,
        0xA490..=0xA4CF => 83,
        0x1700..=0x177F => 84,
        0x10300..=0x1032F => 85,
        0x10330..=0x1034F => 86,
        0x10400..=0x1044F => 87,
        0x1D000..=0x1D24F => 88,
        0x1D400..=0x1D7FF => 89,
        0xF0000..=0xFFFFD => 90,
        0x100000..=0x10FFFD => 90,
        0xFE00..=0xFE0F => 91,
        0xE0100..=0xE01EF => 91,
        0xE0000..=0xE007F => 92,
        0x1900..=0x194F => 93,
        0x1950..=0x197F => 94,
        0x1980..=0x19DF => 95,
        0x1A00..=0x1A1F => 96,
        0x2C00..=0x2C5F => 97,
        0x2D30..=0x2D7F => 98,
        0x4DC0..=0x4DFF => 99,
        0xA800..=0xA82F => 100,
        0x10000..=0x1013F => 101,
        0x10140..=0x1018F => 102,
        0x10380..=0x1039F => 103,
        0x103A0..=0x103DF => 104,
        0x10450..=0x1047F => 105,
        0x10480..=0x104AF => 106,
        0x10800..=0x1083F => 107,
        0x10A00..=0x10A5F => 108,
        0x1D300..=0x1D35F => 109,
        0x12000..=0x123FF => 110,
        0x12400..=0x1247F => 110,
        0x1D360..=0x1D37F => 111,
        0x1B80..=0x1BBF => 112,
        0x1C00..=0x1C4F => 113,
        0x1C50..=0x1C7F => 114,
        0xA880..=0xA8DF => 115,
        0xA900..=0xA92F => 116,
        0xA930..=0xA95F => 117,
        0xAA00..=0xAA5F => 118,
        0x10190..=0x101CF => 119,
        0x101D0..=0x101FF => 120,
        0x102A0..=0x102DF => 121,
        0x10280..=0x1029F => 121,
        0x10920..=0x1093F => 121,
        0x1F030..=0x1F09F => 122,
        0x1F000..=0x1F02F => 122,
        _ => -1,
    }
}

// https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#font-style-matching
// Based on https://github.com/servo/font-kit
// Function copied from fontdb
#[inline(never)]
fn find_best_match(candidates: &[&fontdb::FaceInfo], query: &fontdb::Query) -> Option<usize> {
    debug_assert!(!candidates.is_empty());

    // Step 4.
    let mut matching_set: Vec<usize> = (0..candidates.len()).collect();

    // Step 4a (`font-stretch`).
    let matches = matching_set
        .iter()
        .any(|&index| candidates[index].stretch == query.stretch);
    let matching_stretch = if matches {
        // Exact match.
        query.stretch
    } else if query.stretch <= fontdb::Stretch::Normal {
        // Closest stretch, first checking narrower values and then wider values.
        let stretch = matching_set
            .iter()
            .filter(|&&index| candidates[index].stretch < query.stretch)
            .min_by_key(|&&index| {
                query.stretch.to_number() - candidates[index].stretch.to_number()
            });

        match stretch {
            Some(&matching_index) => candidates[matching_index].stretch,
            None => {
                let matching_index = *matching_set.iter().min_by_key(|&&index| {
                    candidates[index].stretch.to_number() - query.stretch.to_number()
                })?;

                candidates[matching_index].stretch
            }
        }
    } else {
        // Closest stretch, first checking wider values and then narrower values.
        let stretch = matching_set
            .iter()
            .filter(|&&index| candidates[index].stretch > query.stretch)
            .min_by_key(|&&index| {
                candidates[index].stretch.to_number() - query.stretch.to_number()
            });

        match stretch {
            Some(&matching_index) => candidates[matching_index].stretch,
            None => {
                let matching_index = *matching_set.iter().min_by_key(|&&index| {
                    query.stretch.to_number() - candidates[index].stretch.to_number()
                })?;

                candidates[matching_index].stretch
            }
        }
    };
    matching_set.retain(|&index| candidates[index].stretch == matching_stretch);

    // Step 4b (`font-style`).
    let style_preference = match query.style {
        fontdb::Style::Italic => [
            fontdb::Style::Italic,
            fontdb::Style::Oblique,
            fontdb::Style::Normal,
        ],
        fontdb::Style::Oblique => [
            fontdb::Style::Oblique,
            fontdb::Style::Italic,
            fontdb::Style::Normal,
        ],
        fontdb::Style::Normal => [
            fontdb::Style::Normal,
            fontdb::Style::Oblique,
            fontdb::Style::Italic,
        ],
    };
    let matching_style = *style_preference.iter().find(|&query_style| {
        matching_set
            .iter()
            .any(|&index| candidates[index].style == *query_style)
    })?;

    matching_set.retain(|&index| candidates[index].style == matching_style);

    // Step 4c (`font-weight`).
    //
    // The spec doesn't say what to do if the weight is between 400 and 500 exclusive, so we
    // just use 450 as the cutoff.
    let weight = query.weight.0;

    let matching_weight = if matching_set
        .iter()
        .any(|&index| candidates[index].weight.0 == weight)
    {
        fontdb::Weight(weight)
    } else if (400..450).contains(&weight)
        && matching_set
            .iter()
            .any(|&index| candidates[index].weight.0 == 500)
    {
        // Check 500 first.
        fontdb::Weight::MEDIUM
    } else if (450..=500).contains(&weight)
        && matching_set
            .iter()
            .any(|&index| candidates[index].weight.0 == 400)
    {
        // Check 400 first.
        fontdb::Weight::NORMAL
    } else if weight <= 500 {
        // Closest weight, first checking thinner values and then fatter ones.
        let idx = matching_set
            .iter()
            .filter(|&&index| candidates[index].weight.0 <= weight)
            .min_by_key(|&&index| weight - candidates[index].weight.0);

        match idx {
            Some(&matching_index) => candidates[matching_index].weight,
            None => {
                let matching_index = *matching_set
                    .iter()
                    .min_by_key(|&&index| candidates[index].weight.0 - weight)?;
                candidates[matching_index].weight
            }
        }
    } else {
        // Closest weight, first checking fatter values and then thinner ones.
        let idx = matching_set
            .iter()
            .filter(|&&index| candidates[index].weight.0 >= weight)
            .min_by_key(|&&index| candidates[index].weight.0 - weight);

        match idx {
            Some(&matching_index) => candidates[matching_index].weight,
            None => {
                let matching_index = *matching_set
                    .iter()
                    .min_by_key(|&&index| weight - candidates[index].weight.0)?;
                candidates[matching_index].weight
            }
        }
    };
    matching_set.retain(|&index| candidates[index].weight == matching_weight);

    // Ignore step 4d (`font-size`).

    // Return the result.
    matching_set.into_iter().next()
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

pub(crate) fn apply_ttf_variations(face: &mut ttf::Face, font: &Font) {
    if face.is_variable() && face.weight().to_number() != font.weight().to_number() {
        let _ = face.set_variation(ttf::Tag::from_bytes(b"wght"), font.weight().to_var_value());
    }
    if face.is_variable() && face.width().to_number() != font.width().to_number() {
        let _ = face.set_variation(ttf::Tag::from_bytes(b"wdth"), font.width().to_var_value());
    }
}

pub(crate) fn apply_hb_variations(face: &mut rustybuzz::Face, font: &Font) {
    if face.is_variable() && face.weight().to_number() != font.weight().to_number() {
        let _ = face.set_variation(ttf::Tag::from_bytes(b"wght"), font.weight().to_var_value());
    }
    if face.is_variable() && face.width().to_number() != font.width().to_number() {
        let _ = face.set_variation(ttf::Tag::from_bytes(b"wdth"), font.width().to_var_value());
    }
}

/// A font that has been resolved, but not scaled
#[derive(Debug, Clone, Copy)]
pub(crate) struct Metrics {
    // all values in font units
    units_per_em: u16,
    pub(crate) ascent: i16,
    pub(crate) descent: i16,
    pub(crate) x_height: i16,
    pub(crate) cap_height: i16,
    pub(crate) line_gap: i16,
    pub(crate) uline: ttf::LineMetrics,
    pub(crate) strikeout: ttf::LineMetrics,
}

impl Metrics {
    pub(crate) fn height(&self) -> i16 {
        self.ascent - self.descent
    }

    pub(crate) fn scale(&self, size: f32) -> f32 {
        size / self.units_per_em as f32
    }

    pub(crate) fn scaled_height(&self, size: f32) -> f32 {
        (self.ascent - self.descent) as f32 * self.scale(size)
    }

    pub(crate) fn scaled_line_gap(&self, size: f32) -> f32 {
        self.line_gap as f32 * self.scale(size)
    }

    pub(crate) fn scaled(&self, size: f32) -> ScaledMetrics {
        let scale = self.scale(size);
        ScaledMetrics {
            scale,
            em_size: self.units_per_em as f32 * scale,
            ascent: self.ascent as f32 * scale,
            descent: self.descent as f32 * scale,
            x_height: self.x_height as f32 * scale,
            cap_height: self.cap_height as f32 * scale,
            line_gap: self.line_gap as f32 * scale,
            uline: ScaledLineMetrics {
                position: self.uline.position as f32 * scale,
                thickness: self.uline.thickness as f32 * scale,
            },
            strikeout: ScaledLineMetrics {
                position: self.strikeout.position as f32 * scale,
                thickness: self.strikeout.thickness as f32 * scale,
            },
        }
    }
}

pub(crate) fn face_metrics(face: &ttf::Face) -> Metrics {
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

    let uline = face.underline_metrics().unwrap_or_else (|| {
        println!("no underline metrics");
        ttf::LineMetrics {
            position: -(ascent as f32 * 0.1) as i16,
            thickness: (units_per_em as f32 / 14.0) as i16,
        }
    });

    let strikeout = face.strikeout_metrics().unwrap_or_else (|| {
        ttf::LineMetrics {
            position: (x_height as f32 * 0.5) as i16,
            thickness: (units_per_em as f32 / 14.0) as i16,
        }
    });

    Metrics {
        units_per_em,
        ascent,
        descent,
        x_height,
        cap_height,
        line_gap,
        uline,
        strikeout,
    }
}

#[derive(Debug, Clone, Copy)]
/// Metrics of a scaled line (underline or strikeout)
pub struct ScaledLineMetrics {
    /// Position of the line
    pub position: f32,
    /// Thickness of the line
    pub thickness: f32,
}

/// Metrics of a font face, scaled to a font size
#[derive(Debug, Clone, Copy)]
pub struct ScaledMetrics {
    /// Scale factor from the original font units metrics
    pub scale: f32,
    /// Size of the em box after scaling
    pub em_size: f32,
    /// Height between baseline and top of the font face
    pub ascent: f32,
    /// Height between baseline and bottom of the font face (negative value)
    pub descent: f32,
    /// Height between baseline and top of most letters
    /// (the half of this height is used for the "middle" alignment)
    pub x_height: f32,
    /// Height between baseline and top of capitals (used for the "hanging" alignment)
    pub cap_height: f32,
    /// Gap to be added (possibly zero) to the distance between two lines,
    /// from the bottom of the first line to the top of the following one)
    pub line_gap: f32,
    /// Underline metrics
    pub uline: ScaledLineMetrics,
    /// Strikeout metrics
    pub strikeout: ScaledLineMetrics,
}

impl ScaledMetrics {
    /// The total height of the face, from ascent to descent
    pub fn height(&self) -> f32 {
        self.ascent - self.descent
    }

    pub(crate) const fn null() -> ScaledMetrics {
        ScaledMetrics {
            scale: 1.0,
            em_size: 0.0,
            ascent: 0.0,
            descent: 0.0,
            x_height: 0.0,
            cap_height: 0.0,
            line_gap: 0.0,
            uline: ScaledLineMetrics { position: 0.0, thickness: 0.0 },
            strikeout: ScaledLineMetrics { position: 0.0, thickness: 0.0 },
        }
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
