use std::env;
use std::path::PathBuf;

use eidoplot::style::{self};
use eidoplot::{Drawing, data, fontdb, ir};
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;
use iced_oplot::Show;
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

#[allow(dead_code)]
pub fn save_figure<D>(fig: &ir::Figure, data_source: &D, default_name: &str)
where
    D: data::Source,
{
    let args = parse_args();
    let fontdb = eidoplot::bundled_font_db();
    save_fig(fig, data_source, &args, default_name, &fontdb);
}

#[allow(dead_code)]
pub fn save_figure_with_fontdb<D>(
    fig: &ir::Figure,
    data_source: &D,
    fontdb: &fontdb::Database,
    default_name: &str,
) where
    D: data::Source,
{
    let args = parse_args();
    save_fig(fig, data_source, &args, default_name, &fontdb);
}

fn save_fig<D>(
    fig: &ir::Figure,
    data_source: &D,
    args: &Args,
    default_name: &str,
    fontdb: &fontdb::Database,
) where
    D: data::Source,
{
    match &args.png {
        Png::No => (),
        Png::Yes(filename) => {
            let file_name = match filename {
                Some(path) => path.to_string_lossy().to_string(),
                None => format!("{}.png", default_name),
            };
            let style = args.style.clone().unwrap_or_default().to_style();
            save_fig_as_png(fig, data_source, &style, fontdb, &file_name);
        }
    }

    match &args.svg {
        Svg::No => (),
        Svg::Yes(filename) => {
            let file_name = match filename {
                Some(path) => path.to_string_lossy().to_string(),
                None => format!("{}.svg", default_name),
            };
            let style = args.style.clone().unwrap_or_default().to_style();
            save_fig_as_svg(fig, data_source, &style, fontdb, &file_name);
        }
    }

    if args.show {
        let fig = fig.prepare(data_source, Some(fontdb)).unwrap();
        let style = args.style.map(|s| s.to_style().to_custom());
        fig.show(style).unwrap();
    }
}

fn save_fig_as_png<D>(
    fig: &ir::Figure,
    data_source: &D,
    style: &style::Style,
    fontdb: &fontdb::Database,
    file_name: &str,
) where
    D: data::Source,
{
    let width = (fig.size().width() * 2.0) as _;
    let height = (fig.size().height() * 2.0) as _;
    let mut pxl = PxlSurface::new(width, height).unwrap();
    fig.draw(data_source, Some(fontdb), &mut pxl, style)
        .unwrap();
    pxl.save_png(file_name).unwrap();
}

fn save_fig_as_svg<D>(
    fig: &ir::Figure,
    data_source: &D,
    style: &style::Style,
    fontdb: &fontdb::Database,
    file_name: &str,
) where
    D: data::Source,
{
    let width = fig.size().width() as _;
    let height = fig.size().height() as _;
    let mut svg = SvgSurface::new(width, height);
    fig.draw(data_source, Some(fontdb), &mut svg, style)
        .unwrap();
    svg.save_svg(file_name).unwrap();
}
