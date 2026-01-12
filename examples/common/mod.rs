use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use plotive::{Prepare, Style, data, des, fontdb};
use plotive_iced::Show;
use plotive_pxl::SavePng;
use plotive_svg::SaveSvg;
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
    style: Option<Style>,
}

fn parse_args() -> Args {
    let mut args = Args::default();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "png" => args.png = Png::Yes(None),
            "svg" => args.svg = Svg::Yes(None),
            "show" => args.show = true,
            "light" => args.style = Some(Style::light()),
            "tol-bright" => args.style = Some(Style::tol_bright()),
            "okabe-ito" => args.style = Some(Style::okabe_ito()),
            "dark" => args.style = Some(Style::dark()),
            "latte" => args.style = Some(Style::catppuccin_latte()),
            "frappe" => args.style = Some(Style::catppuccin_frappe()),
            "macchiato" => args.style = Some(Style::catppuccin_macchiato()),
            "mocha" => args.style = Some(Style::catppuccin_mocha()),
            "catppuccin-latte" => args.style = Some(Style::catppuccin_latte()),
            "catppuccin-frappe" => args.style = Some(Style::catppuccin_frappe()),
            "catppuccin-macchiato" => args.style = Some(Style::catppuccin_macchiato()),
            "catppuccin-mocha" => args.style = Some(Style::catppuccin_mocha()),
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
    fig: &des::Figure,
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
        let fontdb = plotive::bundled_font_db();
        save_fig(fig, data_source, &args, &fontdb, default_name);
    }
}

fn save_fig<D>(
    fig: &des::Figure,
    data_source: &D,
    args: &Args,
    fontdb: &fontdb::Database,
    default_name: &str,
) where
    D: data::Source,
{
    let fig = fig.prepare(data_source, Some(fontdb)).unwrap();

    match &args.png {
        Png::No => (),
        Png::Yes(filename) => {
            let file_name = match filename {
                Some(path) => path.to_string_lossy().to_string(),
                None => format!("{}.png", default_name),
            };
            fig.save_png(
                &file_name,
                data_source,
                plotive_pxl::Params {
                    style: args.style.as_ref().cloned().unwrap_or_default(),
                    scale: 2.0,
                    fontdb: Some(fontdb),
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
                plotive_svg::Params {
                    style: args.style.as_ref().cloned().unwrap_or_default(),
                    scale: 1.0,
                },
            )
            .unwrap();
        }
    }

    if args.show {
        let data_source = data_source.copy();
        let fontdb = Arc::new(fontdb.clone());

        fig.show(
            data_source,
            plotive_iced::show::Params {
                style: args.style.clone(),
                fontdb: Some(fontdb),
                ..Default::default()
            },
        )
        .unwrap();
    }
}
