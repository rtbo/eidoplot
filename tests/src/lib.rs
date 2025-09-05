#![cfg(test)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use drawing::SurfaceExt;
use eidoplot::{drawing, ir, style};
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;

mod pixelmatch;
mod tests;

fn ref_file_path(file: &str) -> PathBuf {
    let tests_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(tests_dir).join("refs").join(file)
}

fn actual_file_path(file: &str) -> PathBuf {
    let tests_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(tests_dir).join("actual").join(file)
}

fn diff_file_path(file: &str) -> PathBuf {
    let tests_dir = env!("CARGO_MANIFEST_DIR");

    let path = Path::new(file);
    let parent = path.parent();
    let stem = path.file_stem().and_then(std::ffi::OsStr::to_str).unwrap();
    let ext = path.extension().and_then(std::ffi::OsStr::to_str).unwrap();
    let filename = format!("{}-diff.{}", stem, ext);
    let file = parent
        .map(|p| p.join(&filename))
        .unwrap_or_else(|| PathBuf::from(filename));

    Path::new(tests_dir).join("actual").join(file)
}

fn bw_theme() -> impl style::Theme {
    style::theme::Light::new(style::series::BLACK)
}

#[cfg(feature = "regenerate-refs")]
const REGENERATE_REFS: bool = true;

#[cfg(not(feature = "regenerate-refs"))]
const REGENERATE_REFS: bool = false;

struct PreparedAssertion<F> {
    actual_drawn_fig: F,
    ref_drawn_fig: F,
}

trait TestHarness {
    type DrawnFig;

    fn draw<T>(fig: &ir::Figure, theme: T) -> Self::DrawnFig
    where
        T: style::Theme;

    fn regenerate_refs() -> bool;

    fn serialize_fig(file: &Path, fig: &Self::DrawnFig);

    fn deserialize_fig(file: &Path) -> Self::DrawnFig;

    fn prepare_assertion<T>(
        ref_file_name: &str,
        fig: &ir::Figure,
        theme: T,
    ) -> PreparedAssertion<Self::DrawnFig>
    where
        T: style::Theme,
    {
        let ref_file = ref_file_path(&ref_file_name);
        let actual_file = actual_file_path(&ref_file_name);

        let actual_drawn_fig = Self::draw(fig, theme);

        if Self::regenerate_refs() {
            std::fs::create_dir_all(ref_file.parent().unwrap()).unwrap();
            Self::serialize_fig(&ref_file, &actual_drawn_fig);
        }

        if !std::fs::exists(&ref_file).unwrap() {
            std::fs::create_dir_all(actual_file.parent().unwrap()).unwrap();
            Self::serialize_fig(actual_file.as_path(), &actual_drawn_fig);
            panic!(
                "No such ref: \"{}\"\n  Actual figure written to {}",
                &ref_file_name,
                actual_file.display()
            );
        }

        let ref_drawn_fig = Self::deserialize_fig(&ref_file);

        PreparedAssertion {
            actual_drawn_fig,
            ref_drawn_fig,
        }
    }
}

struct PxlHarness;

impl TestHarness for PxlHarness {
    type DrawnFig = tiny_skia::Pixmap;

    fn draw<T>(fig: &ir::Figure, theme: T) -> Self::DrawnFig
    where
        T: style::Theme,
    {
        let size = fig.size();
        let fontdb = Arc::new(eidoplot::bundled_font_db());
        let mut pxl = PxlSurface::new(
            size.width() as u32,
            size.height() as u32,
            Some(fontdb.clone()),
        )
        .unwrap();
        pxl.draw_figure(
            &fig,
            &(),
            theme,
            drawing::Options {
                fontdb: Some(fontdb),
            },
        )
        .unwrap();
        pxl.into_pixmap()
    }

    fn regenerate_refs() -> bool {
        REGENERATE_REFS
            || std::env::var("EIDOPLOT_TEST_REGENERATE_REFS").is_ok()
            || std::env::var("EIDOPLOT_TEST_REGENERATE_PNG_REFS").is_ok()
    }

    fn serialize_fig(file: &Path, fig: &Self::DrawnFig) {
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        fig.save_png(file).unwrap();
    }

    fn deserialize_fig(file: &Path) -> Self::DrawnFig {
        tiny_skia::Pixmap::load_png(file).unwrap()
    }
}

struct SvgHarness;

impl TestHarness for SvgHarness {
    type DrawnFig = Vec<u8>;

