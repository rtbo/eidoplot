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
        Family(value.into())
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    family: Family,
    size: f32,
}

impl Font {
    pub fn new(family: Family, size: f32) -> Self {
        Font { family, size }
    }

    pub fn with_size(self, size: f32) -> Self {
        Font { size, ..self }
    }

    pub fn family(&self) -> &Family {
        &self.family
    }

    pub fn size(&self) -> f32 {
        self.size
    }
}

impl Default for Font {
    fn default() -> Self {
        Font {
            family: Family::default(),
            size: 24.0,
        }
    }
}

impl From<Family> for Font {
    fn from(value: Family) -> Self {
        Font {
            family: value,
            ..Font::default()
        }
    }
}

impl From<(Family, f32)> for Font {
    fn from(value: (Family, f32)) -> Self {
        Font {
            family: value.0,
            size: value.1,
        }
    }
}

impl From<f32> for Font {
    fn from(value: f32) -> Self {
        Font {
            size: value,
            ..Font::default()
        }
    }
}

fn is_valid_font_family(input: &str) -> bool {
    let parts = input.split(',').map(|s| s.trim());

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            return false;
        }

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
                    // If it contains whitespace, it should be quoted
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_font_family() {
        assert!(is_valid_font_family("'Noto Sans', 'Open Sans', sans-serif"));
        assert!(is_valid_font_family("Arial, sans-serif"));
        assert!(is_valid_font_family("'Times New Roman', serif"));
        assert!(!is_valid_font_family("'Noto Sans', Open Sans, sans-serif")); // Open Sans should be quoted
        assert!(!is_valid_font_family("Arial, sans-serif, ")); // Trailing comma with empty part
        assert!(!is_valid_font_family("'', sans-serif")); // Empty quoted string
    }
}
