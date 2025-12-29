use std::f64::consts::PI;

use eidoplot::{data, ir, style};

mod common;

fn main() {
    let x_axis = ir::Axis::new()
        .with_title("x".into())
        .with_ticks(
            ir::axis::Ticks::new()
                .with_locator(ir::axis::ticks::PiMultipleLocator::default().into()),
        )
        .with_grid(Default::default());

    let y_axis = ir::Axis::new()
        .with_title("y".into())
        .with_ticks(Default::default())
        .with_grid(Default::default())
        .with_minor_ticks(Default::default())
        .with_minor_grid(Default::default());

    let series = ir::Series::Line(
        ir::series::Line::new(ir::data_src_ref("x"), ir::data_src_ref("y"))
            .with_name("y=sin(x)")
            .with_line(style::series::Line::default().with_width(4.0)),
    );

    let plot = ir::Plot::new(vec![series])
        .with_x_axis(x_axis)
        .with_y_axis(y_axis)
        .with_border(ir::plot::AxisArrow::default().into())
        .with_legend(ir::plot::LegendPos::InTopRight.into());

    let fig = ir::Figure::new(plot.into()).with_title("Sine wave".into());

    let x: Vec<f64> = (0..=360).map(|t| t as f64 * PI / 180.0).collect();
    let y = x.iter().map(|x| x.sin()).collect();

    let data_source = data::TableSource::new()
        .with_f64_column("x".into(), x)
        .with_f64_column("y".into(), y);

    common::save_figure(&fig, &data_source, None, "sine");
}
