//! Style definitions for lines, fills, markers, and themes.
pub mod catppuccin;
pub(crate) mod defaults;
pub mod series;
pub mod theme;

pub use series::Palette;
pub use theme::Theme;

use crate::{Color, ResolveColor, render};

/// Dash pattern for dashed lines
/// A dash pattern is a sequence of lengths that specify the lengths of
/// alternating dashes and gaps.
///
/// The lengths are relative to the line width.
/// So a pattern will scale with the line width and remain visually consistent.
#[derive(Debug, Clone, PartialEq)]
pub struct Dash(pub Vec<f32>);

impl Default for Dash {
    fn default() -> Self {
        Dash(vec![5.0, 5.0])
    }
}

/// Line pattern defines how the line is drawn
#[derive(Debug, Clone, PartialEq)]
pub enum LinePattern {
    /// Solid line
    Solid,
    /// Dashed line. The pattern is relative to the line width.
    Dash(Dash),
    /// Dotted line. Equivalent to Dash(1.0, 1.0)
    Dot,
}

impl Default for LinePattern {
    fn default() -> Self {
        LinePattern::Solid
    }
}

impl From<Dash> for LinePattern {
    fn from(dash: Dash) -> Self {
        LinePattern::Dash(dash)
    }
}

/// Line style definition
///
/// The color is a generic parameter to support different color resolution strategies,
/// such as fixed colors, theme-based colors, or series-based colors.
#[derive(Debug, Clone, PartialEq)]
pub struct Line<C: Color> {
    /// Line color
    pub color: C,
    /// Line width in figure units
    pub width: f32,
    /// Line pattern
    pub pattern: LinePattern,
    /// Line opacity (0.0 to 1.0)
    pub opacity: Option<f32>,
}

const DOT_DASH: &[f32] = &[1.0, 1.0];

impl<C: Color> Line<C> {
    /// Set the line width in figure units, returning self for chaining
    pub fn with_width(self, width: f32) -> Self {
        Line { width, ..self }
    }

    /// Set the line opacity (0.0 to 1.0), returning self for chaining
    pub fn with_opacity(self, opacity: f32) -> Self {
        Line {
            opacity: Some(opacity),
            ..self
        }
    }

    /// Set the line pattern, returning self for chaining
    pub fn with_pattern(self, pattern: LinePattern) -> Self {
        Line { pattern, ..self }
    }

    /// Convert to a renderable stroke, resolving colors using the provided resolver
    pub fn as_stroke<'a, R>(&'a self, rc: &R) -> render::Stroke<'a>
    where
        R: ResolveColor<C>,
    {
        let color = if let Some(opacity) = self.opacity {
            self.color.resolve(rc).with_opacity(opacity)
        } else {
            self.color.resolve(rc)
        };

        let pattern = match &self.pattern {
            LinePattern::Solid => render::LinePattern::Solid,
            LinePattern::Dash(Dash(a)) => render::LinePattern::Dash(a.as_slice()),
            LinePattern::Dot => render::LinePattern::Dash(DOT_DASH),
        };

        render::Stroke {
            color,
            width: self.width,
            pattern,
        }
    }
}

impl<C: Color> From<C> for Line<C> {
    fn from(color: C) -> Self {
        Line {
            width: 1.0,
            color,
            pattern: LinePattern::default(),
            opacity: None,
        }
    }
}

impl<C: Color> From<(C, f32)> for Line<C> {
    fn from((color, width): (C, f32)) -> Self {
        Line {
            color,
            width,
            pattern: LinePattern::default(),
            opacity: None,
        }
    }
}

impl<C: Color> From<(C, f32, LinePattern)> for Line<C> {
    fn from((color, width, pattern): (C, f32, LinePattern)) -> Self {
        Line {
            color,
            width,
            pattern,
            opacity: None,
        }
    }
}

impl<C: Color> From<(C, f32, Dash)> for Line<C> {
    fn from((color, width, dash): (C, f32, Dash)) -> Self {
        Line {
            color,
            width,
            pattern: LinePattern::Dash(dash),
            opacity: None,
        }
    }
}

