use std::str;

use crate::style::Font;

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
