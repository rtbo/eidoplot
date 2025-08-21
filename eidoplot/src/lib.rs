use std::path;

use eidoplot_text::fontdb;

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
        style::{Color, color},
    };

    pub const FIG_TITLE_MARGIN: f32 = 6.0;
    pub const FIG_TITLE_COLOR: Color = color::BLACK;

    pub const PLOT_PADDING: geom::Padding = geom::Padding::Even(5.0);
    pub const AXIS_LABEL_MARGIN: f32 = 8.0;
    pub const AXIS_LABEL_COLOR: Color = color::BLACK;
    pub const AXIS_ANNOT_FONT_FAMILY: &str = "Noto Sans Math";

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

/// Loads fonts that are bundled with eidoplot
/// and returns an Arc to the database.
pub fn bundled_font_db() -> fontdb::Database {
    const FONTDB_FAMILY_SANS: &str = "Noto Sans";
    const FONTDB_FAMILY_SERIF: &str = "Noto Serif";
    const FONTDB_FAMILY_MONO: &str = "Noto Mono";

    let res_dir = crate::resource_folder();

    let mut db = fontdb::Database::new();

    db.load_fonts_dir(&res_dir);
    db.set_sans_serif_family(FONTDB_FAMILY_SANS);
    db.set_serif_family(FONTDB_FAMILY_SERIF);
    db.set_monospace_family(FONTDB_FAMILY_MONO);

    db
}