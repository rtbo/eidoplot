use eidoplot::{geom, ir};

use crate::*;

fn fig_small(plot: ir::Plot) -> ir::Figure {
    ir::Figure::new(ir::figure::Plots::Plot(plot)).with_size(geom::Size::new(400.0, 300.0))
}

fn line() -> ir::series::Line {
    let x = vec![1.0, 2.0, 3.0];
    let y = x.clone();
    ir::series::Line::new(x.into(), y.into())
}

mod axes;
mod legend;

#[test]
fn empty() {
    let plot = ir::Plot::new(vec![]);
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "empty");
}

#[test]
fn empty_title() {
    let plot = ir::Plot::new(vec![]);
    let fig = fig_small(plot).with_title("Title".to_string().into());

    assert_fig_eq_ref!(&fig, "empty-title");
}