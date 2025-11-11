use std::f64::consts::PI;

use eidoplot::{data, ir};
use eidoplot_utils as utils;

mod common;

fn main() {
    let x = utils::linspace(0.0, 6.0 * PI, 500);
    let sin_x = x.iter().map(|x| x.sin()).collect::<Vec<f64>>();
    let exp_x = x.iter().map(|x| x.exp()).collect::<Vec<f64>>();

    let mut data_src = data::NamedColumns::new();
    data_src.add_column("x", &x as &dyn data::Column);
    data_src.add_column("sin(x)", &sin_x as &dyn data::Column);
    data_src.add_column("exp(x)", &exp_x as &dyn data::Column);

    let title = "Multiple axes".to_string();

    let x_axis = ir::Axis::new()
        .with_title("x".to_string().into())
        .with_ticks(ir::axis::ticks::Locator::PiMultiple { bins: 9 }.into());
    let y1_axis = ir::Axis::new()
        .with_title("sin(x)".to_string().into())
        .with_ticks(Default::default());
    let y2_axis = ir::Axis::new()
        .with_title("exp(x)".to_string().into())
        .with_scale(ir::axis::LogScale::default().into())
        .with_ticks(Default::default());

    let series1 = ir::series::Line::new(
        ir::DataCol::SrcRef("x".to_string()),
        ir::DataCol::SrcRef("sin(x)".to_string()),
    )
    .with_name("sin(x)".to_string())
    .into();
    let series2 = ir::series::Line::new(
        ir::DataCol::SrcRef("x".to_string()),
        ir::DataCol::SrcRef("exp(x)".to_string()),
    )
    .with_name("exp(x)".to_string())
    .with_y_axis(ir::axis::Ref::Id("exp(x)".to_string()))
    .into();

    let plot = ir::Plot::new(vec![series1, series2])
        .with_x_axis(x_axis)
        .with_y_axis(y1_axis)
        .with_y_axis(y2_axis);
    let fig = ir::Figure::new(plot.into())
        .with_title(title.into())
        .with_legend(Default::default());

    common::save_figure(&fig, &data_src, "multiple_axes");
}
