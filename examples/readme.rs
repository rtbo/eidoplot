// example from README.md
use std::f64::consts::PI;
use std::sync::Arc;

use plotive::{data, des};
use plotive_iced::Show;

fn main() {
    let x_axis = des::Axis::new()
        .with_title("x".into())
        .with_ticks(
            des::axis::Ticks::new()
                .with_locator(des::axis::ticks::PiMultipleLocator::default().into()),
        )
        .with_grid(Default::default());

    let y_axis = des::Axis::new()
        .with_title("y".into())
        .with_ticks(Default::default())
        .with_grid(Default::default())
        .with_minor_ticks(Default::default())
        .with_minor_grid(Default::default());

    let series = des::series::Line::new(des::data_src_ref("x"), des::data_src_ref("y"))
        .with_name("y=sin(x)")
        .into();

    let plot = des::Plot::new(vec![series])
        .with_x_axis(x_axis)
        .with_y_axis(y_axis)
        .with_legend(des::plot::LegendPos::InTopRight.into());

    let fig = des::Figure::new(plot.into()).with_title("a sine wave".into());

    let x: Vec<f64> = (0..=360).map(|t| t as f64 * PI / 180.0).collect();
    let y = x.iter().map(|x| x.sin()).collect();

    let data_source = data::TableSource::new()
        .with_f64_column("x", x)
        .with_f64_column("y", y);

    fig.show(Arc::new(data_source), Default::default()).unwrap();
}
