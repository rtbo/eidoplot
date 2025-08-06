pub mod axis;
pub mod backend;
pub mod data;
pub mod figure;
pub mod geom;
pub mod plots;
pub mod prelude;
pub mod render;
pub mod style;
pub mod text;

/// Module containing missing configuration values
/// Basically we put here all magic values that would require proper parameters
mod missing_params {
    use crate::{
        geom,
        style::{Color, color},
    };

    pub const FIG_TITLE_MARGIN: f32 = 6.0;
    pub const FIG_TITLE_COLOR: Color = color::BLACK;
    pub const TICK_SIZE: f32 = 4.0;
    pub const TICK_LABEL_MARGIN: f32 = 4.0;
    pub const AXIS_PADDING: geom::Padding = geom::Padding::Custom {
        t: 0.0,
        r: 0.0,
        b: 30.0,
        l: 50.0,
    };
}
