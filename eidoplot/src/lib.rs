//#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
/*!
 * # eidoplot
 * A simple and minimal data plotting library for Rust
 * 
 * Eidoplot separates figure design from data representation and from rendering surfaces.
 * It aims to be the LATEX of data plotting.
 *
 * ## Supported figure types
 *  - XY line plots
 *  - Histograms
 * 
 * ## Notes about eidoplot's design
 * 
 * The figure design lies in the `ir` module (IR = Intermediate Representation).
 * The IR describes the figure design in a declarative way. It ignores everything about the rendering surfaces.
 * A yet to be designed DSL for figure will be provided to describe the IR in text files that can be parsed by eidoplot.
 * This will allow to bridge easily eidoplot to other programming languages and to write a compiler for eidoplot figures.
 * 
 * The rendering surfaces implements the `render::Surface` trait and are in separate crates.
 * (see `eidoplot-pxl` and `eidoplot-svg`)
 * The rendering surfaces themselves ignore everything about figure design and the `ir` module.
 * They focus on rendering primitives, like lines and text.
 * 
 * IR and rendering are bridged together by the `drawing` module, which exposes very little public API.
 */
// Eidoplot is released under the MIT License with the following copyright:
// Copyright (c) 2025 Rémi Thebault

use std::path;

use eidoplot_text::fontdb;

pub mod data;
pub mod drawing;
pub mod geom;
pub mod ir;
pub mod parse;
pub mod render;
pub mod style;

pub use parse::{parse_eplt};

/// Module containing missing configuration values
/// Basically we put here all magic values that would require proper parameters
mod missing_params {
    use crate::geom;

    pub const FIG_TITLE_MARGIN: f32 = 6.0;

    pub const PLOT_PADDING: geom::Padding = geom::Padding::Even(5.0);
    pub const AXIS_TITLE_MARGIN: f32 = 8.0;

    pub const TICK_SIZE: f32 = 4.0;
    pub const TICK_LABEL_MARGIN: f32 = 4.0;
    pub const MINOR_TICK_LINE_WIDTH: f32 = 0.5;
    pub const MINOR_TICK_SIZE: f32 = 2.0;
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
