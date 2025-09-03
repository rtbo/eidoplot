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

fn ref_file_bytes(file: &str) -> Vec<u8> {
    let path = ref_file_path(file);
    std::fs::read(&path).expect("File should exist")
}

fn ref_png_pxl(file: &str) -> tiny_skia::Pixmap {
    let path = ref_file_path(file);
    tiny_skia::Pixmap::load_png(&path).unwrap()
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

fn save_fig_as_png(fig: &ir::Figure, file: &str, theme: impl style::Theme) {
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
    pxl.save_png(ref_file_path(file)).unwrap();
}

fn save_fig_as_svg(fig: &ir::Figure, file: &str, theme: impl style::Theme) {
    let size = fig.size();
    let mut svg = SvgSurface::new(size.width() as u32, size.height() as u32);
    svg.draw_figure(&fig, &(), theme, drawing::Options::default())
        .unwrap();
    svg.save_svg(ref_file_path(file)).unwrap();
}

fn actual_file_path(file: &str) -> PathBuf {
    let tests_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(tests_dir).join("actual").join(file)
}

fn diff_file_path(file: &str) -> PathBuf {
    let tests_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(tests_dir).join("diffs").join(file)
}

macro_rules! assert_fig_eq_ref {
    (__assert_pxl, $actual_pxl:expr, $ref_pxl:expr, $file:expr) => {

        let actual_file = $crate::actual_file_path($file);
        let diff_file = $crate::diff_file_path($file);

        let (diff_px, diff_count) = $crate::pixelmatch::pixelmatch($actual_pxl.as_ref(), $ref_pxl.as_ref(), Default::default());

        if diff_count != 0 {
            let ref_file = $crate::ref_file_path($file);

            $actual_pxl.save_png(&actual_file).unwrap();

            diff_px.unwrap().save_png(&diff_file).unwrap();

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
            std::fs::write(&actual_file, &$actual_svg).unwrap();
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
        if std::env::var("EIDOPLOT_TEST_REGENERATE_REFS").is_ok()
            || std::env::var("EIDOPLOT_TEST_REGENERATE_PNG_REFS").is_ok()
        {
            save_fig_as_png($fig, $file, $theme);
        }
        let actual_pxl = $crate::fig_to_pxl($fig, $theme);
        // if !$crate::ref_file_path($file).exists() {
        //     actual_pxl.save_png($crate::actual_file_path($file)).unwrap(); 
        // }
        let ref_pxl = $crate::ref_png_pxl($file);
        assert_fig_eq_ref!(__assert_pxl, actual_pxl, ref_pxl, $file);
    };
    (pxl, $fig:expr, $file:expr) => {
        assert_fig_eq_ref!(pxl, fig, expected, $crate::bw_theme());
    };

    (svg, $fig:expr, $file:expr, $theme:expr) => {
        if std::env::var("EIDOPLOT_TEST_REGENERATE_REFS").is_ok()
            || std::env::var("EIDOPLOT_TEST_REGENERATE_SVG_REFS").is_ok()
        {
            save_fig_as_svg($fig, $file, $theme);
        }
        let actual_svg = $crate::fig_to_svg($fig, $theme);
        // if !$crate::ref_file_path($file).exists() {
        //     std::fs::write($crate::actual_file_path($file), &actual_svg).unwrap();
        // }
        let ref_svg = $crate::ref_file_bytes($file);
        assert_fig_eq_ref!(__assert_svg, actual_svg, ref_svg, $file);
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
