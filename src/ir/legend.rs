use std::num::NonZeroU32;

use crate::geom::{Padding, Size};
use crate::style::{defaults, theme};
use crate::text;

#[derive(Debug, Clone)]
pub struct EntryFont {
    pub size: f32,
    pub font: text::Font,
    pub color: theme::Color,
}

impl Default for EntryFont {
    fn default() -> Self {
        Self {
            size: defaults::LEGEND_LABEL_FONT_SIZE,
            font: text::Font::default(),
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
    padding: Padding,
    spacing: Size,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            font: EntryFont::default(),
            fill: Some(theme::Col::LegendFill.into()),
            border: Some(theme::Col::LegendBorder.into()),
            columns: None,
            padding: defaults::LEGEND_PADDING.into(),
            spacing: Size::new(defaults::LEGEND_H_SPACING, defaults::LEGEND_V_SPACING),
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

    pub fn padding(&self) -> Padding {
        self.padding
    }

    pub fn spacing(&self) -> Size {
        self.spacing
    }

    pub fn with_font(self, font: impl Into<EntryFont>) -> Self {
        Self { font: font.into(), ..self }
    }

    pub fn with_fill(self, fill: impl Into<Option<theme::Fill>>) -> Self {
        Self { fill: fill.into(), ..self }
    }

    pub fn with_border(self, border: impl Into<Option<theme::Line>>) -> Self {
        Self { border: border.into(), ..self }
    }

    pub fn with_columns(self, columns: u32) -> Self {
        Self {
            columns: Some(NonZeroU32::new(columns).expect("columns > 0")),
            ..self
        }
    }

    pub fn with_padding(self, padding: impl Into<Padding>) -> Self {
        Self { padding: padding.into(), ..self }
    }

    pub fn with_spacing(self, spacing: impl Into<Size>) -> Self {
        Self { spacing: spacing.into(), ..self }
    }
}
