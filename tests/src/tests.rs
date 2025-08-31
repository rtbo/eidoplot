use eidoplot::{geom, ir, style};

use crate::*;

fn create_fig(plot: ir::Plot) -> ir::Figure {
    ir::Figure::new(ir::figure::Plots::Plot(plot)).with_size(geom::Size::new(400.0, 300.0))
}

#[test]
fn line_y_eq_x() {
    let x = vec![1.0, 2.0, 3.0];
    let y = x.clone();

    let line = ir::Series::Line(ir::series::Line {
        name: None,
        line: style::series::Line::default(),
        x_data: ir::series::DataCol::Inline(x.into()),
        y_data: ir::series::DataCol::Inline(y.into()),
    });

    let plot = ir::Plot {
        series: vec![line],
        ..Default::default()
    };

    let fig = create_fig(plot);

    assert_fig_eq_ref!(&fig, "line_y_eq_x");
}

#[test]
fn line_y_eq_x_with_minor() {
    let x = vec![1.0, 2.0, 3.0];
    let y = x.clone();

    let line = ir::Series::Line(ir::series::Line {
        name: None,
        line: style::series::Line::default(),
        x_data: ir::series::DataCol::Inline(x.into()),
        y_data: ir::series::DataCol::Inline(y.into()),
    });

    let plot = ir::Plot {
        series: vec![line],
        y_axis: ir::axis::Axis::new(Default::default()).with_minor_ticks(Default::default()),
        ..Default::default()
    };

    let fig = create_fig(plot);

    assert_fig_eq_ref!(&fig, "line_y_eq_x_minor");
}
