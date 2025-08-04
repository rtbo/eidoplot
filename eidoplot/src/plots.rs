use crate::axis::{Axis, scale};
use crate::data;
use crate::geom;
use crate::render;
use crate::style;
use crate::style::color;
use crate::{backend, missing_params};

#[derive(Debug, Clone)]
pub enum Plots {
    Plot(Plot),
    Subplots(Subplots),
}

impl Plots {
    pub fn draw<S>(&self, surface: &mut S, rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        match self {
            Plots::Plot(plot) => plot.draw(surface, rect),
            Plots::Subplots(subplots) => {
                let w = (rect.width() - subplots.space * (subplots.cols - 1) as f32)
                    / subplots.cols as f32;
                let h = (rect.height() - subplots.space * (subplots.rows - 1) as f32)
                    / subplots.rows as f32;
                let mut y = rect.y();
                for c in 0..subplots.cols {
                    let mut x = rect.x();
                    for r in 0..subplots.rows {
                        let cols = subplots.cols as u32;
                        let idx = (r * cols + c) as usize;
                        let plot = &subplots.plots[idx];
                        plot.draw(surface, &geom::Rect::from_xywh(x, y, w, h))?;
                        x += w + subplots.space;
                    }
                    y += h + subplots.space;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Subplots {
    pub rows: u32,
    pub cols: u32,
    pub space: f32,
    pub plots: Vec<Plot>,
}

impl Default for Subplots {
    fn default() -> Self {
        Subplots {
            rows: 1,
            cols: 1,
            space: 10.0,
            plots: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlotBorder {
    Box(style::Line),
    Axis(style::Line),
    AxisArrow {
        stroke: style::Line,
        size: f32,
        overflow: f32,
    },
}

impl Default for PlotBorder {
    fn default() -> Self {
        PlotBorder::Box(color::BLACK.into())
    }
}

#[derive(Debug, Clone)]
pub struct Plot {
    pub title: Option<String>,
    pub fill: Option<style::Fill>,
    pub border: Option<PlotBorder>,
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub series: Vec<Series>,
}

impl Default for Plot {
    fn default() -> Self {
        Plot {
            title: None,
            fill: None,
            border: Some(PlotBorder::default()),
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            series: vec![],
        }
    }
}

impl Plot {
    pub fn draw<S>(&self, surface: &mut S, rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        let axis_padding = missing_params::AXIS_PADDING;
        let rect = rect.pad(&axis_padding);

        // initialize view bounds to view the whole data
        let view_bounds = self.calc_data_bounds();

        let coord_map = MapCoordXy {
            x: self.x_axis.scale.coord_mapper(rect.width(), view_bounds.0),
            y: self.y_axis.scale.coord_mapper(rect.height(), view_bounds.1),
        };

        // update view bounds to view what is deemed visible by the axis scale
        let view_bounds = (coord_map.x.view_bounds(), coord_map.y.view_bounds());

        self.draw_background(surface, &rect)?;
        self.draw_series(surface, &rect, &coord_map)?;
        self.draw_ticks(surface, &rect, view_bounds, &coord_map)?;
        self.draw_border(surface, &rect)?;

        Ok(())
    }

    fn calc_data_bounds(&self) -> (data::ViewBounds, data::ViewBounds) {
        let mut x_bounds = data::ViewBounds::NAN;
        let mut y_bounds = data::ViewBounds::NAN;
        for series in &self.series {
            let (x, y) = series.data_bounds();
            x_bounds.add_bounds(x);
            y_bounds.add_bounds(y);
        }
        (x_bounds, y_bounds)
    }

    fn draw_background<S>(&self, surface: &mut S, rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        if let Some(fill) = &self.fill {
            surface.draw_rect(&render::Rect {
                rect: *rect,
                fill: Some(fill.clone()),
                stroke: None,
                transform: None,
            })?;
        }
        Ok(())
    }

    fn draw_border<S>(&self, surface: &mut S, rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        match self.border.as_ref() {
            None => Ok(()),
            Some(PlotBorder::Box(stroke)) => surface.draw_rect(&render::Rect {
                rect: *rect,
                fill: None,
                stroke: Some(stroke.clone()),
                transform: None,
            }),
            Some(PlotBorder::Axis(stroke)) => {
                let mut path = geom::PathBuilder::with_capacity(4, 4);
                path.move_to(rect.left(), rect.top());
                path.line_to(rect.left(), rect.bottom());
                path.line_to(rect.right(), rect.bottom());
                let path = path.finish().expect("Should be a valid path");
                let path = render::Path {
                    path,
                    fill: None,
                    stroke: Some(stroke.clone()),
                    transform: None,
                };
                surface.draw_path(&path)
            }
            Some(PlotBorder::AxisArrow { .. }) => {
                todo!("Draw axis arrow")
            }
        }
    }

    fn draw_series<S>(
        &self,
        surface: &mut S,
        rect: &geom::Rect,
        coord_map: &MapCoordXy,
    ) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        surface.push_clip_rect(rect, None)?;
        for series in &self.series {
            series.plot.draw(surface, &rect, &coord_map)?;
        }
        surface.pop_clip()?;
        Ok(())
    }

    fn draw_ticks<S>(
        &self,
        surface: &mut S,
        rect: &geom::Rect,
        (x_bounds, y_bounds): (data::ViewBounds, data::ViewBounds),
        coord_map: &MapCoordXy,
    ) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        let x_ticks = self
            .x_axis
            .ticks
            .as_ref()
            .map(|t| t.ticks(x_bounds))
            .unwrap_or_else(Vec::new);

        let x_ticks_path = ticks_path(&x_ticks, &x_bounds, &*coord_map.x, None);
        let x_ticks_tr = geom::Transform::from_translate(rect.left(), rect.bottom());
        let x_ticks_path = render::Path {
            path: x_ticks_path,
            fill: None,
            stroke: Some(color::BLACK.into()),
            transform: Some(x_ticks_tr),
        };
        surface.draw_path(&x_ticks_path)?;

        let y_ticks = self
            .y_axis
            .ticks
            .as_ref()
            .map(|t| t.ticks(y_bounds))
            .unwrap_or_else(Vec::new);
        let y_ticks_path = ticks_path(&y_ticks, &y_bounds, &*coord_map.y, Some(x_ticks_path.path));
        let y_ticks_path = render::Path {
            path: y_ticks_path,
            fill: None,
            stroke: Some(color::BLACK.into()),
            transform: Some(x_ticks_tr.pre_rotate(90.0)),
        };
        surface.draw_path(&y_ticks_path)?;
        Ok(())
    }
}

/// Build the ticks path along X axis.
/// Y axis will use the same function and rotate 90°
fn ticks_path(
    ticks: &[f64],
    db: &data::ViewBounds,
    cm: &dyn scale::MapCoord,
    reuse_alloc: Option<geom::Path>,
) -> geom::Path {
    let sz = missing_params::TICK_SIZE;
    let mut path = reuse_alloc
        .map(|p| p.clear())
        .unwrap_or_else(geom::PathBuilder::new);
    for tick in ticks {
        if !db.contains(*tick) {
            continue;
        }
        let x = cm.map_coord(*tick);
        path.move_to(x, -sz);
        path.line_to(x, sz);
    }
    path.finish().expect("Should be a valid path")
}

struct MapCoordXy {
    x: Box<dyn scale::MapCoord>,
    y: Box<dyn scale::MapCoord>,
}

impl MapCoordXy {
    fn map_coord(&self, dp: (f64, f64)) -> (f32, f32) {
        (self.x.map_coord(dp.0), self.y.map_coord(dp.1))
    }
}

#[derive(Debug, Clone)]
pub struct Series {
    pub name: Option<String>,
    pub plot: SeriesPlot,
}

impl Series {
    pub fn data_bounds(&self) -> (data::ViewBounds, data::ViewBounds) {
        match &self.plot {
            SeriesPlot::Xy(xy) => xy.data_bounds(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SeriesPlot {
    /// Plots data in XY space.
    Xy(XySeries),
}

impl SeriesPlot {
    fn draw<S>(&self, surface: &mut S, rect: &geom::Rect, cm: &MapCoordXy) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        match self {
            SeriesPlot::Xy(xy) => xy.draw(surface, rect, cm),
        }
    }
}

#[derive(Debug, Clone)]
pub struct XySeries {
    pub line: style::Line,
    pub points: Vec<(f64, f64)>,
}

impl XySeries {
    pub fn data_bounds(&self) -> (data::ViewBounds, data::ViewBounds) {
        let mut x_bounds = data::ViewBounds::NAN;
        let mut y_bounds = data::ViewBounds::NAN;
        for (x, y) in &self.points {
            x_bounds.add_point(*x);
            y_bounds.add_point(*y);
        }
        (x_bounds, y_bounds)
    }

    fn draw<S>(&self, surface: &mut S, rect: &geom::Rect, cm: &MapCoordXy) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        let mut pb = geom::PathBuilder::with_capacity(self.points.len() + 1, self.points.len());
        for (i, dp) in self.points.iter().enumerate() {
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
            stroke: Some(self.line.clone()),
            transform: None,
        };
        surface.draw_path(&path)?;
        Ok(())
    }
}
