use super::ColorU8;

#[derive(Debug, Clone, Copy)]
pub struct Color(pub usize);

pub trait Palette {
    fn len(&self) -> usize;
    fn get(&self, color: Color) -> ColorU8;
}

impl Palette for &[ColorU8] {
    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn get(&self, color: Color) -> ColorU8 {
        self[color.0 % self.len()]
    }
}

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
