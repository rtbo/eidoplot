use eidoplot::{data, ir, style};

use rand::SeedableRng;
use rand_distr::{Distribution, Normal};

use std::f64::consts::PI;

mod common;

fn main() {
    const MU: f64 = 13.0;
    const SIGMA: f64 = 2.0;
    const N_DIST: usize = 100;
    const N_POP: usize = 1000;

    const X_MIN: f64 = MU - 4.0 * SIGMA;
    const X_MAX: f64 = MU + 4.0 * SIGMA;

    let x = (0..N_DIST)
        .map(|i| X_MIN + (i as f64 * (X_MAX - X_MIN) / (N_DIST - 1) as f64))
        .collect::<Vec<f64>>();
    let y = x
        .iter()
        .map(|&x| 1.0 / (SIGMA * (2.0 * PI).sqrt()) * (-0.5 * ((x - MU) / SIGMA).powf(2.0)).exp())
        .collect::<Vec<f64>>();

    let mut rng = predictable_rng(None);
    let normal = Normal::new(MU, SIGMA).unwrap();
    let pop = (0..N_POP).map(|_| normal.sample(&mut rng)).collect();

    let data_source = data::VecMapSource::new().with_col("pop".into(), pop);

    let title = ir::Text::new(
        format!("Normal distribution (\u{03bc}={}, \u{03c3}={})", MU, SIGMA),
        style::Font::new("serif".into(), 24.0)
            .with_style(style::font::Style::Italic)
            .with_weight(style::font::Weight::BOLD),
    );

    let x_axis = ir::Axis::new(ir::axis::Scale::default()).with_label("x".into());
    let y_axis = ir::Axis::new(ir::axis::Scale::default()).with_label("population density".into());

    let dist_series = ir::Series {
        name: Some("distribution".into()),
        plot: ir::SeriesPlot::Xy(ir::series::Xy {
            line: style::Line {
                color: style::color::DARKSLATEBLUE,
                width: 4.0,
                pattern: style::LinePattern::Solid,
            },
            data: data::Xy::Inline(x, y),
        }),
    };
    let pop_series = ir::Series {
        name: Some("population".into()),
        plot: ir::SeriesPlot::Histogram(ir::series::Histogram {
            fill: style::color::CORNFLOWERBLUE.with_opacity(0.9).into(),
            line: None,
            bins: 16,
            density: true,
            data: data::X::Src("pop".into()),
        }),
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

    common::save_figure(&fig, &data_source);
}

fn predictable_rng(seed: Option<u64>) -> impl rand::Rng {
    let seed = seed.unwrap_or(586350478348);
    rand_chacha::ChaCha8Rng::seed_from_u64(seed)
}
