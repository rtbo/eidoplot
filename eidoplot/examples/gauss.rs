use std::f64::consts::PI;

use eidoplot::{data, ir, style};
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};

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

    let title: ir::figure::Title =
        format!("Normal distribution (\u{03bc}={}, \u{03c3}={})", MU, SIGMA).into();

    let x_axis = ir::Axis::new(ir::axis::Scale::default()).with_label("x".into());
    let y_axis = ir::Axis::new(ir::axis::Scale::default())
        .with_label("population density".into())
        .with_ticks(ir::axis::Ticks::default().with_formatter(ir::axis::ticks::Formatter::Percent));

    let dist_series = ir::Series {
        name: Some("distribution".into()),
        plot: ir::SeriesPlot::Xy(ir::series::Xy {
            line: style::Line {
                color: style::color::DARKSLATEBLUE,
                width: 4.0,
                pattern: style::LinePattern::Solid,
            },
            x_data: ir::series::DataCol::Inline(x.into()),
            y_data: ir::series::DataCol::Inline(y.into()),
        }),
    };
    let pop_series = ir::Series {
        name: Some("population".into()),
        plot: ir::SeriesPlot::Histogram(ir::series::Histogram {
            fill: style::color::CORNFLOWERBLUE.with_opacity(0.9).into(),
            line: None,
            bins: 16,
            density: true,
            data: ir::series::DataCol::SrcRef("pop".to_string()),
        }),
    };

    let plot = ir::Plot {
        title: None,
        x_axis,
        y_axis,
        series: vec![pop_series, dist_series],
        legend: Some(ir::Legend::new(ir::legend::Pos::OutRight)),
        ..ir::Plot::default()
    };

    let fig = ir::Figure::new(ir::figure::Plots::Plot(plot)).with_title(Some(title));

    let data_source = data::VecSource::new().with_f64_column("pop".into(), pop);

    common::save_figure(&fig, &data_source);
}

fn predictable_rng(seed: Option<u64>) -> impl rand::Rng {
    let seed = seed.unwrap_or(586350478348);
    rand_chacha::ChaCha8Rng::seed_from_u64(seed)
}