    fn draw<T>(fig: &ir::Figure, theme: T) -> Self::DrawnFig
    where
        T: style::Theme,
    {
        let size = fig.size();
        let mut svg = SvgSurface::new(size.width() as u32, size.height() as u32);
        svg.draw_figure(&fig, &(), theme, drawing::Options::default())
            .unwrap();
        let mut buf = Vec::new();
        svg.write(&mut buf).unwrap();
        buf
    }

    fn regenerate_refs() -> bool {
        REGENERATE_REFS
            || std::env::var("EIDOPLOT_TEST_REGENERATE_REFS").is_ok()
            || std::env::var("EIDOPLOT_TEST_REGENERATE_SVG_REFS").is_ok()
    }

    fn serialize_fig(file: &Path, fig: &Self::DrawnFig) {
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, fig).unwrap();
    }

    fn deserialize_fig(file: &Path) -> Self::DrawnFig {
        std::fs::read(file).unwrap()
    }
}

macro_rules! assert_fig_eq_ref {
    (__assert_pxl, $ref_file_name:expr, $prepared:expr) => {
        let actual_file = $crate::actual_file_path(&$ref_file_name);
        let diff_file = $crate::diff_file_path(&$ref_file_name);

        let (diff_pxl, diff_count) = $crate::pixelmatch::pixelmatch(
            $prepared.actual_drawn_fig.as_ref(),
            $prepared.ref_drawn_fig.as_ref(), Default::default());

        if diff_count != 0 {
            let ref_file = $crate::ref_file_path(&$ref_file_name);

            $crate::PxlHarness::serialize_fig(actual_file.as_path(), &$prepared.actual_drawn_fig);
            $crate::PxlHarness::serialize_fig(diff_file.as_path(), diff_pxl.as_ref().unwrap());

            panic!(
                "PXL assertion failed\n actual figure: {:?}\n    ref figure: {:?}\n    diff image: {:?}\n    diff count: {}",
                actual_file, ref_file, diff_file, diff_count
            );
        } else {
            if std::fs::exists(&actual_file).unwrap() {
                std::fs::remove_file(&actual_file).unwrap();
            }
            if std::fs::exists(&diff_file).unwrap() {
                std::fs::remove_file(&diff_file).unwrap();
            }
        }
    };
    (__assert_svg, $ref_file_name:expr, $prepared:expr) => {
        let actual_file = $crate::actual_file_path(&$ref_file_name);

        if $prepared.actual_drawn_fig != $prepared.ref_drawn_fig {
            let ref_file = $crate::ref_file_path(&$ref_file_name);
            $crate::SvgHarness::serialize_fig(&actual_file, &$prepared.actual_drawn_fig);
            panic!(
                "SVG assertion failed\n actual figure: {:?}\n    ref figure: {:?}",
                actual_file, ref_file
            );
        } else {
            if std::fs::exists(&actual_file).unwrap() {
                std::fs::remove_file(&actual_file).unwrap();
            }
        }
    };

    (pxl, $fig:expr, $ref_file_name:expr, $theme:expr) => {
        let prepared = $crate::PxlHarness::prepare_assertion($ref_file_name, $fig, $theme);
        assert_fig_eq_ref!(__assert_pxl, $ref_file_name, prepared);
    };
    (pxl, $fig:expr, $file:expr) => {
        assert_fig_eq_ref!(pxl, fig, expected, $crate::bw_theme());
    };

    (svg, $fig:expr, $ref_file_name:expr, $theme:expr) => {
        let prepared = $crate::SvgHarness::prepare_assertion($ref_file_name, $fig, $theme);
        assert_fig_eq_ref!(__assert_svg, $ref_file_name, prepared);
    };
    (svg, $fig:expr, $file:expr) => {
        assert_fig_eq_ref!(svg, fig, expected, $crate::bw_theme());
    };

    ($fig:expr, $file:expr, $theme:expr) => {
        let png_file = $file.to_string() + ".png";
        let svg_file = $file.to_string() + ".svg";
        assert_fig_eq_ref!(pxl, $fig, png_file.as_str(), $theme);
        assert_fig_eq_ref!(svg, $fig, svg_file.as_str(), $theme);
    };
    ($fig:expr, $file:expr) => {
        assert_fig_eq_ref!($fig, $file, $crate::bw_theme());
    };
}

pub(crate) use assert_fig_eq_ref;
