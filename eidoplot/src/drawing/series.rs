use crate::data;
use crate::drawing::{CalcViewBounds, scale, Ctx};
use crate::geom;
use crate::ir;
use crate::render;

use scale::CoordMapXy;

impl CalcViewBounds for ir::Series {
    fn calc_view_bounds(&self) -> (data::ViewBounds, data::ViewBounds) {
        match &self.plot {
            ir::plot::SeriesPlot::Xy(xy) => xy.calc_view_bounds(),
        }
    }
}

impl CalcViewBounds for ir::plot::XySeries {
    fn calc_view_bounds(&self) -> (data::ViewBounds, data::ViewBounds) {
        let mut x_bounds = data::ViewBounds::NAN;
        let mut y_bounds = data::ViewBounds::NAN;
        for (x, y) in &self.points {
            x_bounds.add_point(*x);
            y_bounds.add_point(*y);
        }
        (x_bounds, y_bounds)
    }
}

pub fn draw_series_plot<S>(
    ctx: &mut Ctx<S>,
    series_plot: &ir::plot::SeriesPlot,
    rect: &geom::Rect,
    cm: &CoordMapXy,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    match series_plot {
        ir::plot::SeriesPlot::Xy(xy) => draw_series_xy(ctx, xy, rect, cm),
    }
}
fn draw_series_xy<S>(
    ctx: &mut Ctx<S>,
    xy: &ir::plot::XySeries,
    rect: &geom::Rect,
    cm: &CoordMapXy,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    let mut pb = geom::PathBuilder::with_capacity(xy.points.len() + 1, xy.points.len());
    for (i, dp) in xy.points.iter().enumerate() {
        let (x, y) = cm.map_coord(*dp);
        let x = rect.left() + x;
        let y = rect.bottom() - y;
        if i == 0 {
            pb.move_to(x, y);
        } else {
            pb.line_to(x, y);
        }
    }
    let path = pb.finish().expect("Should be a valid path");
    let path = render::Path {
        path,
        fill: None,
        stroke: Some(xy.line.clone()),
        transform: None,
    };
    ctx.surface.draw_path(&path)?;
    Ok(())
}
