use crate::style::{Color, color};
use crate::{geom, style};

pub const FONT_FAMILY: &str = "sans-serif";
pub const DASH_PATTERN: style::Dash = style::Dash(5.0, 5.0);

pub const FIG_SIZE: geom::Size = geom::Size::new(800.0, 600.0);
pub const FIG_PADDING: geom::Padding = geom::Padding::Even(20.0);

pub const TITLE_FONT_FAMILY: &str = FONT_FAMILY;
pub const TITLE_FONT_SIZE: f32 = 24.0;

pub const AXIS_LABEL_FONT_FAMILY: &str = FONT_FAMILY;
pub const AXIS_LABEL_FONT_SIZE: f32 = 16.0;
pub const AXIS_LABEL_COLOR: Color = color::BLACK;

pub const TICKS_LABEL_FONT_FAMILY: &str = FONT_FAMILY;
pub const TICKS_LABEL_FONT_SIZE: f32 = 12.0;
pub const TICKS_LABEL_COLOR: Color = color::BLACK;
pub const TICKS_GRID_LINE: Option<style::Line> = Some({
    style::Line {
        width: 1.0,
        color: color::GRAY,
        pattern: style::LinePattern::Dash(DASH_PATTERN),
    }
});
pub const MINOR_TICKS_GRID_LINE: Option<style::Line> = Some({
    style::Line {
        width: 0.5,
        color: color::GRAY,
        pattern: style::LinePattern::Dash(DASH_PATTERN),
    }
});

pub const MARKER_SIZE: f32 = 10.0;

pub const LEGEND_LABEL_FONT_FAMILY: &str = "Noto Sans Math";
pub const LEGEND_LABEL_FONT_SIZE: f32 = 16.0;
pub const LEGEND_LABEL_COLOR: Color = color::BLACK;
pub const LEGEND_BORDER: Option<style::Line> = Some({
    style::Line {
        width: 1.0,
        color: color::GRAY,
        pattern: style::LinePattern::Solid,
    }
});
pub const LEGEND_FILL: Option<style::Fill> =
    Some(style::Fill::Solid(color::WHITE.with_opacity(0.7)));
pub const LEGEND_SHAPE_SPACING: f32 = 10.0;
pub const LEGEND_SHAPE_SIZE: geom::Size = geom::Size::new(25.0, 14.0);
pub const LEGEND_PADDING: f32 = 8.0;
pub const LEGEND_SPACING: f32 = 16.0;
pub const LEGEND_MARGIN: f32 = 12.0;

pub const PLOT_XY_AUTO_INSETS: geom::Padding = geom::Padding::Even(15.0);
pub const PLOT_HIST_AUTO_INSETS: geom::Padding = geom::Padding::Custom {
    t: 15.0,
    r: 15.0,
    b: 0.0,
    l: 15.0,
};