/// Fill style definition
/// The color is a generic parameter to support different color resolution strategies,
/// such as fixed colors, theme based colors, or series-based colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill<C: Color> {
    /// Solid fill
    Solid {
        /// Fill color
        color: C,
        /// Fill opacity (0.0 to 1.0)
        opacity: Option<f32>,
    },
}

impl<C> Default for Fill<C>
where
    C: Color + Default,
{
    fn default() -> Self {
        Fill::Solid {
            color: C::default(),
            opacity: None,
        }
    }
}

impl<C: Color> Fill<C> {
    /// Set the fill opacity (0.0 to 1.0), returning self for chaining
    pub fn with_opacity(self, opacity: f32) -> Self {
        match self {
            Fill::Solid { color, .. } => Fill::Solid {
                color,
                opacity: Some(opacity),
            },
        }
    }

    /// Convert to a renderable paint, resolving colors using the provided resolver
    pub fn as_paint<R>(&self, rc: &R) -> render::Paint
    where
        R: ResolveColor<C>,
    {
        match self {
            Fill::Solid {
                color,
                opacity: None,
            } => render::Paint::Solid(color.resolve(rc)),
            Fill::Solid {
                color,
                opacity: Some(opacity),
            } => render::Paint::Solid(color.resolve(rc).with_opacity(*opacity)),
        }
    }
}

impl<C: Color> From<C> for Fill<C> {
    fn from(color: C) -> Self {
        Fill::Solid {
            color,
            opacity: None,
        }
    }
}

/// Shape of a marker, used in scatter plots
#[derive(Debug, Clone, Copy, Default)]
pub enum MarkerShape {
    /// Circle marker (the default)
    #[default]
    Circle,
    /// Square marker
    Square,
    ///  Diamond marker
    Diamond,
    ///  Cross marker
    Cross,
    ///  Plus marker
    Plus,
    ///  Upward pointing triangle marker
    TriangleUp,
    ///  Downward pointing triangle marker
    TriangleDown,
}

/// Size of a marker, used in scatter plots
#[derive(Debug, Clone, Copy)]
pub struct MarkerSize(pub f32);

impl Default for MarkerSize {
    fn default() -> Self {
        MarkerSize(defaults::MARKER_SIZE)
    }
}

impl From<f32> for MarkerSize {
    fn from(size: f32) -> Self {
        MarkerSize(size)
    }
}

/// Marker style definition, used in scatter plots
#[derive(Debug, Clone)]
pub struct Marker<C: Color> {
    /// Marker size
    pub size: MarkerSize,
    /// Marker shape
    pub shape: MarkerShape,
    /// Marker fill style
    pub fill: Option<Fill<C>>,
    /// Marker stroke style
    pub stroke: Option<Line<C>>,
}

impl<C> Default for Marker<C>
where
    C: Color + Default,
{
    fn default() -> Self {
        Marker {
            size: MarkerSize::default(),
            shape: MarkerShape::default(),
            fill: Some(Fill::default()),
            stroke: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorU8;
    use crate::style::theme;

    #[test]
    fn test_color_resolve() {
        let theme = theme::light(series::palettes::standard());

        let theme_line: theme::Line = (theme::Color::Theme(theme::Col::LegendBorder), 2.0).into();
        let stroke = theme_line.as_stroke(&theme);
        assert_eq!(stroke.color, ColorU8::from_html(b"#000000"));

        let series_line: Line<series::IndexColor> = (series::IndexColor(2), 2.0).into();
        let stroke = series_line.as_stroke(theme.palette());
        assert_eq!(stroke.color, ColorU8::from_html(b"#2ca02c"));

        let series_line: Line<series::AutoColor> = (series::AutoColor, 2.0).into();
        let stroke = series_line.as_stroke(&(theme.palette(), 2));
        assert_eq!(stroke.color, ColorU8::from_html(b"#2ca02c"));

        let fixed_color: Line<ColorU8> = (ColorU8::from_html(b"#123456"), 2.0).into();
        let stroke = fixed_color.as_stroke(&());
        assert_eq!(stroke.color, ColorU8::from_html(b"#123456"));
    }
}
