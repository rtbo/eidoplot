use eidoplot::{
    ir,
    time::{self, DateTime},
};
use eidoplot_utils::timespace;
use rand_distr::{Distribution, Normal};

mod common;

fn main() {
    const MEAN: f64 = 2.0;
    const STD_DEV: f64 = 0.05;

    let mut rng = common::predictable_rng(None);
    let normal = Normal::new(MEAN, STD_DEV).unwrap();

    let x = timespace(
        DateTime::fmt_parse("2025-01-01", "%Y-%m-%d").unwrap(),
        DateTime::fmt_parse("2026-01-01", "%Y-%m-%d").unwrap(),
        366,
    );

    let y = x
        .iter()
        .map(|_| normal.sample(&mut rng))
        .collect::<Vec<f64>>();

    // let mut data_source = data::NamedColumns::new();
    // data_source.add_column("x", &x as &dyn data::Column);
    // data_source.add_column("y", &y as &dyn data::Column);

    // let series = ir::series::Line::new(
    //     ir::DataCol::SrcRef("x".to_string()),
    //     ir::DataCol::SrcRef("y".to_string()),
    // );
    // let x_axis = ir::Axis::new().with_ticks(Default::default());
    // let plot = ir::Plot::new(vec![series.into()]).with_x_axis(x_axis);
    // let fig = ir::Figure::new(plot.into());

    // common::save_figure(&fig, &data_source, "time_series");
}
