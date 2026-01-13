use plotive::des;

use crate::tests::fig_small;
use crate::{TestHarness, assert_fig_eq_ref};

fn line() -> des::series::Line {
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let y = vec![0.0, 2.0, 3.0, 1.0, 4.0, 4.0];
    des::series::Line::new(des::data_inline(x), des::data_inline(y)).into()
}

#[test]
fn interp_linear() {
    let plot = line()
        .with_interpolation(des::series::Interpolation::Linear)
        .into_plot();
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "interp/linear");
}

#[test]
fn interp_step_early() {
    let plot = line()
        .with_interpolation(des::series::Interpolation::StepEarly)
        .into_plot();
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "interp/step-early");
}

#[test]
fn interp_step_middle() {
    let plot = line()
        .with_interpolation(des::series::Interpolation::StepMiddle)
        .into_plot();
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "interp/step-middle");
}

#[test]
fn interp_step_late() {
    let plot = line()
        .with_interpolation(des::series::Interpolation::StepLate)
        .into_plot();
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "interp/step-late");
}

#[test]
fn interp_spline() {
    let plot = line()
        .with_interpolation(des::series::Interpolation::Spline)
        .into_plot();
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "interp/spline");
}
