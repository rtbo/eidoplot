/*!
 * This module deals with colors and style of data series.
 */
use crate::style::{self, defaults};
use crate::style::ColorU8;

/// A series color palette
pub trait Palette {
    /// Number of colors in the palette
    fn len(&self) -> usize;
    /// Get a color from the palette
    fn get(&self, color: IndexColor) -> ColorU8;
}

/// A series color identified by its index in a palette
#[derive(Debug, Clone, Copy)]
pub struct IndexColor(pub usize);

impl style::Color for IndexColor {}

impl<P> style::ResolveColor<IndexColor> for P
where
    P: Palette,
{
    fn resolve_color(&self, color: &IndexColor) -> ColorU8 {
        self.get(*color)
    }
}

/// A series color that is automatically chosen from a palette based on the series index
#[derive(Debug, Clone, Copy)]
pub struct AutoColor;

/// Resolve automatically series color using a palette and a series index
impl<P> style::ResolveColor<AutoColor> for (&P, usize)
where
    P: Palette,
{
    fn resolve_color(&self, _color: &AutoColor) -> ColorU8 {
        self.0.get(IndexColor(self.1))
    }
}

impl style::Color for AutoColor {}

/// A flexible color for data series
#[derive(Debug, Clone, Copy, Default)]
pub enum Color {
    #[default]
    Auto,
    Index(IndexColor),
    Fixed(ColorU8),
}

impl From<IndexColor> for Color {
    fn from(color: IndexColor) -> Self {
        Color::Index(color)
    }
}

impl From<AutoColor> for Color {
    fn from(_color: AutoColor) -> Self {
        Color::Auto
    }
}

impl From<ColorU8> for Color {
    fn from(color: ColorU8) -> Self {
        Color::Fixed(color)
    }
}

impl style::Color for Color {}

/// Resolve a series color using a palette and a series index for automatic colors
impl<P> style::ResolveColor<Color> for (&P, usize)
where
    P: Palette,
{
    fn resolve_color(&self, _color: &Color) -> ColorU8 {
        match _color {
            Color::Index(c) => self.0.get(*c),
            Color::Auto => self.0.get(IndexColor(self.1)),
            Color::Fixed(c) => *c,
        }
    }
}

pub type Line = style::Line<Color>;

impl From<ColorU8> for Line {
    fn from(color: ColorU8) -> Self {
        Line {
            color: color.into(),
            width: defaults::SERIES_LINE_WIDTH,
            pattern: style::LinePattern::Solid,
            opacity: None,
        }
    }
}

pub type Fill = style::Fill<Color>;

impl From<ColorU8> for Fill {
    fn from(color: ColorU8) -> Self {
        Fill::Solid {
            color: color.into(),
            opacity: None,
        }
    }
}

pub type Marker = style::Marker<Color>;

impl From<ColorU8> for Marker {
    fn from(color: ColorU8) -> Self {
        Marker {
            size: Default::default(),
            shape: Default::default(),
            fill: Some(Fill::Solid {
                color: color.into(),
                opacity: None,
            }),
            stroke: None,
        }
    }
}

impl Palette for &[ColorU8] {
    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn get(&self, color: IndexColor) -> ColorU8 {
        self[color.0 % self.len()]
    }
}

/// A Palette for monochrome black plotting
/// Don't use with a dark theme.
pub const BLACK: &[ColorU8] = &[ColorU8::from_html(b"#000000")];

/// The standard eidoplot color palette (10 colors)
pub const STANDARD: &[ColorU8] = &[
    ColorU8::from_html(b"#1f77b4"), // blue
    ColorU8::from_html(b"#ff7f0e"), // orange
    ColorU8::from_html(b"#2ca02c"), // green
    ColorU8::from_html(b"#d62728"), // red
    ColorU8::from_html(b"#9467bd"), // purple
    ColorU8::from_html(b"#8c564b"), // brown
    ColorU8::from_html(b"#e377c2"), // pink
    ColorU8::from_html(b"#7f7f7f"), // gray
    ColorU8::from_html(b"#bcbd22"), // olive
    ColorU8::from_html(b"#17becf"), // cyan
];

/// The pastel eidoplot color palette (10 colors)
pub const PASTEL: &[ColorU8] = &[
    ColorU8::from_html(b"#aec7e8"), // light blue
    ColorU8::from_html(b"#ffbb78"), // light orange
    ColorU8::from_html(b"#98df8a"), // light green
    ColorU8::from_html(b"#ff9896"), // light red
    ColorU8::from_html(b"#c5b0d5"), // light purple
    ColorU8::from_html(b"#c49c94"), // light brown
    ColorU8::from_html(b"#f7b6d2"), // light pink
    ColorU8::from_html(b"#c7c7c7"), // light gray
    ColorU8::from_html(b"#dbdb8d"), // light olive
    ColorU8::from_html(b"#9edae5"), // light cyan
];

/// Paul Tol's 7-color colorblind-safe palette
pub const TOL_BRIGHT: &[ColorU8] = &[
    ColorU8::from_html(b"#4477AA"), // blue
    ColorU8::from_html(b"#EE6677"), // red
    ColorU8::from_html(b"#228833"), // green
    ColorU8::from_html(b"#CCBB44"), // yellow
    ColorU8::from_html(b"#66CCEE"), // cyan
    ColorU8::from_html(b"#AA3377"), // purple
    ColorU8::from_html(b"#BBBBBB"), // gray
];

/// Okabe & Ito colorblind-safe palette (8 colors)
pub const OKABE_ITO: &[ColorU8] = &[
    ColorU8::from_html(b"#E69F00"), // orange
    ColorU8::from_html(b"#56B4E9"), // sky blue
    ColorU8::from_html(b"#009E73"), // bluish green
    ColorU8::from_html(b"#F0E442"), // yellow
    ColorU8::from_html(b"#0072B2"), // blue
    ColorU8::from_html(b"#D55E00"), // vermillion
    ColorU8::from_html(b"#CC79A7"), // reddish purple
];
