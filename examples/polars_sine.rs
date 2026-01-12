use std::f64::consts::PI;

use plotive::{des, style};
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

    let title = des::figure::Title::from("Sine wave from polars");

    let x_axis = des::Axis::new()
        .with_title("x".into())
        .with_ticks(Default::default());
    let y_axis = des::Axis::new()
        .with_title("y".into())
        .with_ticks(Default::default());

    let series = des::Series::Line(
        des::series::Line::new(des::data_src_ref("x"), des::data_src_ref("y"))
            .with_name("y=sin(x)")
            .with_line(style::series::Stroke::default().with_width(4.0)),
    );

    let plot = des::Plot::new(vec![series])
        .with_x_axis(x_axis)
        .with_y_axis(y_axis)
        .with_legend(des::plot::LegendPos::InTopRight.into());

    let fig = des::Figure::new(plot.into()).with_title(title);

    common::save_figure(&fig, &data_source, None, "polars-sine");
}
