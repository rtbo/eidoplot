use eidoplot::{data, eplt, utils};

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

    let filename = common::example_res("subplots.eplt");
    let content = std::fs::read_to_string(&filename).unwrap();
    let figs = eplt::parse_diag(&content, Some(&filename)).unwrap();

    common::save_figure(&figs[0], &data_source, "subplots_eplt");
}
