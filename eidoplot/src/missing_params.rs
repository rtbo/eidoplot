//! Module containing missing configuration values
//! Basically we put here all magic values that would require proper parameters
use crate::geom;

pub const TICK_SIZE: f32 = 4.0;
pub const AXIS_PADDING: geom::Padding = geom::Padding::Custom {
    t: 0.0, r: 0.0, b: 30.0, l: 50.0,
};
