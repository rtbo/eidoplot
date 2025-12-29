use std::f64::consts::PI;

use eidoplot::{data, eplt, utils};

mod common;

fn main() {
    let x = utils::linspace(0.0, 6.0 * PI, 500);
    let sin_x = x.iter().map(|x| x.sin()).collect::<Vec<f64>>();
    let exp_x = x.iter().map(|x| x.exp()).collect::<Vec<f64>>();

    let mut data_src = data::NamedColumns::new();
    data_src.add_column("x", &x as &dyn data::Column);
    data_src.add_column("sin(x)", &sin_x as &dyn data::Column);
    data_src.add_column("exp(x)", &exp_x as &dyn data::Column);

    let filename = common::example_res("multiple_axes.eplt");
    let content = std::fs::read_to_string(&filename).unwrap();
    let figs = eplt::parse_diag(&content, Some(&filename)).unwrap();

    common::save_figure(&figs[0], &data_src, None, "multiple_axes_eplt");
}
