use eidoplot::data;
use eidoplot::drawing::{self, SurfaceExt};
use eidoplot::ir;

use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;

use std::env;
use std::sync::Arc;

pub fn save_figure<D>(fig: &ir::Figure, data_source: &D)
where
    D: data::Source,
{
    let fontdb = Arc::new(eidoplot::bundled_font_db());

    let mut written = false;
    for arg in env::args() {
        if arg == "png" {
            write_png(&fig, data_source, fontdb.clone());
            written = true;
        } else if arg == "svg" {
            write_svg(&fig, data_source);
            written = true;
        }
    }
    if !written {
        write_png(&fig, data_source, fontdb);
    }
}

fn write_svg<D>(fig: &ir::Figure, data_source: &D)
where
    D: data::Source,
{
    let mut svg = SvgSurface::new(800, 600);
    svg.draw_figure(fig, data_source, drawing::Options::default())
        .unwrap();
    svg.save("plot.svg").unwrap();
}

fn write_png<D>(fig: &ir::Figure, data_source: &D, fontdb: Arc<fontdb::Database>)
where
    D: data::Source,
{
    let mut pxl = PxlSurface::new(1600, 1200, Some(fontdb.clone())).unwrap();
    pxl.draw_figure(
        fig,
        data_source,
        drawing::Options {
            fontdb: Some(fontdb.clone()),
        },
    )
    .unwrap();
    pxl.save_png("plot.png").unwrap();
}
