use eidoplot::drawing::{self, FigureExt};
use eidoplot::{ir, style};
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;

use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Normal};

use std::sync::Arc;
use std::{env, f64::consts::PI};

fn main() {
    const MU: f64 = 13.0;
    const SIGMA: f64 = 2.0;
    const N: usize = 100;
    const X_MIN: f64 = MU - 4.0 * SIGMA;
    const X_MAX: f64 = MU + 4.0 * SIGMA;
    let x = (0..N)
        .map(|i| X_MIN + (i as f64 * (X_MAX - X_MIN) / (N - 1) as f64))
        .collect::<Vec<f64>>();
    let y = x
        .iter()
        .map(|&x| 1.0 / (SIGMA * (2.0 * PI).sqrt()) * (-0.5 * ((x - MU) / SIGMA).powf(2.0)).exp())
        .collect::<Vec<f64>>();

    let points = x.iter().zip(y.iter()).map(|(&x, &y)| (x, y)).collect();

    let mut rng = ThreadRng::from_seed(0);
    let normal = Normal::new(MU, SIGMA).unwrap();
    let pop = (0..N).map(|_| normal.sample(&mut rng)).collect();

    let title = ir::Text::new(
        "Normal distribution",
        style::Font::new("serif".into(), 24.0)
            .with_style(style::font::Style::Italic)
            .with_weight(style::font::Weight::BOLD),
    );

    let x_axis = ir::Axis::new(ir::axis::Scale::Linear(ir::axis::Range::MinMax(
        X_MIN, X_MAX,
    )))
    .with_label("x".into());
    let y_axis = ir::Axis::new(ir::axis::Scale::default()).with_label("y".into());

    let dist_series = ir::plot::Series {
        name: Some("distribution".into()),
        plot: ir::plot::SeriesPlot::Xy(ir::plot::XySeries {
            line: style::Line {
                color: style::color::BLUE,
                width: 3.0,
                pattern: style::LinePattern::Solid,
            },
            points,
        }),
    };
    let pop_series = ir::plot::Series {
        name: Some("population".into()),
        plot: ir::plot::SeriesPlot::Histogram(ir::plot::HistogramSeries {
            fill: style::color::PURPLE.into(),
            line: Some(style::Line {
                color: style::color::RED,
                width: 3.0,
                pattern: style::LinePattern::Solid,
            }),
            bins: 10,
            density: true,
            points: pop,
        })
    };

    let plot = ir::Plot {
        title: None,
        x_axis,
        y_axis,
        series: vec![pop_series, dist_series],
        legend: Some(ir::Legend::default().with_font(
            style::Font::new("Noto Sans Math".into(), 16.0).with_style(style::font::Style::Italic),
        )),
        ..ir::Plot::default()
    };

    let fig = ir::Figure::new(ir::figure::Plots::Plot(plot)).with_title(Some(title));

    save_figure(&fig);
}

fn save_figure(fig: &ir::Figure) {
    let fontdb = eidoplot::bundled_font_db();

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
