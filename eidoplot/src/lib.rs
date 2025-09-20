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
pub mod eplt;
pub mod geom;
pub mod ir;
pub mod render;
pub mod style;

/// Module containing missing configuration values
/// Basically we put here all magic values that would require proper parameters
mod missing_params {
    use crate::geom;

    pub const FIG_TITLE_MARGIN: f32 = 6.0;

    pub const PLOT_PADDING: geom::Padding = geom::Padding::Even(0.0);

    pub const AXIS_TITLE_MARGIN: f32 = 8.0;
    pub const AXIS_ANNOT_MARGIN: f32 = 4.0;

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

#[cfg(test)]
pub(crate) mod tests {
    pub trait Near {
        fn near_abs(&self, other: &Self, tol: f64) -> bool;
        fn near_rel(&self, other: &Self, err: f64) -> bool;
    }

    impl Near for f64 {
        fn near_abs(&self, other: &Self, tol: f64) -> bool {
            (self - other).abs() <= tol
        }

        fn near_rel(&self, other: &Self, err: f64) -> bool {
            let diff = (self - other).abs();
            let largest = self.abs().max(other.abs());
            diff <= largest * err
        }
    }

    impl Near for f32 {
        fn near_abs(&self, other: &Self, tol: f64) -> bool {
            (self - other).abs() as f64 <= tol
        }

        fn near_rel(&self, other: &Self, err: f64) -> bool {
            let diff = (self - other).abs() as f64;
            let largest = self.abs().max(other.abs()) as f64;
            diff <= largest * err
        }
    }

    macro_rules! assert_near {
        (abs, $a:expr, $b:expr, $tol:expr) => {
            assert!($a.near_abs(&$b, $tol), "Assertion failed: Values are not close enough.\nValue 1: {:?}\nValue 2: {:?}\nTolerance: {}", $a, $b, $tol);
        };
        (abs, $a:expr, $b:expr) => {
            assert_near!(abs, $a, $b, 1e-8);
        };
        (rel, $a:expr, $b:expr, $err:expr) => {
            assert!($a.near_rel(&$b, $err), "Assertion failed: Values are not close enough.\nValue 1: {:?}\nValue 2: {:?}\nRelative error: {}", $a, $b, $err);
        };
        (rel, $a:expr, $b:expr) => {
            assert_near!(rel, $a, $b, 1e-8);
        };
    }

    pub(crate) use assert_near;

    #[test]
    fn test_close_to() {
        let a = 1.0;
        let b = 1.0 + 1e-9;
        assert_near!(abs, a, b);
        assert!(!a.near_abs(&b, 1e-10));
        assert_near!(rel, a, b);
        assert!(!a.near_rel(&b, 1e-10));
    }
}
