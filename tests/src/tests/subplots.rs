use plotive::des;

use super::{fig_high, fig_wide, line};
use crate::{TestHarness, assert_fig_eq_ref};

#[test]
fn subplots_default() {
    let plot1 = des::Plot::new(vec![line().into()]);
    let plot2 = des::Plot::new(vec![line().into()]);
    let subplots = des::Subplots::new(2, 1)
        .with_plot((0, 0), plot1)
        .with_plot((1, 0), plot2);

    let fig = fig_high(subplots);
    assert_fig_eq_ref!(&fig, "subplots/default");
}

#[test]
fn subplots_space10() {
    let plot1 = des::Plot::new(vec![line().into()]);
    let plot2 = des::Plot::new(vec![line().into()]);
    let subplots = des::Subplots::new(2, 1)
        .with_plot((0, 0), plot1)
        .with_plot((1, 0), plot2)
        .with_space(10.0);

    let fig = fig_high(subplots);
    assert_fig_eq_ref!(&fig, "subplots/space10");
}

#[test]
fn subplots_sharedx() {
    let plot1 = des::Plot::new(vec![line().into()]).with_x_axis(
        des::Axis::new()
            .with_ticks(Default::default())
            .with_scale(des::axis::Ref::Id("x".to_string()).into()),
    );
    let plot2 = des::Plot::new(vec![line().into()]).with_x_axis(
        des::Axis::new()
            .with_id("x".to_string())
            .with_ticks(Default::default()),
    );

    let subplots = des::Subplots::new(2, 1)
        .with_plot((0, 0), plot1)
        .with_plot((1, 0), plot2);

    let fig = fig_high(subplots);
    assert_fig_eq_ref!(&fig, "subplots/sharedx");
}

#[test]
fn subplots_sharedy() {
    let plot1 = des::Plot::new(vec![line().into()]).with_y_axis(
        des::Axis::new()
            .with_id("y".to_string())
            .with_ticks(Default::default()),
    );
    let plot2 = des::Plot::new(vec![line().into()]).with_y_axis(
        des::Axis::new()
            .with_ticks(Default::default())
            .with_scale(des::axis::Ref::Id("y".to_string()).into()),
    );
    let subplots = des::Subplots::new(1, 2)
        .with_plot((0, 0), plot1)
        .with_plot((0, 1), plot2);

    let fig = fig_wide(subplots);
    assert_fig_eq_ref!(&fig, "subplots/sharedy");
}
