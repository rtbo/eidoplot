use std::str;

pub const DEFAULT_FONT_FAMILY: &str = "'Open Sans','Noto Sans',sans-serif";

#[derive(Debug, Clone)]
pub struct FontFamily(pub String);

impl FontFamily {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for FontFamily {
    fn default() -> Self {
        FontFamily("sans-serif".into())
    }
}

impl<S> From<S> for FontFamily
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        FontFamily(value.into())
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    family: FontFamily,
    size: f32,
}

impl Font {
    pub fn new(family: FontFamily, size: f32) -> Self {
        Font { family, size }
    }

    pub fn family(&self) -> &FontFamily {
        &self.family
    }

    pub fn size(&self) -> f32 {
        self.size
    }
}

impl Default for Font {
    fn default() -> Self {
        Font {
            family: FontFamily::default(),
            size: 24.0,
        }
    }
}

impl From<FontFamily> for Font {
    fn from(value: FontFamily) -> Self {
        Font {
            family: value,
            ..Font::default()
        }
    }
}

impl From<(FontFamily, f32)> for Font {
    fn from(value: (FontFamily, f32)) -> Self {
        Font {
            family: value.0,
            size: value.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    text: String,
    font: Option<Font>,
}

impl Text {
    pub fn new<S, F>(text: S, font: F) -> Self
    where
        S: Into<String>,
        F: Into<Font>,
    {
        Text {
            text: text.into(),
            font: Some(font.into()),
        }
    }

    pub fn from_str(text: &str) -> Self {
        Text {
            text: text.to_string(),
            font: None,
        }
    }

    pub fn with_font<F>(self, font: F) -> Self
    where
        F: Into<Font>,
    {
        Text {
            text: self.text,
            font: Some(font.into()),
        }
    }

    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    pub fn font(&self) -> Option<&Font> {
        self.font.as_ref()
    }
}

impl<S> From<S> for Text
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Text {
            text: value.into(),
            font: None,
        }
    }
}
