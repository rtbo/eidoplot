use eidoplot::{data, ir, style};

use std::f64::consts::PI;

mod common;

fn main() {
    let x: Vec<f64> = (0..=360).map(|t| t as f64 * PI / 180.0).collect();
    let y = x.iter().map(|x| x.sin()).collect();

    let data_source = data::VecMapSource::new()
        .with_col("x".into(), x)
        .with_col("y".into(), y);

    let title = ir::Text::new(
        "Sine wave",
        style::Font::new("serif".into(), 24.0)
            .with_style(style::font::Style::Italic)
            .with_weight(style::font::Weight::BOLD),
    );

    let x_axis = ir::Axis::new(ir::axis::Scale::default())
        .with_label("x".into())
        .with_ticks(ir::axis::ticks::Locator::PiMultiple { bins: 8 }.into());
    let y_axis = ir::Axis::new(ir::axis::Scale::default()).with_label("y".into());

    let series = ir::Series {
        name: Some("y=sin(x)".into()),
        plot: ir::SeriesPlot::Xy(ir::series::Xy {
            line: style::Line {
                color: style::color::BLUE,
                width: 3.0,
                pattern: style::LinePattern::Solid,
            },
            data: data::Xy::Src("x".to_string(), "y".to_string()),
        }),
    };

    let plot = ir::Plot {
        title: None,
        x_axis,
        y_axis,
        series: vec![series],
        legend: Some(ir::Legend::default().with_font(
            style::Font::new("Noto Sans Math".into(), 16.0).with_style(style::font::Style::Italic),
        )),
        ..ir::Plot::default()
    };

    let fig = ir::Figure::new(ir::figure::Plots::Plot(plot)).with_title(Some(title));

    common::save_figure(&fig, &data_source);
}
