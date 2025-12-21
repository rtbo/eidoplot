use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use eidoplot::drawing::FigureExt;
use eidoplot::style::series::palettes;
use eidoplot::style::{self};
use eidoplot::{data, fontdb, ir, text};
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;
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
    Yes,
    YesToFile(String),
}

#[derive(Debug, Clone, Default)]
enum Svg {
    #[default]
    No,
    Yes,
    YesToFile(String),
}

#[derive(Debug, Clone, Copy, Default)]
enum Theme {
    #[default]
    LightStandard,
    LightPastel,
    LightTolBright,
    LightOkabeIto,
    DarkPastel,
    DarkStandard,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
}

impl From<Theme> for style::Theme {
    fn from(value: Theme) -> Self {
        match value {
            Theme::LightStandard => style::theme::light(palettes::standard()),
            Theme::LightPastel => style::theme::light(palettes::pastel()),
            Theme::LightTolBright => style::theme::light(palettes::tol_bright()),
            Theme::LightOkabeIto => style::theme::light(palettes::okabe_ito()),
            Theme::DarkPastel => style::theme::dark(palettes::pastel()),
            Theme::DarkStandard => style::theme::dark(palettes::standard()),
            Theme::CatppuccinLatte => style::catppuccin::latte(),
            Theme::CatppuccinFrappe => style::catppuccin::frappe(),
            Theme::CatppuccinMacchiato => style::catppuccin::macchiato(),
            Theme::CatppuccinMocha => style::catppuccin::mocha(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Args {
    png: Png,
    svg: Svg,
    theme: Theme,
}

fn parse_args() -> Args {
    let mut args = Args::default();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "png" => args.png = Png::Yes,
            "svg" => args.svg = Svg::Yes,
            "light" => args.theme = Theme::LightStandard,
            "light-standard" => args.theme = Theme::LightStandard,
            "light-pastel" => args.theme = Theme::LightPastel,
            "tol-bright" => args.theme = Theme::LightTolBright,
            "okabe-ito" => args.theme = Theme::LightOkabeIto,
            "dark" => args.theme = Theme::DarkPastel,
            "dark-pastel" => args.theme = Theme::DarkPastel,
            "dark-standard" => args.theme = Theme::DarkStandard,
            "latte" => args.theme = Theme::CatppuccinLatte,
            "frappe" => args.theme = Theme::CatppuccinFrappe,
            "macchiato" => args.theme = Theme::CatppuccinMacchiato,
            "mocha" => args.theme = Theme::CatppuccinMocha,
            "catppuccin-latte" => args.theme = Theme::CatppuccinLatte,
            "catppuccin-frappe" => args.theme = Theme::CatppuccinFrappe,
            "catppuccin-macchiato" => args.theme = Theme::CatppuccinMacchiato,
            "catppuccin-mocha" => args.theme = Theme::CatppuccinMocha,
            _ if arg.starts_with("png=") => {
                let filename = arg.trim_start_matches("png=");
                args.png = Png::YesToFile(filename.to_string());
            }
            _ if arg.starts_with("svg=") => {
                let filename = arg.trim_start_matches("svg=");
                args.svg = Svg::YesToFile(filename.to_string());
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
            }
        }
    }

    if matches!((&args.png, &args.svg), (Png::No, Svg::No)) {
        args.png = Png::Yes;
    }

    args
}

pub fn save_figure<D>(fig: &ir::Figure, data_source: &D, default_name: &str)
where
    D: data::Source,
{
    let args = parse_args();
    let theme = args.theme.into();
    let fontdb = Arc::new(text::bundled_font_db());
    save_fig(fig, data_source, &theme, &args, default_name, fontdb);
}

fn save_fig<D>(
    fig: &ir::Figure,
    data_source: &D,
    theme: &style::Theme,
    args: &Args,
    default_name: &str,
    fontdb: Arc<fontdb::Database>,
) where
    D: data::Source,
{
    match &args.png {
        Png::No => (),
        Png::Yes => save_fig_as_png(
            fig,
            data_source,
            theme,
            fontdb.clone(),
            &format!("{}.png", default_name),
        ),
        Png::YesToFile(file_name) => {
            save_fig_as_png(fig, data_source, theme, fontdb.clone(), &file_name)
        }
    }

    match &args.svg {
        Svg::No => (),
        Svg::Yes => save_fig_as_svg(
            fig,
            data_source,
            theme,
            fontdb,
            &format!("{}.svg", default_name),
        ),
        Svg::YesToFile(file_name) => save_fig_as_svg(fig, data_source, theme, fontdb, &file_name),
    }
}

fn save_fig_as_png<D>(
    fig: &ir::Figure,
    data_source: &D,
    theme: &style::Theme,
    fontdb: Arc<fontdb::Database>,
    file_name: &str,
) where
    D: data::Source,
{
    let width = (fig.size().width() * 2.0) as _;
    let height = (fig.size().height() * 2.0) as _;
    let mut pxl = PxlSurface::new(width, height).unwrap();
    fig.draw(&mut pxl, theme, data_source, Some(fontdb))
        .unwrap();
    pxl.save_png(file_name).unwrap();
}

fn save_fig_as_svg<D>(
    fig: &ir::Figure,
    data_source: &D,
    theme: &style::Theme,
    fontdb: Arc<fontdb::Database>,
    file_name: &str,
) where
    D: data::Source,
{
    let width = fig.size().width() as _;
    let height = fig.size().height() as _;
    let mut svg = SvgSurface::new(width, height);
    fig.draw(&mut svg, theme, data_source, Some(fontdb))
        .unwrap();
    svg.save_svg(file_name).unwrap();
}
