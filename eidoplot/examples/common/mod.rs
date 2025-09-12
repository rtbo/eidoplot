use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use eidoplot::drawing::{self, SurfaceExt};
use eidoplot::style::{self, series, theme};
use eidoplot::{data, ir};
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;
use eidoplot_text::fontdb;

/// Get the path to a file in the examples folder
#[allow(dead_code)]
pub fn example_res(path: &str) -> PathBuf {
    let root = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(root).join("examples").join(path)
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
                let filename = arg.trim_start_matches("png=").to_string();
                args.png = Png::YesToFile(filename);
            }
            _ if arg.starts_with("svg=") => {
                let filename = arg.trim_start_matches("svg=").to_string();
                args.svg = Svg::YesToFile(filename);
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
    save_fig_match_theme(fig, data_source, &args, default_name);
}

fn save_fig_match_theme<D>(fig: &ir::Figure, data_source: &D, args: &Args, default_name: &str)
where
    D: data::Source,
{
    let fontdb = Arc::new(eidoplot::bundled_font_db());

    match &args.theme {
        Theme::LightStandard => {
            save_fig_with_theme(
                fig,
                data_source,
                theme::Light::new(series::STANDARD),
                args,
                default_name,
                fontdb,
            );
        }
        Theme::LightPastel => {
            save_fig_with_theme(
                fig,
                data_source,
                theme::Light::new(series::PASTEL),
                args,
                default_name,
                fontdb,
            );
        }
        Theme::LightTolBright => {
            save_fig_with_theme(
                fig,
                data_source,
                theme::Light::new(series::TOL_BRIGHT),
                args,
                default_name,
                fontdb,
            );
        }
        Theme::LightOkabeIto => {
            save_fig_with_theme(
                fig,
                data_source,
                theme::Light::new(series::OKABE_ITO),
                args,
                default_name,
                fontdb,
            );
        }
        Theme::DarkPastel => {
            save_fig_with_theme(
                fig,
                data_source,
                theme::Dark::new(series::PASTEL),
                args,
                default_name,
                fontdb,
            );
        }
        Theme::DarkStandard => {
            save_fig_with_theme(
                fig,
                data_source,
                theme::Dark::new(series::STANDARD),
                args,
                default_name,
                fontdb,
            );
        }
        Theme::CatppuccinLatte => {
            save_fig_with_theme(
                fig,
                data_source,
                style::catppuccin::Latte,
                args,
                default_name,
                fontdb,
            );
        }
        Theme::CatppuccinFrappe => {
            save_fig_with_theme(
                fig,
                data_source,
                style::catppuccin::Frappe,
                args,
                default_name,
                fontdb,
            );
        }
        Theme::CatppuccinMacchiato => {
            save_fig_with_theme(
                fig,
                data_source,
                style::catppuccin::Macchiato,
                args,
                default_name,
                fontdb,
            );
        }
        Theme::CatppuccinMocha => {
            save_fig_with_theme(
                fig,
                data_source,
                style::catppuccin::Mocha,
                args,
                default_name,
                fontdb,
            );
        }
    }
}

fn save_fig_with_theme<T, D>(
    fig: &ir::Figure,
    data_source: &D,
    theme: T,
    args: &Args,
    default_name: &str,
    fontdb: Arc<fontdb::Database>,
) where
    D: data::Source,
    T: style::Theme + Clone,
{
    match &args.png {
        Png::No => (),
        Png::Yes => save_fig_as_png(
            fig,
            data_source,
            theme.clone(),
            fontdb.clone(),
            &format!("{}.png", default_name),
        ),
        Png::YesToFile(file_name) => {
            save_fig_as_png(fig, data_source, theme.clone(), fontdb.clone(), &file_name)
        }
    }

    match &args.svg {
        Svg::No => (),
        Svg::Yes => save_fig_as_svg(
            fig,
            data_source,
            theme.clone(),
            fontdb,
            &format!("{}.svg", default_name),
        ),
        Svg::YesToFile(file_name) => {
            save_fig_as_svg(fig, data_source, theme.clone(), fontdb, &file_name)
        }
    }
}

fn save_fig_as_png<D, T>(
    fig: &ir::Figure,
    data_source: &D,
    theme: T,
    fontdb: Arc<fontdb::Database>,
    file_name: &str,
) where
    D: data::Source,
    T: style::Theme,
{
    let mut pxl = PxlSurface::new(1600, 1200, Some(fontdb.clone())).unwrap();
    pxl.draw_figure(
        fig,
        data_source,
        theme,
        drawing::Options {
            fontdb: Some(fontdb),
        },
    )
    .unwrap();
    pxl.save_png(file_name).unwrap();
}

fn save_fig_as_svg<D, T>(
    fig: &ir::Figure,
    data_source: &D,
    theme: T,
    fontdb: Arc<fontdb::Database>,
    file_name: &str,
) where
    D: data::Source,
    T: style::Theme,
{
    let mut svg = SvgSurface::new(800, 600);
    svg.draw_figure(
        fig,
        data_source,
        theme,
        drawing::Options {
            fontdb: Some(fontdb),
        },
    )
    .unwrap();
    svg.save_svg(file_name).unwrap();
}
