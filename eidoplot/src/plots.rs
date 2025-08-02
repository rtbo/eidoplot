use crate::axis;
use crate::geom;
use crate::render;
use crate::style;
use crate::style::color;

#[derive(Debug, Clone)]
pub enum Plots {
    Plot(Plot),
    Subplots (Subplots),
}

impl Plots {
    pub fn draw_in_rect<S>(&self, surface: &mut S, rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: crate::backend::Surface,
    {
        match self {
            Plots::Plot(plot) => plot.draw_in_rect(surface, rect),
            Plots::Subplots(subplots) => {
                let w = (rect.w - subplots.space * (subplots.cols - 1) as f32) / subplots.cols as f32;
                let h = (rect.h - subplots.space * (subplots.rows - 1) as f32) / subplots.rows as f32;
                let mut y = rect.y;
                for c in 0..subplots.cols {
                    let mut x = rect.x;
                    for r in 0..subplots.rows {
                        let cols = subplots.cols as u32;
                        let idx = (r * cols + c) as usize;
                        let plot = &subplots.plots[idx];
                        plot.draw_in_rect(surface, &geom::Rect { x, y, w, h })?;
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
    pub plots: Vec<Plot>,
    pub space: f32,
}

#[derive(Debug, Clone)]
pub struct Plot {
    pub title: Option<String>,
    pub fill: Option<style::Fill>,
    pub x_axis: axis::Axis,
    pub y_axis: axis::Axis,
    pub series: Vec<Series>,
}

impl Plot {
    pub fn draw_in_rect<S>(&self, surface: &mut S, rect: &geom::Rect) -> Result<(), S::Error>
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
        if let Some(fill) = &self.fill {
            surface.draw_rect(&render::Rect {
                rect: mesh_rect,
                fill: Some(fill.clone()),
                outline: Some(color::BLACK.into()),
            })?;
        }
        for series in &self.series {
            series.plot.draw_in_rect(surface, &mesh_rect)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Series {
    pub name: Option<String>,
    pub plot: SeriesPlot,
}

#[derive(Debug, Clone)]
pub enum SeriesPlot {
    /// Plots data in XY space.
    Xy {
        line: style::Line,
        // TODO: concept of data source
        points: Vec<(f64, f64)>,
    }
}

impl SeriesPlot {
    pub fn draw_in_rect<S>(&self, _surface: &mut S, _rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: crate::backend::Surface,
    {
        Ok(())
    }
}
