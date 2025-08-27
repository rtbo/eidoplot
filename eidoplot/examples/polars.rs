use std::f64::consts::PI;

use eidoplot::ir;
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

    let title = ir::figure::Title {
        text: "Sine wave from polars".into(),
        font: ir::figure::TitleFont::default(),
    };

    let x_axis = ir::Axis::new(ir::axis::Scale::default()).with_label("x".into());
    let y_axis = ir::Axis::new(ir::axis::Scale::default()).with_label("y".into());

    let series = ir::Series {
        name: Some("y=sin(x)".into()),
        plot: ir::SeriesPlot::Line(ir::series::Line {
            line: 3.0.into(),
            x_data: ir::series::DataCol::SrcRef("x".to_string()),
            y_data: ir::series::DataCol::SrcRef("y".to_string()),
        }),
    };

    let plot = ir::Plot {
        title: None,
        x_axis,
        y_axis,
        series: vec![series],
        ..ir::Plot::default()
    };

    let fig = ir::Figure::new(ir::figure::Plots::Plot(plot)).with_title(Some(title));

    common::save_figure(&fig, &data_source);
}
