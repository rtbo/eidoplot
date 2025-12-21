use std::f64::consts::PI;

use eidoplot::{ir, style};
use polars::prelude::*;

mod common;

fn main() {
    let x: Vec<f64> = (0..=360).map(|t| t as f64 * PI / 180.0).collect();
    let y: Vec<f64> = x.iter().map(|x| x.sin()).collect();

    let data_source: DataFrame = df!(
        "x" => &x,
        "y" => &y,
    )
    .unwrap();

    let title = ir::figure::Title::from("Sine wave from polars");

    let x_axis = ir::Axis::new()
        .with_title("x".into())
        .with_ticks(Default::default());
    let y_axis = ir::Axis::new()
        .with_title("y".into())
        .with_ticks(Default::default());

    let series = ir::Series::Line(
        ir::series::Line::new(ir::data_src_ref("x"), ir::data_src_ref("y"))
            .with_name("y=sin(x)")
            .with_line(style::series::Line::default().with_width(4.0)),
    );

    let plot = ir::Plot::new(vec![series])
        .with_x_axis(x_axis)
        .with_y_axis(y_axis)
        .with_legend(ir::plot::LegendPos::InTopRight.into());

    let fig = ir::Figure::new(plot.into()).with_title(title);

    common::save_figure(&fig, &data_source, "polars-sine");
}
