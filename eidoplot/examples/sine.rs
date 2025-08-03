use eidoplot::prelude::*;
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;

use std::{env, f64::consts::PI};

fn main() {
    let points = (0..=360)
        .map(|t| (t as f64 * PI / 180.0, (t as f64 * PI / 180.0).sin()))
        .collect();

    let fig = Figure {
        size: FigSize::default(),
        title: Some("Sine wave".into()),
        padding: 20.0.into(),
        plots: Some(Plots::Plot(Plot {
            title: None,
            x_axis: axis::Axis {
                name: Some("x".into()),
                ..axis::Axis::default()
            },
            y_axis: axis::Axis {
                name: Some("y".into()),
                ..axis::Axis::default()
            },
            series: vec![Series {
                name: Some("y=sin(x)".into()),
                plot: SeriesPlot::Xy(XySeries {
                    line: style::Line {
                        color: css::FUCHSIA,
                        width: 1.5,
                        pattern: style::LinePattern::Dash(5.0, 5.0),
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
