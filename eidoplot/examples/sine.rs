use std::f64::consts::PI;

use eidoplot::{data, ir, style};

mod common;

fn main() {
    let title = "Sine wave".to_string().into();

    let x_axis = ir::Axis::new()
        .with_title("x".to_string().into())
        .with_ticks(ir::axis::ticks::Locator::PiMultiple { bins: 9 }.into());
    let y_axis = ir::Axis::new()
        .with_title("y".to_string().into())
        .with_minor_ticks(ir::axis::ticks::MinorTicks::default());

    let series = ir::Series::Line(
        ir::series::Line::new(
            ir::DataCol::SrcRef("x".to_string()),
            ir::DataCol::SrcRef("y".to_string()),
        )
        .with_name("y=sin(x)".to_string())
        .with_line(style::series::Line::default().with_width(4.0)),
    );

    let plot = ir::Plot::new(vec![series])
        .with_x_axis(x_axis)
        .with_y_axis(y_axis)
        .with_legend(Some(ir::plot::LegendPos::InTopRight.into()));

    let fig = ir::Figure::new(ir::figure::Plots::Plot(plot)).with_title(Some(title));

    let x: Vec<f64> = (0..=360).map(|t| t as f64 * PI / 180.0).collect();
    let y = x.iter().map(|x| x.sin()).collect();

    let data_source = data::TableSource::new()
        .with_f64_column("x".into(), x)
        .with_f64_column("y".into(), y);

    common::save_figure(&fig, &data_source);
}
