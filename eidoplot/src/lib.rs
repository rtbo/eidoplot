pub mod data;
pub mod drawing;
pub mod geom;
pub mod ir;
pub mod render;
pub mod style;

use std::{path, sync::Arc};

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

fn resource_folder() -> path::PathBuf {
    path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("share")
}

/// Creates a new font database, loads fonts from the resource folder,
/// and returns an Arc to the database.
pub fn default_font_db() -> Arc<fontdb::Database> {
    let mut db = fontdb::Database::new();
    let res_dir = resource_folder();
    db.load_fonts_dir(&res_dir);
    Arc::new(db)
}
