use eidoplot::{ir, style};

use super::{fig_small, line, line2};
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
        .with_x_axis(ir::Axis::default().with_title("x axis".to_string().into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/x-title");
}

#[test]
fn axes_y_title() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series])
        .with_y_axis(ir::Axis::default().with_title("y axis".to_string().into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/y-title");
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
fn axes_x_major_ticks() {
    let series = line().into();
    let plot =
        ir::Plot::new(vec![series]).with_x_axis(ir::Axis::default().with_ticks(Default::default()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/x-major-ticks");
}

#[test]
fn axes_y_major_ticks() {
    let series = line().into();
    let plot =
        ir::Plot::new(vec![series]).with_y_axis(ir::Axis::default().with_ticks(Default::default()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/y-major-ticks");
}

#[test]
fn axes_major_ticks() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series])
        .with_x_axis(ir::Axis::default().with_ticks(Default::default()))
        .with_y_axis(ir::Axis::default().with_ticks(Default::default()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/major-ticks");
}

#[test]
fn axes_x_title_major_ticks() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series]).with_x_axis(
        ir::Axis::new()
            .with_title("x axis".to_string().into())
            .with_ticks(Default::default()),
    );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/x-title-major-ticks");
}

#[test]
fn axes_y_title_major_ticks() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series]).with_y_axis(
        ir::Axis::new()
            .with_title("y axis".to_string().into())
            .with_ticks(Default::default()),
    );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/y-title-major-ticks");
}

#[test]
fn axes_titles_major_ticks() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series])
        .with_x_axis(
            ir::Axis::new()
                .with_title("x axis".to_string().into())
                .with_ticks(Default::default()),
        )
        .with_y_axis(
            ir::Axis::new()
                .with_title("y axis".to_string().into())
                .with_ticks(Default::default()),
        );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/titles-major-ticks");
}

#[test]
fn axes_minor_ticks() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series])
        .with_x_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
                .with_minor_ticks(Default::default()),
        )
        .with_y_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
                .with_minor_ticks(Default::default()),
        );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/minor-ticks");
}

#[test]
fn axes_x_major_grid() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series]).with_x_axis(
        ir::Axis::new()
            .with_ticks(Default::default())
            .with_grid(Default::default()),
    );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/x-major-grid");
}

#[test]
fn axes_y_major_grid() {
    let series = line().into();
    let plot = ir::Plot::new(vec![series]).with_y_axis(
        ir::Axis::new()
            .with_ticks(Default::default())
            .with_grid(Default::default()),
    );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/y-major-grid");
}

#[test]
fn axes_major_grid() {
    let series = line().into();
    let axis = ir::Axis::new()
        .with_ticks(Default::default())
        .with_grid(Default::default());
    let plot = ir::Plot::new(vec![series])
        .with_x_axis(axis.clone())
        .with_y_axis(axis);
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/major-grid");
}

#[test]
fn axes_minor_grid() {
    let series = line().into();
    let axis = ir::Axis::new()
        .with_ticks(Default::default())
        .with_grid(Default::default())
        .with_minor_ticks(Default::default())
        .with_minor_grid(Default::default());

    let plot = ir::Plot::new(vec![series])
        .with_x_axis(axis.clone())
        .with_y_axis(axis);
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/minor-grid");
}

#[test]
fn axes_categories() {
    let x = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let y = vec![1.0, 1.4, 3.0];
    let series = ir::series::Bars::new(x.into(), y.into())
        .with_fill(style::color::TRANSPARENT.into())
        .with_line(Default::default());

    let plot = ir::Plot::new(vec![series.into()])
        .with_x_axis(ir::Axis::new().with_ticks(Default::default()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/categories");
}

#[test]
fn axes_pi_locator() {
    use std::f64::consts::PI;
    let x = vec![PI, 2.0 * PI, 3.0 * PI];
    let y = vec![1.0, 1.4, 3.0];
    let series = ir::series::Line::new(x.into(), y.into());

    let plot = ir::Plot::new(vec![series.into()]).with_x_axis(ir::Axis::new().with_ticks(
        ir::axis::Ticks::new().with_locator(ir::axis::ticks::Locator::PiMultiple { bins: 5 }),
    ));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/pi-locator");
}

#[test]
fn axes_pi_locator_minor() {
    use std::f64::consts::PI;
    let x = vec![PI, 2.0 * PI, 3.0 * PI];
    let y = vec![1.0, 1.4, 3.0];
    let series = ir::series::Line::new(x.into(), y.into());

    let plot = ir::Plot::new(vec![series.into()]).with_x_axis(
        ir::Axis::new()
            .with_ticks(
                ir::axis::Ticks::new()
                    .with_locator(ir::axis::ticks::Locator::PiMultiple { bins: 5 }),
            )
            .with_minor_ticks(
                ir::axis::MinorTicks::new()
                    .with_locator(ir::axis::ticks::Locator::PiMultiple { bins: 30 }),
            ),
    );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/pi-locator-minor");
}

#[test]
fn axes_top_right() {
    let s1 = line();
    let plot = ir::Plot::new(vec![s1.into()])
        .with_x_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
                .with_opposite_side()
        )
        .with_y_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
                .with_opposite_side()
        );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/top-right");
}

#[test]
fn axes_multiple_bl() {
    let s1 = line();
    let s2 = line2(&[4.0, 5.0, 6.0], &[6.0, 5.0, 4.0])
        .with_x_axis(ir::axis::Ref::Id("x2".to_string()))
        .with_y_axis(ir::axis::Ref::Id("y2".to_string()));
    let plot = ir::Plot::new(vec![s1.into(), s2.into()])
        .with_x_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
        )
        .with_y_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
        )
        .with_x_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
                .with_id("x2".to_string().into())
        )
        .with_y_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
                .with_id("y2".to_string().into())
        );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/multiple-bl");
}

#[test]
fn axes_multiple_trbl() {
    let s1 = line();
    let s2 = line2(&[4.0, 5.0, 6.0], &[6.0, 5.0, 4.0])
        .with_x_axis(ir::axis::Ref::Id("x2".to_string()))
        .with_y_axis(ir::axis::Ref::Id("y2".to_string()));
    let plot = ir::Plot::new(vec![s1.into(), s2.into()])
        .with_x_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
        )
        .with_y_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
        )
        .with_x_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
                .with_id("x2".to_string().into())
                .with_opposite_side(),
        )
        .with_y_axis(
            ir::Axis::new()
                .with_ticks(Default::default())
                .with_id("y2".to_string().into())
                .with_opposite_side(),
        );
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "axes/multiple-trbl");
}
