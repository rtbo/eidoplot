use eidoplot::prelude::*;
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;

use std::{env, f64::consts::PI};

fn main() {
    let points = (0..=360)
        .map(|t| (t as f64 * PI / 180.0, (t as f64 * PI / 180.0).sin()))
        .collect();

    let fig = Figure {
        title: Some("Sine wave".into()),
        plots: Some(Plots::Plot(Plot {
            title: None,
            x_axis: axis::Axis {
                name: Some("x".into()),
                ticks: Some(tick::Locator::PiMultiple { bins: 8 }.into()),
                ..axis::Axis::default()
            },
            y_axis: axis::Axis {
                name: Some("y".into()),
                scale: axis::Scale::Linear(scale::Range::MinMax(-0.8, 0.8)),
                ..axis::Axis::default()
            },
            series: vec![Series {
                name: Some("y=sin(x)".into()),
                plot: SeriesPlot::Xy(XySeries {
                    line: style::Line {
                        color: color::BLUE,
                        width: 3.0,
                        pattern: style::LinePattern::Solid,
                    },
                    points,
                }),
            }],
            ..Plot::default()
        })),
        ..Figure::default()
    };

    let mut written = false;

    for arg in env::args() {
        if arg == "png" {
            write_png(&fig);
            written = true;
        } else if arg == "svg" {
            write_svg(&fig);
            written = true;
        }
    }
    if !written {
        write_svg(&fig);
    }
}

fn write_svg(fig: &Figure) {
    let mut svg = SvgSurface::new(800, 600);
    fig.draw(&mut svg).unwrap();
    svg.save("plot.svg").unwrap();
}

fn write_png(fig: &Figure) {
    let mut surf = PxlSurface::new(1600, 1200);
    fig.draw(&mut surf).unwrap();
    surf.save("plot.png").unwrap();
}
