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
    let stem = path.file_stem().and_then(std::ffi::OsStr::to_str).unwrap();
    let ext = path.extension().and_then(std::ffi::OsStr::to_str).unwrap();
    let file = format!("{}-diff.{}", stem, ext);

    Path::new(tests_dir).join("actual").join(file)
}

fn bw_theme() -> impl style::Theme {
    style::theme::Light::new(style::series::BLACK)
}

fn fig_to_pxl<T>(fig: &ir::Figure, theme: T) -> tiny_skia::Pixmap
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

fn fig_to_svg<T>(fig: &ir::Figure, theme: T) -> Vec<u8>
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

fn pxl_to_file(pxl: &tiny_skia::Pixmap, file: &Path) {
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    pxl.save_png(file).unwrap();
}

fn svg_to_file(svg: &[u8], file: &Path) {
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, svg).unwrap();
}

#[cfg(feature = "regenerate-refs")]
const REGENERATE_REFS: bool = true;

#[cfg(not(feature = "regenerate-refs"))]
const REGENERATE_REFS: bool = false;

#[cfg(feature = "allow-no-ref")]
const ALLOW_NO_REF: bool = true;

#[cfg(not(feature = "allow-no-ref"))]
const ALLOW_NO_REF: bool = false;

macro_rules! assert_fig_eq_ref {
    (__assert_pxl, $actual_pxl:expr, $ref_pxl:expr, $file:expr) => {
        let actual_file = $crate::actual_file_path($file);
        let diff_file = $crate::diff_file_path($file);

        let (diff_pxl, diff_count) = $crate::pixelmatch::pixelmatch($actual_pxl.as_ref(), $ref_pxl.as_ref(), Default::default());

        if diff_count != 0 {
            let ref_file = $crate::ref_file_path($file);

            $crate::pxl_to_file(&$actual_pxl, &actual_file);
            $crate::pxl_to_file(diff_pxl.as_ref().unwrap(), &diff_file);

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
    (__assert_svg, $actual_svg:expr, $ref_svg:expr, $file:expr) => {
        let actual_file = $crate::actual_file_path($file);
        if $actual_svg != $ref_svg {
            let ref_file = $crate::ref_file_path($file);
            $crate::svg_to_file(&$actual_svg, &actual_file);
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

    (pxl, $fig:expr, $file:expr, $theme:expr) => {
        let actual_pxl = $crate::fig_to_pxl($fig, $theme);
        let ref_file = $crate::ref_file_path($file);

        if $crate::REGENERATE_REFS
            || std::env::var("EIDOPLOT_TEST_REGENERATE_REFS").is_ok()
            || std::env::var("EIDOPLOT_TEST_REGENERATE_PNG_REFS").is_ok()
        {
            $crate::pxl_to_file(&actual_pxl, &ref_file);
        }

        if !std::fs::exists(&ref_file).unwrap() {
            let actual_file = $crate::actual_file_path($file);
            $crate::pxl_to_file(&actual_pxl, &actual_file);
            if !$crate::ALLOW_NO_REF {
                panic!("No such ref: \"{}\"\n  Actual figure written to {}", $file, actual_file.display());
            }
        } else {
            let ref_pxl = tiny_skia::Pixmap::load_png(&ref_file).unwrap();
            assert_fig_eq_ref!(__assert_pxl, actual_pxl, ref_pxl, $file);
        }
    };
    (pxl, $fig:expr, $file:expr) => {
        assert_fig_eq_ref!(pxl, fig, expected, $crate::bw_theme());
    };

    (svg, $fig:expr, $file:expr, $theme:expr) => {
        let actual_svg = $crate::fig_to_svg($fig, $theme);
        let ref_file = $crate::ref_file_path($file);

        if $crate::REGENERATE_REFS
            || std::env::var("EIDOPLOT_TEST_REGENERATE_REFS").is_ok()
            || std::env::var("EIDOPLOT_TEST_REGENERATE_SVG_REFS").is_ok()
        {
            svg_to_file(&actual_svg, &ref_file);
        }

        if !std::fs::exists(&ref_file).unwrap() {
            let actual_file = $crate::actual_file_path($file);
            svg_to_file(&actual_svg, &actual_file);
            if !$crate::ALLOW_NO_REF {
                panic!("No such ref: \"{}\"\n  Actual figure written to {}", $file, actual_file.display());
            }
        } else {
            let ref_svg = std::fs::read(&ref_file).unwrap();
            assert_fig_eq_ref!(__assert_svg, actual_svg, ref_svg, $file);
        }
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
