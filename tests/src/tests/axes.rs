use eidoplot::ir;

use crate::{assert_fig_eq_ref, TestHarness};
use super::{line, fig_small};


#[test]
fn axes_default() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series]);
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/default");
}

#[test]
fn axes_y_major_ticks() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series])
        .with_y_axis(ir::axis::Axis::default().with_ticks(Default::default()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/y-major-ticks");
}

#[test]
fn axes_major_ticks() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series])
        .with_x_axis(ir::axis::Axis::default().with_ticks(Default::default()))
        .with_y_axis(ir::axis::Axis::default().with_ticks(Default::default()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/major-ticks");
}