//! Module containing missing configuration values
//! Basically we put here all magic values that would require proper parameters
use crate::{geom, style::{color, Color}, text::DEFAULT_FONT_FAMILY};

pub const FIG_TITLE_MARGIN: f32 = 6.0;
pub const FIG_TITLE_COLOR: Color = color::BLACK;
pub const TICK_SIZE: f32 = 4.0;
pub const TICK_COLOR: Color = color::BLACK;
pub const TICK_LABEL_FONT_FAMILY: &str = DEFAULT_FONT_FAMILY;
pub const TICK_LABEL_FONT_SIZE: f32 = 10.0;
pub const TICK_LABEL_MARGIN: f32 = 4.0;
pub const AXIS_PADDING: geom::Padding = geom::Padding::Custom {
    t: 0.0,
    r: 0.0,
    b: 30.0,
    l: 50.0,
};
