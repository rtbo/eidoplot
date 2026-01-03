use plotive::{data, ir, text, utils};

mod common;

use std::f64::consts::PI;

fn main() {
    let x1 = utils::linspace(0.0, 2.0 * PI, 400);
    let y1: Vec<f64> = x1.iter().map(|x| (x * x).sin()).collect();
    let x2 = utils::linspace(0.5 * PI, 2.5 * PI, 400);
    let y2: Vec<f64> = x1.iter().map(|x| -(x * x).sin()).collect();

    let mut data_source = data::NamedColumns::new();
    data_source.add_column("x1", &x1 as &dyn data::Column);
    data_source.add_column("y1", &y1 as &dyn data::Column);
    data_source.add_column("x2", &x2 as &dyn data::Column);
    data_source.add_column("y2", &y2 as &dyn data::Column);

    let ax_x1 = ir::Axis::new()
        .with_grid(Default::default())
        .with_scale(ir::axis::ref_id("x2").into());
    let ax_y1 = ir::Axis::new().with_ticks(Default::default());
    let ax_x2 = ir::Axis::new()
        .with_id("x2")
        .with_ticks(
            ir::axis::Ticks::new()
                .with_locator(ir::axis::ticks::PiMultipleLocator::default().into()),
        )
        .with_grid(Default::default());
    let ax_y2 = ir::Axis::new().with_ticks(Default::default());

    let series1 = ir::series::Line::new(ir::data_src_ref("x1"), ir::data_src_ref("y1")).into();
    let series2 = ir::series::Line::new(ir::data_src_ref("x2"), ir::data_src_ref("y2")).into();

    let plot1 = ir::Plot::new(vec![series1])
        .with_x_axis(ax_x1)
        .with_y_axis(ax_y1);
    let plot2 = ir::Plot::new(vec![series2])
        .with_x_axis(ax_x2)
        .with_y_axis(ax_y2);

    let subplots = ir::Subplots::new(2, 1)
        .with_plot((0, 0), plot1)
        .with_plot((1, 0), plot2)
        .with_space(10.0);

    let fig = ir::Figure::new(subplots.into()).with_title("Subplots".into());

    let mut fontdb = text::fontdb::Database::new();
    fontdb.load_system_fonts();

    common::save_figure(&fig, &data_source, Some(&fontdb), "subplots");
}
