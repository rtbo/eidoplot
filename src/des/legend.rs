//! Legend configuration for a plot.
//!! A legend is a box containing entries describing the different series
//! displayed in a plot.
use std::num::NonZeroU32;

use crate::geom::{Padding, Size};
use crate::style::{defaults, theme};
use crate::text;

/// The font configuration for legend entries
#[derive(Debug, Clone)]
pub struct EntryFont {
    /// The font size in figure units
    pub size: f32,
    /// The font
    pub font: text::Font,
    /// The font color
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

/// Legend configuration for a plot
#[derive(Debug, Clone)]
pub struct Legend<Pos> {
    pos: Pos,
    font: EntryFont,
    fill: Option<theme::Fill>,
    border: Option<theme::Stroke>,
    columns: Option<NonZeroU32>,
    padding: Padding,
    margin: f32,
    spacing: Size,
}

impl<Pos: Default> Default for Legend<Pos> {
    /// Create a default legend configuration
    /// - Fill color: theme::Col::LegendFill
    /// - Border: theme::Col::LegendBorder, 1.0
    /// - Font: default EntryFont
    /// - Default column layout (depdend on the position and number and width of entries)
    /// - Default padding and spacing
    fn default() -> Self {
        Self {
            pos: Pos::default(),
            font: EntryFont::default(),
            fill: Some(theme::Col::LegendFill.into()),
            border: Some(theme::Col::LegendBorder.into()),
            columns: None,
            padding: defaults::LEGEND_PADDING.into(),
            margin: defaults::LEGEND_MARGIN,
            spacing: Size::new(defaults::LEGEND_H_SPACING, defaults::LEGEND_V_SPACING),
        }
    }
}

impl<Pos> Legend<Pos>
where
    Pos: Default,
{
    /// Create a new legend at the specified position with default properties
    pub fn new(pos: Pos) -> Self {
        Self {
            pos,
            ..Self::default()
        }
    }
}

impl<Pos> Legend<Pos>
where
    Pos: Copy,
{
    /// Get the position of the legend
    pub fn pos(&self) -> Pos {
        self.pos
    }
}

impl<Pos> Legend<Pos> {
    /// Get the font configuration for legend entries
    pub fn font(&self) -> &EntryFont {
        &self.font
    }

    /// Get the fill style for the legend background
    pub fn fill(&self) -> Option<&theme::Fill> {
        self.fill.as_ref()
    }

    /// Get the border style for the legend box
    pub fn border(&self) -> Option<&theme::Stroke> {
        self.border.as_ref()
    }

    /// Get the number of columns for the legend entries
    pub fn columns(&self) -> Option<u32> {
        self.columns.map(|c| c.get())
    }

    /// Get the spacing between legend entries
    pub fn spacing(&self) -> Size {
        self.spacing
    }

    /// Get the padding inside the legend box
    pub fn padding(&self) -> Padding {
        self.padding
    }

    /// Get the margin around the legend box
    pub fn margin(&self) -> f32 {
        self.margin
    }

    /// Set the position of the legend and return self for chaining
    pub fn with_pos(self, pos: Pos) -> Self {
        Self { pos, ..self }
    }

    /// Set the font configuration for legend entries and return self for chaining
    pub fn with_font(self, font: EntryFont) -> Self {
        Self { font, ..self }
    }

    /// Set the fill style for the legend background and return self for chaining
    pub fn with_fill(self, fill: Option<theme::Fill>) -> Self {
        Self { fill, ..self }
    }

    /// Set the border style for the legend box and return self for chaining
    pub fn with_border(self, border: Option<theme::Stroke>) -> Self {
        Self { border, ..self }
    }

    /// Set the number of columns for the legend entries and return self for chaining
    pub fn with_columns(self, columns: u32) -> Self {
        Self {
            columns: Some(NonZeroU32::new(columns).expect("columns > 0")),
            ..self
        }
    }

    /// Set the spacing between legend entries and return self for chaining
    pub fn with_spacing(self, spacing: Size) -> Self {
        Self { spacing, ..self }
    }

    /// Set the padding inside the legend box and return self for chaining
    pub fn with_padding(self, padding: Padding) -> Self {
        Self { padding, ..self }
    }

    /// Set the margin around the legend box and return self for chaining
    pub fn with_margin(self, margin: f32) -> Self {
        Self { margin, ..self }
    }
}
