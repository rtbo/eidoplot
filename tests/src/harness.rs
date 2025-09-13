use std::path::{Path, PathBuf};
use std::sync::Arc;

use eidoplot::drawing::{self, SurfaceExt};
use eidoplot::{ir, style};
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;

use crate::pixelmatch::pixelmatch;

#[cfg(feature = "regenerate-refs")]
const REGENERATE_REFS: bool = true;

#[cfg(not(feature = "regenerate-refs"))]
const REGENERATE_REFS: bool = false;

pub trait TestHarness {
    type DrawnFig;
    type DiffFig;

    fn id() -> &'static str;
    fn fig_file_ext() -> &'static str;
    fn diff_file_suffix() -> &'static str;

    fn ref_file_path(ref_name: &str) -> PathBuf {
        let file_name = format!("{}{}", ref_name, Self::fig_file_ext());
        let tests_dir = env!("CARGO_MANIFEST_DIR");
        Path::new(tests_dir).join("refs").join(file_name)
    }

    fn actual_file_path(ref_name: &str) -> PathBuf {
        let file_name = format!("{}{}", ref_name, Self::fig_file_ext());
        let tests_dir = env!("CARGO_MANIFEST_DIR");
        Path::new(tests_dir).join("actual").join(file_name)
    }

    fn diff_file_path(ref_name: &str) -> PathBuf {
        let file_name = format!("{}{}", ref_name, Self::diff_file_suffix());
        let tests_dir = env!("CARGO_MANIFEST_DIR");
        Path::new(tests_dir).join("actual").join(file_name)
    }

    fn draw_fig<T>(fig: &ir::Figure, theme: T) -> Self::DrawnFig
    where
        T: style::Theme;

    fn diff_fig(actual: &Self::DrawnFig, ref_: &Self::DrawnFig) -> Option<Self::DiffFig>;

    fn serialize_fig(file: &Path, fig: &Self::DrawnFig);
    fn deserialize_fig(file: &Path) -> Self::DrawnFig;
    fn serialize_diff(file: &Path, diff: &Self::DiffFig);

    fn regenerate_refs() -> bool;

    fn check_fig_eq_ref<T>(fig: &ir::Figure, ref_name: &str, theme: T) -> Result<(), String>
    where
        T: style::Theme,
    {
        let ref_file = Self::ref_file_path(&ref_name);
        let actual_file = Self::actual_file_path(&ref_name);
        let diff_file = Self::diff_file_path(&ref_name);

        let actual_fig = Self::draw_fig(fig, theme);

        if Self::regenerate_refs() {
            std::fs::create_dir_all(ref_file.parent().unwrap()).unwrap();
            Self::serialize_fig(&ref_file, &actual_fig);

            if std::fs::exists(&actual_file).unwrap() {
                std::fs::remove_file(&actual_file).unwrap();
            }
            if std::fs::exists(&diff_file).unwrap() {
                std::fs::remove_file(&diff_file).unwrap();
            }

            return Ok(());
        }

        if !std::fs::exists(&ref_file).unwrap() {
            std::fs::create_dir_all(actual_file.parent().unwrap()).unwrap();
            Self::serialize_fig(actual_file.as_path(), &actual_fig);
            return Err(format!(
                "No such {} ref: \"{}\"\n  Actual figure written to {}",
                Self::id(),
                ref_name,
                actual_file.display()
            ));
        }

        let ref_fig = Self::deserialize_fig(&ref_file);

        if let Some(diff_fig) = Self::diff_fig(&actual_fig, &ref_fig) {
            std::fs::create_dir_all(actual_file.parent().unwrap()).unwrap();
            std::fs::create_dir_all(diff_file.parent().unwrap()).unwrap();
            Self::serialize_fig(actual_file.as_path(), &actual_fig);
            Self::serialize_diff(diff_file.as_path(), &diff_fig);

            Err(format!(
                "{} assertion failed\n  Actual figure: {:?}\n     Ref figure: {:?}\n           Diff: {:?}",
                Self::id(),
                actual_file,
                ref_file,
                diff_file
            ))
        } else {
            if std::fs::exists(&actual_file).unwrap() {
                std::fs::remove_file(&actual_file).unwrap();
            }
            if std::fs::exists(&diff_file).unwrap() {
                std::fs::remove_file(&diff_file).unwrap();
            }
            Ok(())
        }
    }
}

