use eidoplot::{data, ir};
use eidoplot_utils as utils;

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

    let title = "Subplots".to_string();

    let ax_x1 = ir::Axis::new()
        .with_grid(Default::default())
        .with_scale(ir::axis::Scale::Shared(ir::axis::Ref::Id("x2".to_string())));
    let ax_y1 = ir::Axis::new().with_ticks(Default::default());
    let ax_x2 = ir::Axis::new()
        .with_id("x2".to_string())
        .with_ticks(ir::axis::ticks::Locator::PiMultiple { bins: 9 }.into())
        .with_grid(Default::default());
    let ax_y2 = ir::Axis::new().with_ticks(Default::default());

    let series1 = ir::series::Line::new(
        ir::DataCol::SrcRef("x1".to_string()),
        ir::DataCol::SrcRef("y1".to_string()),
    )
    .into();
    let series2 = ir::series::Line::new(
        ir::DataCol::SrcRef("x2".to_string()),
        ir::DataCol::SrcRef("y2".to_string()),
    )
    .into();

    let plot1 = ir::Plot::new(vec![series1])
        .with_x_axis(ax_x1)
        .with_y_axis(ax_y1);
    let plot2 = ir::Plot::new(vec![series2])
        .with_x_axis(ax_x2)
        .with_y_axis(ax_y2);

    let subplots = ir::Subplots::new(2, 1)
        .with_plot(0, 0, plot1)
        .with_plot(1, 0, plot2)
        .with_space(10.0);

    let fig = ir::Figure::new(subplots.into()).with_title(title.into());

    common::save_figure(&fig, &data_source, "subplots");
}
