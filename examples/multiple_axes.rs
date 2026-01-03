use std::f64::consts::PI;

use plotive::{data, des, utils};

mod common;

fn main() {
    let x = utils::linspace(0.0, 6.0 * PI, 500);
    let sin_x = x.iter().map(|x| x.sin()).collect::<Vec<f64>>();
    let exp_x = x.iter().map(|x| x.exp()).collect::<Vec<f64>>();

    let mut data_src = data::NamedColumns::new();
    data_src.add_column("x", &x as &dyn data::Column);
    data_src.add_column("sin(x)", &sin_x as &dyn data::Column);
    data_src.add_column("exp(x)", &exp_x as &dyn data::Column);

    let x_axis = des::Axis::new().with_title("x".into()).with_ticks(
        des::axis::Ticks::new().with_locator(des::axis::ticks::PiMultipleLocator::default().into()),
    );
    let y1_axis = des::Axis::new()
        .with_title("sin(x)".into())
        .with_ticks(Default::default());
    let y2_axis = des::Axis::new()
        .with_title("exp(x)".into())
        .with_scale(des::axis::LogScale::default().into())
        .with_ticks(Default::default());

    let series1 = des::series::Line::new(des::data_src_ref("x"), des::data_src_ref("sin(x)"))
        .with_name("sin(x)")
        .into();
    let series2 = des::series::Line::new(des::data_src_ref("x"), des::data_src_ref("exp(x)"))
        .with_name("exp(x)")
        .with_y_axis(des::axis::ref_id("exp(x)"))
        .into();

    let plot = des::Plot::new(vec![series1, series2])
        .with_border(des::plot::AxisArrow::default().into())
        .with_x_axis(x_axis)
        .with_y_axis(y1_axis)
        .with_y_axis(y2_axis);
    let fig = des::Figure::new(plot.into())
        .with_title("Multiple axes".into())
        .with_legend(Default::default());

    common::save_figure(&fig, &data_src, None, "multiple_axes");
}
