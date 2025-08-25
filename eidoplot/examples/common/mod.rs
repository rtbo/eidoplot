use std::env;
use std::sync::Arc;

use eidoplot::drawing::{self, SurfaceExt};
use eidoplot::style::{self, palette, theme};
use eidoplot::{data, ir};
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;
use eidoplot_text::fontdb;

pub fn save_figure<D>(fig: &ir::Figure, data_source: &D)
where
    D: data::Source,
{
    let theme = theme::Light::new(palette::Standard);

    let fontdb = Arc::new(eidoplot::bundled_font_db());

    let mut written = false;
    for arg in env::args() {
        if arg == "png" {
            write_png(&fig, data_source, theme.clone(), fontdb.clone());
            written = true;
        } else if arg == "svg" {
            write_svg(&fig, data_source, theme.clone());
            written = true;
        }
    }
    if !written {
        write_png(&fig, data_source, theme, fontdb);
    }
}

fn write_svg<D, T>(fig: &ir::Figure, data_source: &D, theme: T)
where
    D: data::Source,
    T: style::Theme,
{
    let mut svg = SvgSurface::new(800, 600);
    svg.draw_figure(fig, data_source, theme, drawing::Options::default())
        .unwrap();
    svg.save("plot.svg").unwrap();
}

fn write_png<D, T>(fig: &ir::Figure, data_source: &D, theme: T, fontdb: Arc<fontdb::Database>)
where
    D: data::Source,
    T: style::Theme,
{
    let mut pxl = PxlSurface::new(1600, 1200, Some(fontdb.clone())).unwrap();
    pxl.draw_figure(
        fig,
        data_source,
        theme,
        drawing::Options {
            fontdb: Some(fontdb.clone()),
        },
    )
    .unwrap();
    pxl.save_png("plot.png").unwrap();
}
