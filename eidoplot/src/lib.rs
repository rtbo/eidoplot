pub mod data;
pub mod drawing;
pub mod geom;
pub mod ir;
pub mod render;
pub mod style;

/// Module containing missing configuration values
/// Basically we put here all magic values that would require proper parameters
mod missing_params {
    use crate::{
        geom,
        style::{Color, color, defaults},
    };

    pub const FIG_TITLE_MARGIN: f32 = 6.0;
    pub const FIG_TITLE_COLOR: Color = color::BLACK;

    pub const PLOT_AXIS_PADDING: geom::Padding = geom::Padding::Custom {
        t: 0.0,
        r: 0.0,
        b: 30.0,
        l: 50.0,
    };
    pub const AXIS_LABEL_MARGIN: f32 = 4.0;
    pub const AXIS_LABEL_FONT_FAMILY: &str = defaults::FONT_FAMILY;
    pub const AXIS_LABEL_FONT_SIZE: f32 = 14.0;
    pub const AXIS_LABEL_COLOR: Color = color::BLACK;

    pub const TICK_SIZE: f32 = 4.0;
    pub const TICK_COLOR: Color = color::BLACK;
    pub const TICK_LABEL_MARGIN: f32 = 4.0;
}