pub struct PxlHarness;

impl TestHarness for PxlHarness {
    type DrawnFig = tiny_skia::Pixmap;
    type DiffFig = tiny_skia::Pixmap;

    fn id() -> &'static str {
        "PXL"
    }

    fn fig_file_ext() -> &'static str {
        ".png"
    }

    fn diff_file_suffix() -> &'static str {
        "-diff.png"
    }

    fn draw_fig<T>(fig: &ir::Figure, theme: T) -> Self::DrawnFig
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

    fn diff_fig(actual_fig: &Self::DrawnFig, ref_fig: &Self::DrawnFig) -> Option<Self::DiffFig> {
        let (diff_pxl, diff_count) =
            pixelmatch(actual_fig.as_ref(), ref_fig.as_ref(), Default::default());
        if diff_count > 0 {
            Some(diff_pxl.unwrap())
        } else {
            None
        }
    }

    fn regenerate_refs() -> bool {
        REGENERATE_REFS
            || std::env::var("EIDOPLOT_TEST_REGENERATE_REFS").is_ok()
            || std::env::var("EIDOPLOT_TEST_REGENERATE_PNG_REFS").is_ok()
    }

    fn serialize_fig(file: &Path, fig: &Self::DrawnFig) {
        fig.save_png(file).unwrap();
    }

    fn deserialize_fig(file: &Path) -> Self::DrawnFig {
        tiny_skia::Pixmap::load_png(file).unwrap()
    }

    fn serialize_diff(file: &Path, diff: &Self::DiffFig) {
        diff.save_png(file).unwrap();
    }
}

pub struct SvgHarness;

impl TestHarness for SvgHarness {
    type DrawnFig = String;
    type DiffFig = String;

    fn id() -> &'static str {
        "SVG"
    }

    fn fig_file_ext() -> &'static str {
        ".svg"
    }

    fn diff_file_suffix() -> &'static str {
        ".svg.diff"
    }

    fn draw_fig<T>(fig: &ir::Figure, theme: T) -> Self::DrawnFig
    where
        T: style::Theme,
    {
        let size = fig.size();
        let mut svg = SvgSurface::new(size.width() as u32, size.height() as u32);
        svg.draw_figure(&fig, &(), theme, drawing::Options::default())
            .unwrap();
        let mut buf = Vec::new();
        svg.write(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    fn diff_fig(actual_fig: &Self::DrawnFig, ref_fig: &Self::DrawnFig) -> Option<Self::DiffFig> {
        if actual_fig != ref_fig {
            let diff = similar::TextDiff::from_lines(ref_fig.as_str(), actual_fig.as_str());
            let udiff = diff.unified_diff();
            Some(udiff.to_string())
        } else {
            None
        }
    }

    fn regenerate_refs() -> bool {
        REGENERATE_REFS
            || std::env::var("EIDOPLOT_TEST_REGENERATE_REFS").is_ok()
            || std::env::var("EIDOPLOT_TEST_REGENERATE_SVG_REFS").is_ok()
    }

    fn serialize_fig(file: &Path, fig: &Self::DrawnFig) {
        std::fs::write(file, fig).unwrap();
    }

    fn deserialize_fig(file: &Path) -> Self::DrawnFig {
        let buf = std::fs::read(file).unwrap();
        String::from_utf8(buf).unwrap()
    }

    fn serialize_diff(file: &Path, diff: &Self::DiffFig) {
        std::fs::write(file, diff).unwrap();
    }
}
