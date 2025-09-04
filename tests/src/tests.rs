use eidoplot::{geom, ir};

use crate::*;

fn create_fig(plot: ir::Plot) -> ir::Figure {
    ir::Figure::new(ir::figure::Plots::Plot(plot)).with_size(geom::Size::new(400.0, 300.0))
}

#[test]
fn axes_default() {
    let x = vec![1.0, 2.0, 3.0];
    let y = x.clone();

    let line: ir::Series = ir::series::Line::new(None, x.into(), y.into()).into();

    let plot = ir::Plot::new(vec![line]);

    let fig = create_fig(plot);

    assert_fig_eq_ref!(&fig, "axes/default");
}

#[test]
fn axes_minor_ticks() {
    let x = vec![1.0, 2.0, 3.0];
    let y = x.clone();

    let line: ir::Series = ir::series::Line::new(None, x.into(), y.into()).into();

    let plot = ir::Plot::new(vec![line])
        .with_y_axis(ir::axis::Axis::default().with_minor_ticks(Default::default()));

    let fig = create_fig(plot);

    assert_fig_eq_ref!(&fig, "axes/minor-ticks");
}

#[test]
fn legend_pos() {
    let x = vec![1.0, 2.0, 3.0];
    let y = x.clone();

    let line: ir::Series = ir::series::Line::new(Some("line".into()), x.into(), y.into()).into();

    let plot = ir::Plot::new(vec![line.clone()]);
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/bottom");

    let plot = ir::Plot::new(vec![line.clone()]).with_legend(Some(ir::plot::LegendPos::OutRight.into()));
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/right");

    let plot = ir::Plot::new(vec![line.clone()]).with_legend(Some(ir::plot::LegendPos::OutLeft.into()));
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/left");

    let plot = ir::Plot::new(vec![line.clone()]).with_legend(Some(ir::plot::LegendPos::OutTop.into()));
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/top");

    let plot = ir::Plot::new(vec![line.clone()]).with_legend(Some(ir::plot::LegendPos::InTopLeft.into()));
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/in_top_left");

    let plot = ir::Plot::new(vec![line.clone()]).with_legend(Some(ir::plot::LegendPos::InTop.into()));
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/in_top");

    let plot = ir::Plot::new(vec![line.clone()]).with_legend(Some(ir::plot::LegendPos::InTopRight.into()));
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/in_top_right");

    let plot = ir::Plot::new(vec![line.clone()]).with_legend(Some(ir::plot::LegendPos::InRight.into()));
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/in_right");

    let plot = ir::Plot::new(vec![line.clone()]).with_legend(Some(ir::plot::LegendPos::InBottomRight.into()));
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/in_bottom_right");

    let plot = ir::Plot::new(vec![line.clone()]).with_legend(Some(ir::plot::LegendPos::InBottom.into()));
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/in_bottom");

    let plot = ir::Plot::new(vec![line.clone()]).with_legend(Some(ir::plot::LegendPos::InBottomLeft.into()));
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/in_bottom_left");

    let plot = ir::Plot::new(vec![line]).with_legend(Some(ir::plot::LegendPos::InLeft.into()));
    let fig = create_fig(plot);
    assert_fig_eq_ref!(&fig, "legend-pos/in_left");
}
