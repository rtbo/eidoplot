use plotive::{des, geom};

use crate::*;

fn fig_small<P>(plots: P) -> des::Figure
where
    P: Into<des::figure::Plots>,
{
    des::Figure::new(plots.into()).with_size(geom::Size::new(400.0, 300.0))
}

fn fig_mid<P>(plots: P) -> des::Figure
where
    P: Into<des::figure::Plots>,
{
    des::Figure::new(plots.into()).with_size(geom::Size::new(600.0, 450.0))
}

fn fig_high<P>(plots: P) -> des::Figure
where
    P: Into<des::figure::Plots>,
{
    des::Figure::new(plots.into()).with_size(geom::Size::new(400.0, 500.0))
}

fn fig_wide<P>(plots: P) -> des::Figure
where
    P: Into<des::figure::Plots>,
{
    des::Figure::new(plots.into()).with_size(geom::Size::new(600.0, 300.0))
}

fn line() -> des::series::Line {
    let x = vec![1.0, 2.0, 3.0];
    let y = vec![1.0, 2.0, 3.0];
    des::series::Line::new(x.into(), y.into())
}

fn line2(x: &[f64], y: &[f64]) -> des::series::Line {
    let x = x.to_vec();
    let y = y.to_vec();
    des::series::Line::new(x.into(), y.into())
}

mod axes;
mod interp;
mod legend;
mod subplots;

#[test]
fn empty() {
    let plot = des::Plot::new(vec![]);
    let fig = fig_small(plot);

    assert_fig_eq_ref!(&fig, "empty");
}

#[test]
fn empty_title() {
    let plot = des::Plot::new(vec![]);
    let fig = fig_small(plot).with_title("Title".into());

    assert_fig_eq_ref!(&fig, "empty-title");
}
