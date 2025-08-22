use std::num::NonZeroU32;

use crate::style;
use crate::style::defaults;

#[derive(Debug, Default, Clone, Copy)]
pub enum Pos {
    OutTop,
    OutRight,
    #[default]
    OutBottom,
    OutLeft,
    InTop,
    InTopRight,
    InRight,
    InBottomRight,
    InBottom,
    InBottomLeft,
    InLeft,
    InTopLeft,
}

impl Pos {
    pub fn is_inside(&self) -> bool {
        matches!(
            self,
            Pos::InTop
                | Pos::InTopRight
                | Pos::InRight
                | Pos::InBottomRight
                | Pos::InBottom
                | Pos::InBottomLeft
                | Pos::InLeft
                | Pos::InTopLeft
        )
    }

    pub fn prefers_vertical(&self) -> bool {
        self.is_inside() || matches!(self, Pos::OutLeft | Pos::OutRight)
    }
}

#[derive(Debug, Clone)]
pub struct EntryFont {
    pub size: f32,
    pub font: style::Font,
    pub color: style::Color,
}

impl Default for EntryFont {
    fn default() -> Self {
        Self {
            size: defaults::LEGEND_LABEL_FONT_SIZE,
            font: style::Font::default(),
            color: defaults::LEGEND_LABEL_COLOR,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Legend {
    pos: Pos,
    font: EntryFont,
    fill: Option<style::Fill>,
    border: Option<style::Line>,
    label_fill: style::Fill,
    columns: Option<NonZeroU32>,
    padding: f32,
    spacing: f32,
    margin: f32,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            pos: Pos::OutBottom,
            font: EntryFont::default(),
            fill: defaults::LEGEND_FILL,
            border: defaults::LEGEND_BORDER,
            label_fill: defaults::LEGEND_LABEL_COLOR.into(),
            columns: None,
            padding: defaults::LEGEND_PADDING,
            spacing: defaults::LEGEND_SPACING,
            margin: defaults::LEGEND_MARGIN,
        }
    }
}

impl Legend {
    pub fn new(pos: Pos) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }

    pub fn font(&self) -> &EntryFont {
        &self.font
    }

    pub fn fill(&self) -> Option<&style::Fill> {
        self.fill.as_ref()
    }

    pub fn border(&self) -> Option<&style::Line> {
        self.border.as_ref()
    }

    pub fn label_fill(&self) -> &style::Fill {
        &self.label_fill
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

    /// Margin separating the legend from the plot area
    pub fn margin(&self) -> f32 {
        self.margin
    }

    pub fn with_pos(self, pos: Pos) -> Self {
        Self { pos, ..self }
    }

    pub fn with_font(self, font: EntryFont) -> Self {
        Self { font, ..self }
    }

    pub fn with_fill(self, fill: Option<style::Fill>) -> Self {
        Self { fill, ..self }
    }

    pub fn with_border(self, border: Option<style::Line>) -> Self {
        Self { border, ..self }
    }

    pub fn with_label_fill(self, label_fill: style::Fill) -> Self {
        Self { label_fill, ..self }
    }

    pub fn with_columns(self, columns: Option<NonZeroU32>) -> Self {
        Self { columns, ..self }
    }

    pub fn with_padding(self, padding: f32) -> Self {
        Self { padding, ..self }
    }

    pub fn with_spacing(self, spacing: f32) -> Self {
        Self { spacing, ..self }
    }

    /// Set the margin separating the legend from the plot area
    pub fn with_margin(self, margin: f32) -> Self {
        Self { margin, ..self }
    }
}
