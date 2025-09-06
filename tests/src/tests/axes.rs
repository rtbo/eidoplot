use eidoplot::ir;

use super::{fig_small, line};
use crate::{TestHarness, assert_fig_eq_ref};

#[test]
fn axes_default() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series]);
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/default");
}

#[test]
fn axes_x_title() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series])
        .with_x_axis(ir::axis::Axis::default().with_title("x axis".to_string().into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/x-title");
}

#[test]
fn axes_titles() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series])
        .with_x_axis(ir::Axis::new().with_title("x axis".to_string().into()))
        .with_y_axis(ir::Axis::new().with_title("y axis".to_string().into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/titles");
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
fn axes_y_title_major_ticks() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series]).with_y_axis(
        ir::axis::Axis::new()
            .with_title("y axis".to_string().into())
            .with_ticks(Default::default()),
    );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/y-title-major-ticks");
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

#[test]
fn axes_minor_ticks() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series])
        .with_x_axis(
            ir::axis::Axis::new()
                .with_ticks(Default::default())
                .with_minor_ticks(Default::default()),
        )
        .with_y_axis(
            ir::axis::Axis::new()
                .with_ticks(Default::default())
                .with_minor_ticks(Default::default()),
        );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/minor-ticks");
}

#[test]
fn axes_y_major_grid() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series]).with_y_axis(
        ir::axis::Axis::new().with_ticks(ir::axis::Ticks::new().with_grid(Default::default())),
    );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/y-major-grid");
}

#[test]
fn axes_major_grid() {
    let series = line().into();
    let axis =
        ir::axis::Axis::new().with_ticks(ir::axis::Ticks::new().with_grid(Default::default()));
    let plot = ir::Plot::new(vec![series])
        .with_x_axis(axis.clone())
        .with_y_axis(axis);
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/major-grid");
}

#[test]
fn axes_minor_grid() {
    let series = line().into();
    let axis = ir::axis::Axis::new()
        .with_ticks(ir::axis::Ticks::new().with_grid(Default::default()))
        .with_minor_ticks(ir::axis::MinorTicks::new().with_grid(Default::default()));

    let plot = ir::Plot::new(vec![series])
        .with_x_axis(axis.clone())
        .with_y_axis(axis);
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/minor-grid");
}
