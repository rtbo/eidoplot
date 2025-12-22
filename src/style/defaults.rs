use crate::geom;

pub const FONT_FAMILY: &str = "sans-serif";

pub const FIG_SIZE: geom::Size = geom::Size::new(800.0, 600.0);
pub const FIG_PADDING: geom::Padding = geom::Padding::Even(20.0);

pub const TITLE_FONT_SIZE: f32 = 20.0;
pub const AXIS_LABEL_FONT_SIZE: f32 = 16.0;
pub const TICKS_LABEL_FONT_SIZE: f32 = 12.0;

pub const SERIES_LINE_WIDTH: f32 = 1.5;
pub const MARKER_SIZE: f32 = 10.0;

pub const LEGEND_LABEL_FONT_SIZE: f32 = 13.0;
pub const LEGEND_SHAPE_SPACING: f32 = 10.0;
pub const LEGEND_SHAPE_SIZE: geom::Size = geom::Size::new(25.0, 14.0);
pub const LEGEND_PADDING: f32 = 8.0;
pub const LEGEND_H_SPACING: f32 = 16.0;
pub const LEGEND_V_SPACING: f32 = 10.0;
pub const LEGEND_MARGIN: f32 = 12.0;

pub const PLOT_XY_AUTO_INSETS: geom::Padding = geom::Padding::Even(20.0);
pub const PLOT_VER_BARS_AUTO_INSETS: geom::Padding = geom::Padding::Custom {
    t: 20.0,
    r: 20.0,
    b: 0.0,
    l: 20.0,
};
pub const PLOT_HOR_BARS_AUTO_INSETS: geom::Padding = geom::Padding::Custom {
    t: 20.0,
    r: 20.0,
    b: 20.0,
    l: 0.0,
};
pub const PLOT_AXIS_ARROW_SIZE: f32 = 10.0;
pub const PLOT_AXIS_ARROW_OVERFLOW: f32 = 10.0;
