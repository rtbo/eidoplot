use std::env;
use std::sync::Arc;

use eidoplot::drawing::{self, SurfaceExt};
use eidoplot::style::{self, series, theme};
use eidoplot::{data, ir};
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;
use eidoplot_text::fontdb;

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

pub fn save_figure<D>(fig: &ir::Figure, data_source: &D)
where
    D: data::Source,
{
    let args = parse_args();
    save_fig_with_theme(fig, data_source, &args);
}

fn save_fig_with_theme<D>(fig: &ir::Figure, data_source: &D, args: &Args)
where
    D: data::Source,
{
    let fontdb = Arc::new(eidoplot::bundled_font_db());

    match &args.theme {
        Theme::LightStandard => {
            save_fig_with_resolved_theme(
                fig,
                data_source,
                theme::Light::new(series::STANDARD),
                args,
                fontdb,
            );
        }
        Theme::LightPastel => {
            save_fig_with_resolved_theme(
                fig,
                data_source,
                theme::Light::new(series::PASTEL),
                args,
                fontdb,
            );
        }
        Theme::LightTolBright => {
            save_fig_with_resolved_theme(
                fig,
                data_source,
                theme::Light::new(series::TOL_BRIGHT),
                args,
                fontdb,
            );
        }
        Theme::LightOkabeIto => {
            save_fig_with_resolved_theme(
                fig,
                data_source,
                theme::Light::new(series::OKABE_ITO),
                args,
                fontdb,
            );
        }
        Theme::DarkPastel => {
            save_fig_with_resolved_theme(
                fig,
                data_source,
                theme::Dark::new(series::PASTEL),
                args,
                fontdb,
            );
        }
        Theme::DarkStandard => {
            save_fig_with_resolved_theme(
                fig,
                data_source,
                theme::Dark::new(series::STANDARD),
                args,
                fontdb,
            );
        }
        Theme::CatppuccinLatte => {
            save_fig_with_resolved_theme(fig, data_source, style::catppuccin::Latte, args, fontdb);
        }
        Theme::CatppuccinFrappe => {
            save_fig_with_resolved_theme(fig, data_source, style::catppuccin::Frappe, args, fontdb);
        }
        Theme::CatppuccinMacchiato => {
            save_fig_with_resolved_theme(
                fig,
                data_source,
                style::catppuccin::Macchiato,
                args,
                fontdb,
            );
        }
        Theme::CatppuccinMocha => {
            save_fig_with_resolved_theme(fig, data_source, style::catppuccin::Mocha, args, fontdb);
        }
    }
}

fn save_fig_with_resolved_theme<T, D>(
    fig: &ir::Figure,
    data_source: &D,
    theme: T,
    args: &Args,
    fontdb: Arc<fontdb::Database>,
) where
    D: data::Source,
    T: style::Theme + Clone,
{
    match &args.png {
        Png::No => (),
        Png::Yes => write_png(fig, data_source, theme.clone(), fontdb.clone(), "plot.png"),
        Png::YesToFile(file_name) => {
            write_png(fig, data_source, theme.clone(), fontdb.clone(), &file_name)
        }
    }

    match &args.svg {
        Svg::No => (),
        Svg::Yes => write_svg(fig, data_source, theme.clone(), fontdb, "plot.png"),
        Svg::YesToFile(file_name) => write_svg(fig, data_source, theme.clone(), fontdb, &file_name),
    }
}

fn write_svg<D, T>(
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
    svg.save(file_name).unwrap();
}

fn write_png<D, T>(
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
