use eidoplot::ir;

use super::{fig_small, line};
use crate::{TestHarness, assert_fig_eq_ref};

#[test]
fn legend_pos_default() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(Default::default());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/bottom");
}

#[test]
fn legend_pos_top() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::OutTop.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/top");
}

#[test]
fn legend_pos_right() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::OutRight.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/right");
}

#[test]
fn legend_pos_bottom() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::OutBottom.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/bottom");
}

#[test]
fn legend_pos_left() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::OutLeft.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/left");
}

#[test]
fn legend_pos_in_top_left() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::InTopLeft.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_top_left");
}

#[test]
fn legend_pos_in_top() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::InTop.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_top");
}

#[test]
fn legend_pos_in_top_right() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::InTopRight.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_top_right");
}

#[test]
fn legend_pos_in_right() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::InRight.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_right");
}

#[test]
fn legend_pos_in_bottom_right() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::InBottomRight.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_bottom_right");
}

#[test]
fn legend_pos_in_bottom() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::InBottom.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_bottom");
}

#[test]
fn legend_pos_in_bottom_left() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::InBottomLeft.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_bottom_left");
}

#[test]
fn legend_pos_in_left() {
    let series = line().with_name("line".into()).into();
    let plot = ir::Plot::new(vec![series]).with_legend(ir::plot::LegendPos::InLeft.into());
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "legend-pos/in_left");
}
