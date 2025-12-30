use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use eidoplot::style::{self};
use eidoplot::{Drawing, data, fontdb, ir};
use eidoplot_pxl::SavePng;
use eidoplot_svg::SaveSvg;
use eidoplot_iced::Show;
use rand::SeedableRng;

/// Get the path to a file in the examples folder
#[allow(dead_code)]
pub fn example_res(path: &str) -> PathBuf {
    let root = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(root).join("examples").join(path)
}

/// Get a predictable random number generator
#[allow(dead_code)]
pub fn predictable_rng(seed: Option<u64>) -> impl rand::Rng {
    let seed = seed.unwrap_or(586350478348);
    rand_chacha::ChaCha8Rng::seed_from_u64(seed)
}

#[derive(Debug, Clone, Default)]
enum Png {
    #[default]
    No,
    Yes(Option<PathBuf>),
}

#[derive(Debug, Clone, Default)]
enum Svg {
    #[default]
    No,
    Yes(Option<PathBuf>),
}

#[derive(Debug, Clone, Default)]
struct Args {
    png: Png,
    svg: Svg,
    show: bool,
    style: Option<style::Builtin>,
}

fn parse_args() -> Args {
    let mut args = Args::default();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "png" => args.png = Png::Yes(None),
            "svg" => args.svg = Svg::Yes(None),
            "show" => args.show = true,
            "light" => args.style = Some(style::Builtin::Light),
            "tol-bright" => args.style = Some(style::Builtin::TolBright),
            "okabe-ito" => args.style = Some(style::Builtin::OkabeIto),
            "dark" => args.style = Some(style::Builtin::Dark),
            "latte" => args.style = Some(style::Builtin::CatppuccinLatte),
            "frappe" => args.style = Some(style::Builtin::CatppuccinFrappe),
            "macchiato" => args.style = Some(style::Builtin::CatppuccinMacchiato),
            "mocha" => args.style = Some(style::Builtin::CatppuccinMocha),
            "catppuccin-latte" => args.style = Some(style::Builtin::CatppuccinLatte),
            "catppuccin-frappe" => args.style = Some(style::Builtin::CatppuccinFrappe),
            "catppuccin-macchiato" => args.style = Some(style::Builtin::CatppuccinMacchiato),
            "catppuccin-mocha" => args.style = Some(style::Builtin::CatppuccinMocha),
            _ if arg.starts_with("png=") => {
                let filename = arg.trim_start_matches("png=");
                args.png = Png::Yes(Some(PathBuf::from(filename)));
            }
            _ if arg.starts_with("svg=") => {
                let filename = arg.trim_start_matches("svg=");
                args.svg = Svg::Yes(Some(PathBuf::from(filename)));
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
            }
        }
    }

    if matches!(
        (&args.png, &args.svg, &args.show),
        (Png::No, Svg::No, false)
    ) {
        args.show = true;
    }

    args
}

pub fn save_figure<D>(
    fig: &ir::Figure,
    data_source: &D,
    fontdb: Option<&fontdb::Database>,
    default_name: &str,
) where
    D: data::Source,
{
    let args = parse_args();
    if let Some(fontdb) = fontdb {
        save_fig(fig, data_source, &args, fontdb, default_name);
    } else {
        let fontdb = eidoplot::bundled_font_db();
        save_fig(fig, data_source, &args, &fontdb, default_name);
    }
}

fn save_fig<D>(
    fig: &ir::Figure,
    data_source: &D,
    args: &Args,
    fontdb: &fontdb::Database,
    default_name: &str,
) where
    D: data::Source,
{
    let fig = fig.prepare(data_source, Some(fontdb)).unwrap();
    let style = args.style.clone().unwrap_or_default().to_style();

    match &args.png {
        Png::No => (),
        Png::Yes(filename) => {
            let file_name = match filename {
                Some(path) => path.to_string_lossy().to_string(),
                None => format!("{}.png", default_name),
            };
            fig.save_png(
                &file_name,
                eidoplot_pxl::DrawingParams {
                    style: style.clone(),
                    scale: 2.0,
                },
            )
            .unwrap();
        }
    }

    match &args.svg {
        Svg::No => (),
        Svg::Yes(filename) => {
            let file_name = match filename {
                Some(path) => path.to_string_lossy().to_string(),
                None => format!("{}.svg", default_name),
            };
            fig.save_svg(
                &file_name,
                eidoplot_svg::DrawingParams {
                    style: style.clone(),
                    scale: 1.0,
                },
            )
            .unwrap();
        }
    }

    if args.show {
        let data_source = data_source.copy();
        let fontdb = Arc::new(fontdb.clone());

        fig.show(data_source, fontdb, Some(style.to_custom()))
            .unwrap();
    }
}
