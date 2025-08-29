pub mod catppuccin;
pub mod color;
pub(crate) mod defaults;
pub mod series;
pub mod theme;

pub use color::ColorU8;

pub mod font {
    pub use eidoplot_text::font::{Family, Font, Style, Weight, Width, parse_font_families};
}

pub use font::Font;
pub use series::Palette;
pub use theme::Theme;

use crate::render;

pub trait ResolveColor<Color> {
    fn resolve_color(&self, color: &Color) -> ColorU8;
}

pub trait Color {
    #[inline]
    fn resolve<R>(&self, rc: &R) -> ColorU8
    where
        R: ResolveColor<Self>,
        Self: Sized,
    {
        rc.resolve_color(self)
    }
}

impl Color for ColorU8 {}

impl ResolveColor<ColorU8> for () {
    fn resolve_color(&self, color: &ColorU8) -> ColorU8 {
        *color
    }
}

#[derive(Debug, Clone)]
pub struct Dash(Vec<f32>);

impl Default for Dash {
    fn default() -> Self {
        Dash(vec![5.0, 5.0])
    }
}

/// Line pattern defines how the line is drawn
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Line<C: Color> {
    pub color: C,
    pub width: f32,
    pub pattern: LinePattern,
    pub opacity: Option<f32>,
}

impl<C> Default for Line<C>
where
    C: Color + Default,
{
    fn default() -> Self {
        Line {
            color: C::default(),
            width: 1.0,
            pattern: LinePattern::default(),
            opacity: None,
        }
    }
}

const DOT_DASH: &[f32] = &[1.0, 1.0];

impl<C: Color> Line<C> {
    pub fn with_width(self, width: f32) -> Self {
        Line {
            width,
            ..self
        }
    }

    pub fn with_opacity(self, opacity: f32) -> Self {
        Line {
            opacity: Some(opacity),
            ..self
        }         
    }

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

impl<C: Color> From<f32> for Line<C>
where C: Color + Default {
    fn from(value: f32) -> Self {
        Line {
            width: value,
            color: C::default(),
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

#[derive(Debug, Clone, Copy)]
pub enum Fill<C: Color> {
    Solid { color: C, opacity: Option<f32> },
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
    pub fn with_opacity(self, opacity: f32) -> Self {
        match self {
            Fill::Solid { color, .. } => Fill::Solid {
                color,
                opacity: Some(opacity),
            },
        }
    }

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

#[derive(Debug, Clone, Copy, Default)]
pub enum MarkerShape {
    #[default]
    Circle,
    Square,
    Diamond,
    Cross,
    Plus,
    TriangleUp,
    TriangleDown,
}

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

#[derive(Debug, Clone)]
pub struct Marker<C: Color> {
    pub size: MarkerSize,
    pub shape: MarkerShape,
    pub fill: Option<Fill<C>>,
    pub stroke: Option<Line<C>>,
}

#[cfg(test)]
mod tests {
    use crate::style::theme;
    use theme::Theme;

    use super::*;

    #[test]
    fn test_color_resolve() {
        let theme = theme::Light::new(series::STANDARD);

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
