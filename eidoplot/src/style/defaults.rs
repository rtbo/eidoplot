use crate::{geom, style};
use crate::style::{Color, color};

pub const FONT_FAMILY: &str = "sans-serif";
pub const DASH_PATTERN: style::Dash = style::Dash(5.0, 5.0);

pub const FIG_SIZE: geom::Size = geom::Size::new(800.0, 600.0);
pub const FIG_PADDING: geom::Padding = geom::Padding::Even(20.0);

pub const TITLE_FONT_FAMILY: &str = FONT_FAMILY;
pub const TITLE_FONT_SIZE: f32 = 24.0;

pub const TICKS_LABEL_FONT_SIZE: f32 = 12.0;
pub const TICKS_LABEL_COLOR: Color = color::BLACK;
pub const TICKS_GRID_LINE: Option<style::Line> = Some({
    style::Line {
        width: 1.0,
        color: color::GRAY,
        pattern: style::LinePattern::Dash(DASH_PATTERN),
    }
});

pub const PLOT_XY_AUTO_INSETS: (f32, f32) = (15.0, 15.0);
