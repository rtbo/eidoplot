use eidoplot::drawing::{self, FigureExt};
use eidoplot::{ir, style};
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;

use std::sync::Arc;
use std::{env, f64::consts::PI};

fn main() {
    let points = (0..=360)
        .map(|t| (t as f64 * PI / 180.0, (t as f64 * PI / 180.0).sin()))
        .collect();

    let x_axis = ir::axis::Axis {
        label: Some("x".into()),
        ticks: Some(ir::axis::ticks::Locator::PiMultiple { bins: 8 }.into()),
        ..ir::axis::Axis::default()
    };
    let y_axis = ir::axis::Axis {
        label: Some("y".into()),
        ..ir::axis::Axis::default()
    };

    let series = ir::plot::Series {
        name: Some("y=sin(x)".into()),
        plot: ir::plot::SeriesPlot::Xy(ir::plot::XySeries {
            line: style::Line {
                color: style::color::BLUE,
                width: 3.0,
                pattern: style::LinePattern::Solid,
            },
            points,
        }),
    };

    let plot = ir::Plot {
        title: None,
        x_axis,
        y_axis,
        series: vec![series],
        ..ir::Plot::default()
    };

    let fig = ir::Figure::new(ir::figure::Plots::Plot(plot)).with_title(Some("Sine wave".into()));

    save_figure(&fig);
}

fn save_figure(fig: &ir::Figure) {
    let fontdb = eidoplot::default_font_db();

    let mut written = false;
    for arg in env::args() {
        if arg == "png" {
            write_png(&fig, fontdb.clone());
            written = true;
        } else if arg == "svg" {
            write_svg(&fig);
            written = true;
        }
    }
    if !written {
        write_png(&fig, fontdb);
    }
}

fn write_svg(fig: &ir::Figure) {
    let mut svg = SvgSurface::new(800, 600);
    fig.draw(&mut svg, drawing::Options::default()).unwrap();
    svg.save("plot.svg").unwrap();
}

fn write_png(fig: &ir::Figure, fontdb: Arc<fontdb::Database>) {
    let mut pxl = PxlSurface::new(1600, 1200);
    fig.draw(
        &mut pxl,
        drawing::Options {
            fontdb: Some(fontdb.clone()),
        },
    )
    .unwrap();
    pxl.save("plot.png", Some(fontdb)).unwrap();
}
