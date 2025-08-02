use eidoplot::prelude::*;
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;

use std::{env, f64::consts::PI};

fn main() {
    let points = (0..=360).map(|t| (t as f64 * PI / 180.0, (t as f64 * PI / 180.0).sin())).collect();
    let fig = Figure {
        size: FigSize::default(),
        title: Some("Sine wave".into()),
        padding: 20.0.into(),
        fill: Some(css::ANTIQUEWHITE.into()),
        plots: Some(Plots::Plot(Plot {
            title: None,
            fill: Some(css::ALICEBLUE.into()),
            x_axis: axis::Axis {
                name: Some("x".into()),
                ticks: Some(TickLocator::PiMultiple { num: 1.0, den: 2.0 }),
                ..axis::Axis::default()
            },
            y_axis: axis::Axis {
                name: Some("y".into()),
                ..axis::Axis::default()
            },
            series: vec![Series {
                name: Some("y=sin(x)".into()),
                plot: SeriesPlot::Xy {
                    line: css::FUCHSIA.into(), 
                    points,
                }
            }],
        })),
    };

    let mut written = false;

    for arg in env::args() {
        if arg == "png" {
            write_png(&fig);
            written = false;
        } else if arg == "svg" {
            write_svg(&fig);
            written = false;
        }
    }

    if !written {
        write_svg(&fig);
    }
}

fn write_svg(fig: &Figure) {
    let mut svg = SvgSurface::new(1200, 900);
    fig.draw(&mut svg).unwrap();
    svg.save("sine.svg").unwrap();
}

fn write_png(fig: &Figure) {
    let mut surf = PxlSurface::new(1200, 900);
    fig.draw(&mut surf).unwrap();
    surf.save("sine.png").unwrap();
}
