use crate::axis::{Axis, scale};
use crate::data;
use crate::geom;
use crate::render;
use crate::style;
use crate::style::color;

#[derive(Debug, Clone)]
pub enum Plots {
    Plot(Plot),
    Subplots(Subplots),
}

impl Plots {
    pub fn draw<S>(&self, surface: &mut S, rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: crate::backend::Surface,
    {
        match self {
            Plots::Plot(plot) => plot.draw(surface, rect),
            Plots::Subplots(subplots) => {
                let w =
                    (rect.w - subplots.space * (subplots.cols - 1) as f32) / subplots.cols as f32;
                let h =
                    (rect.h - subplots.space * (subplots.rows - 1) as f32) / subplots.rows as f32;
                let mut y = rect.y;
                for c in 0..subplots.cols {
                    let mut x = rect.x;
                    for r in 0..subplots.rows {
                        let cols = subplots.cols as u32;
                        let idx = (r * cols + c) as usize;
                        let plot = &subplots.plots[idx];
                        plot.draw(surface, &geom::Rect { x, y, w, h })?;
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

#[derive(Debug, Clone)]
pub struct Plot {
    pub title: Option<String>,
    pub mesh_fill: Option<style::Fill>,
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub series: Vec<Series>,
}

impl Default for Plot {
    fn default() -> Self {
        Plot {
            title: None,
            mesh_fill: None,
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            series: vec![],
        }
    }
}

impl Plot {
    pub fn draw<S>(&self, surface: &mut S, rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: crate::backend::Surface,
    {
        let axis_padding = geom::Padding::Custom {
            t: 0.0,
            r: 0.0,
            b: rect.h / 10.0,
            l: rect.w / 10.0,
        };
        let mesh_rect = rect.pad(&axis_padding);
        if let Some(fill) = &self.mesh_fill {
            surface.draw_rect(&render::Rect {
                rect: mesh_rect,
                fill: Some(fill.clone()),
                stroke: Some(color::BLACK.into()),
            })?;
        }

        let data_bounds = {
            let mut x_bounds = data::Bounds::NAN;
            let mut y_bounds = data::Bounds::NAN;
            for series in &self.series {
                let (x, y) = series.data_bounds();
                x_bounds.add_bounds(x);
                y_bounds.add_bounds(y);
            }
            (x_bounds, y_bounds)
        };
        let x_ticks = self
            .x_axis
            .ticks
            .as_ref()
            .map(|t| t.ticks(data_bounds.0))
            .unwrap_or_else(Vec::new);
        let y_ticks = self
            .y_axis
            .ticks
            .as_ref()
            .map(|t| t.ticks(data_bounds.1))
            .unwrap_or_else(Vec::new);

        let cm = CoordMap {
            x: self.x_axis.scale.coord_mapper(mesh_rect.w, data_bounds.0),
            y: self.y_axis.scale.coord_mapper(mesh_rect.h, data_bounds.1),
        };

        let mut x_path = geom::PathBuilder::new();
        x_path.move_to(mesh_rect.x, mesh_rect.y + mesh_rect.h);
        x_path.line_to(mesh_rect.x + mesh_rect.w, mesh_rect.y + mesh_rect.h);
        for t in x_ticks.iter().copied() {
            let x = cm.x.map_coord(t);
            x_path.move_to(mesh_rect.x + x, mesh_rect.y + mesh_rect.h + 2.0);
            x_path.line_to(mesh_rect.x + x, mesh_rect.y + mesh_rect.h - 2.0);
        }
        let x_path = x_path.finish().expect("Should be a valid path");
        let x_path = render::Path {
            path: x_path,
            fill: None,
            stroke: Some(color::BLACK.into()),
        };
        surface.draw_path(&x_path)?;

        let mut y_path = geom::PathBuilder::new();
        y_path.move_to(mesh_rect.x, mesh_rect.y);
        y_path.line_to(mesh_rect.x, mesh_rect.y + mesh_rect.h);
        for t in y_ticks.iter().copied() {
            let y = cm.y.map_coord(t);
            y_path.move_to(mesh_rect.x - 2.0, mesh_rect.y + mesh_rect.h - y);
            y_path.line_to(mesh_rect.x + 2.0, mesh_rect.y + mesh_rect.h - y);
        }
        let y_path = y_path.finish().expect("Should be a valid path");
        let y_path = render::Path {
            path: y_path,
            fill: None,
            stroke: Some(color::BLACK.into()),
        };
        surface.draw_path(&y_path)?;

        for series in &self.series {
            series.plot.draw(surface, &mesh_rect, &cm)?;
        }
        Ok(())
    }
}

struct CoordMap {
    x: Box<dyn scale::MapCoord>,
    y: Box<dyn scale::MapCoord>,
}

impl CoordMap {
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
    pub fn data_bounds(&self) -> (data::Bounds, data::Bounds) {
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
    fn draw<S>(&self, surface: &mut S, rect: &geom::Rect, cm: &CoordMap) -> Result<(), S::Error>
    where
        S: crate::backend::Surface,
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
    pub fn data_bounds(&self) -> (data::Bounds, data::Bounds) {
        let mut x_bounds = data::Bounds::NAN;
        let mut y_bounds = data::Bounds::NAN;
        for (x, y) in &self.points {
            x_bounds.add_point(*x);
            y_bounds.add_point(*y);
        }
        (x_bounds, y_bounds)
    }

    fn draw<S>(&self, surface: &mut S, rect: &geom::Rect, cm: &CoordMap) -> Result<(), S::Error>
    where
        S: crate::backend::Surface,
    {
        let mut pb = geom::PathBuilder::with_capacity(self.points.len() + 1, self.points.len());
        for (i, dp) in self.points.iter().enumerate() {
            let (x, y) = cm.map_coord(*dp);
            let x = x + rect.x;
            // Y coord is flipped here. Is it the right place?
            let y = rect.h - y + rect.y;
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
        };
        surface.draw_path(&path)?;
        Ok(())
    }
}
