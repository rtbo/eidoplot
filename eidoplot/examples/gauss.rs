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

    let x_axis = ir::Axis::new()
        .with_title("x".to_string().into())
        .with_ticks(Default::default());
    let y_axis = ir::Axis::new()
        .with_title("y".to_string().into())
        .with_ticks(
            ir::axis::Ticks::new().with_formatter(Some(ir::axis::ticks::Formatter::Percent)),
        );

    let pop_series = ir::Series::Histogram(
        ir::series::Histogram::new(ir::DataCol::SrcRef("pop".to_string()))
            .with_name("population".into())
            .with_fill(style::series::Fill::Solid {
                color: style::series::Color::Auto,
                opacity: Some(0.7),
            })
            .with_bins(16)
            .with_density(),
    );

    let dist_series = ir::Series::Line(
        ir::series::Line::new(x.into(), y.into())
            .with_name("distribution".into())
            .with_line(style::series::Line {
                width: 4.0,
                ..style::Line::default()
            }),
    );

    let series = vec![dist_series, pop_series];

    let plot = ir::Plot::new(series)
        .with_x_axis(x_axis)
        .with_y_axis(y_axis)
        .with_legend(ir::plot::LegendPos::OutRight.into());

    let fig = ir::Figure::new(plot.into()).with_title(title);

    let data_source = data::TableSource::new().with_f64_column("pop".into(), pop);

    common::save_figure(&fig, &data_source, "gauss");
}

fn predictable_rng(seed: Option<u64>) -> impl rand::Rng {
    let seed = seed.unwrap_or(586350478348);
    rand_chacha::ChaCha8Rng::seed_from_u64(seed)
}
