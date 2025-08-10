use std::fmt;

use crate::style::defaults;

#[derive(Debug, Clone)]
pub struct InvalidFamilyString(pub String);

impl fmt::Display for InvalidFamilyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid font family: \"{}\"", self.0)
    }
}

impl std::error::Error for InvalidFamilyString {}

#[derive(Debug, Clone)]
pub struct Family(String);

impl Family {
    pub fn new<S>(s: S) -> Result<Family, InvalidFamilyString>
    where
        S: Into<String>,
    {
        let s = s.into();
        if s.is_empty() || !is_valid_font_family(s.as_str()) {
            Err(InvalidFamilyString(s))
        } else {
            Ok(Family(s))
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for Family {
    fn default() -> Self {
        Family(defaults::FONT_FAMILY.into())
    }
}

impl<S> From<S> for Family
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Family::new(value.into()).expect("Invalid font family")
    }
}

/// Specifies the weight of glyphs in the font, their degree of blackness or stroke thickness.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct Weight(pub u16);

impl Default for Weight {
    #[inline]
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
    #[inline]
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
    #[inline]
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
}

impl Default for Width {
    #[inline]
    fn default() -> Self {
        Width::Normal
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    family: Family,
    size: f32,
    weight: Weight,
    width: Width,
    style: Style,
}

impl Font {
    pub fn new(family: Family, size: f32) -> Self {
        Font { family, size, weight: Weight::NORMAL, width: Width::Normal, style: Style::Normal }
    }

    pub fn with_family(self, family: Family) -> Self {
        Font { family, ..self }
    }

    pub fn with_size(self, size: f32) -> Self {
        Font { size, ..self }
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

    pub fn family(&self) -> &Family {
        &self.family
    }

    pub fn size(&self) -> f32 {
        self.size
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

impl From<(Family, f32)> for Font {
    fn from((family, size): (Family, f32)) -> Self {
        Font::new(family, size)
    }
}

fn is_valid_font_family(input: &str) -> bool {
    let parts = input.split(',').map(|s| s.trim());

    let mut num_parts = 0;
    let mut has_unquoted_ws = false;

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            return false;
        }

        num_parts += 1;

        match part {
            "serif" | "sans-serif" | "cursive" | "fantasy" | "monospace" => continue,
            _ => {
                // Check if it's a valid custom font family name
                if part.starts_with('\'') && part.ends_with('\'') {
                    // Check if there's at least one character between the quotes
                    if part.len() <= 2 {
                        return false;
                    }
                } else if part.starts_with('"') && part.ends_with('"') {
                    // Check if there's at least one character between the quotes
                    if part.len() <= 2 {
                        return false;
                    }
                } else if part.contains(char::is_whitespace) {
                    // It should be quoted if it contains whitespace and has more than one part
                    has_unquoted_ws = true;
                }
            }
        }
    }

    num_parts > 0 && (num_parts == 1 || !has_unquoted_ws)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_font_family() {
        assert!(is_valid_font_family("sans-serif"));
        assert!(is_valid_font_family("Noto Sans Math"));
        assert!(is_valid_font_family("'Noto Sans', 'Open Sans', sans-serif"));
        assert!(is_valid_font_family("Arial, sans-serif"));
        assert!(is_valid_font_family("'Times New Roman', serif"));
        assert!(!is_valid_font_family("'Noto Sans', Open Sans, sans-serif")); // Open Sans should be quoted
        assert!(!is_valid_font_family("Arial, sans-serif, ")); // Trailing comma with empty part
        assert!(!is_valid_font_family("'', sans-serif")); // Empty quoted string
    }
}
