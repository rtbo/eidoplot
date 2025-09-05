use std::num::NonZeroU32;

use crate::style::{self, theme, defaults};

#[derive(Debug, Clone)]
pub struct EntryFont {
    pub size: f32,
    pub font: style::Font,
    pub color: theme::Color,
}

impl Default for EntryFont {
    fn default() -> Self {
        Self {
            size: defaults::LEGEND_LABEL_FONT_SIZE,
            font: style::Font::default(),
            color: theme::Col::Foreground.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Legend {
    font: EntryFont,
    fill: Option<theme::Fill>,
    border: Option<theme::Line>,
    columns: Option<NonZeroU32>,
    padding: f32,
    spacing: f32,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            font: EntryFont::default(),
            fill: Some(theme::Col::LegendFill.into()),
            border: Some(theme::Col::LegendBorder.into()),
            columns: None,
            padding: defaults::LEGEND_PADDING,
            spacing: defaults::LEGEND_SPACING,
        }
    }
}

impl Legend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn font(&self) -> &EntryFont {
        &self.font
    }

    pub fn fill(&self) -> Option<&theme::Fill> {
        self.fill.as_ref()
    }

    pub fn border(&self) -> Option<&theme::Line> {
        self.border.as_ref()
    }

    pub fn columns(&self) -> Option<NonZeroU32> {
        self.columns
    }

    pub fn padding(&self) -> f32 {
        self.padding
    }

    pub fn spacing(&self) -> f32 {
        self.spacing
    }

    pub fn with_font(self, font: EntryFont) -> Self {
        Self { font, ..self }
    }

    pub fn with_fill(self, fill: Option<theme::Fill>) -> Self {
        Self { fill, ..self }
    }

    pub fn with_border(self, border: Option<theme::Line>) -> Self {
        Self { border, ..self }
    }

    pub fn with_columns(self, columns: NonZeroU32) -> Self {
        Self { columns: Some(columns), ..self }
    }

    pub fn with_padding(self, padding: f32) -> Self {
        Self { padding, ..self }
    }

    pub fn with_spacing(self, spacing: f32) -> Self {
        Self { spacing, ..self }
    }
}
