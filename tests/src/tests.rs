use eidoplot::{geom, ir};

use crate::*;

fn fig_small<P>(plots: P) -> ir::Figure
where
    P: Into<ir::figure::Plots>,
{
    ir::Figure::new(plots.into()).with_size(geom::Size::new(400.0, 300.0))
}

fn fig_mid<P>(plots: P) -> ir::Figure
where
    P: Into<ir::figure::Plots>,
{
    ir::Figure::new(plots.into()).with_size(geom::Size::new(600.0, 450.0))
}

fn fig_high<P>(plots: P) -> ir::Figure
where
    P: Into<ir::figure::Plots>,
{
    ir::Figure::new(plots.into()).with_size(geom::Size::new(400.0, 500.0))
}

fn fig_wide<P>(plots: P) -> ir::Figure
where
    P: Into<ir::figure::Plots>,
{
    ir::Figure::new(plots.into()).with_size(geom::Size::new(600.0, 300.0))
}

fn line() -> ir::series::Line {
    let x = vec![1.0, 2.0, 3.0];
    let y = vec![1.0, 2.0, 3.0];
    ir::series::Line::new(x.into(), y.into())
}

fn line2(x: &[f64], y: &[f64]) -> ir::series::Line {
    let x = x.to_vec();
    let y = y.to_vec();
    ir::series::Line::new(x.into(), y.into())
}

mod axes;
mod legend;
mod subplots;

#[test]
fn empty() {
    let plot = ir::Plot::new(vec![]);
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "empty");
}

#[test]
fn empty_title() {
    let plot = ir::Plot::new(vec![]);
    let fig = fig_small(plot).with_title("Title".into());

    assert_fig_eq_ref!(&fig, "empty-title");
}
