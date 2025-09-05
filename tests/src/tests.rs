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

#[test]
fn empty() {
    let plot = ir::Plot::new(vec![]);
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "empty");
}

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

#[test]
fn legend_pos_default() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]);
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/bottom");
}

#[test]
fn legend_pos_top() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::OutTop.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/top");
}

#[test]
fn legend_pos_right() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::OutRight.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/right");
}

#[test]
fn legend_pos_bottom() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::OutBottom.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/bottom");
}

#[test]
fn legend_pos_left() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::OutLeft.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/left");
}

#[test]
fn legend_pos_in_top_left() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::InTopLeft.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_top_left");
}

#[test]
fn legend_pos_in_top() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::InTop.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_top");
}

#[test]
fn legend_pos_in_top_right() {
    let series = line().with_name("line".into()).into();
    let plot =
        ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::InTopRight.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_top_right");
}

#[test]
fn legend_pos_in_right() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::InRight.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_right");
}

#[test]
fn legend_pos_in_bottom_right() {
    let series = line().with_name("line".into()).into();
    let plot =
        ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::InBottomRight.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_bottom_right");
}

#[test]
fn legend_pos_in_bottom() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::InBottom.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_bottom");
}

#[test]
fn legend_pos_in_bottom_left() {
    let series = line().with_name("line".into()).into();
    let plot =
        ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::InBottomLeft.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_bottom_left");
}

#[test]
fn legend_pos_in_left() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(Some(ir::plot::LegendPos::InLeft.into()));
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_left");
}
