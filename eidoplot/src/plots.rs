use crate::axis;
use crate::geom;
use crate::render;
use crate::style;
use crate::style::color;

pub enum Plots {
    Plot(Plot),
    Subplots {
        rows: u32,
        cols: u32,
        plots: Vec<Plot>,
    },
}

impl Plots {
    pub fn draw<S>(&self, surface: &mut S) -> Result<(), S::Error>
    where
        S: crate::backend::Surface,
    {
        match self {
            Plots::Plot(plot) => plot.draw(surface),
            Plots::Subplots { plots, .. } => {
                for plot in plots {
                    plot.draw(surface)?
                }
                Ok(())
            }
        }
    }
}

pub struct Plot {
    pub title: Option<String>,
    pub fill: Option<style::Fill>,
    pub desc: PlotDesc,
}

impl Plot {
    pub fn draw<S>(&self, surface: &mut S) -> Result<(), S::Error>
    where
        S: crate::backend::Surface,
    {
        if let Some(fill) = &self.fill {
            surface.draw_rect(&render::Rect {
                rect: geom::Rect {
                    x: 50.0,
                    y: 50.0,
                    w: 700.0,
                    h: 400.0,
                },
                fill: Some(fill.clone()),
                outline: Some(color::BLACK.into()),
            })?;
        }
        Ok(())
    }
}

pub enum PlotDesc {
    Curves(Curves),
}

pub struct Curves {
    pub x_axis: axis::Axis,
    pub y_axis: axis::Axis,
    pub series: Vec<XySeries>,
}

pub struct XySeries {
    pub name: String,
    pub line_style: style::Line,
    pub points: Vec<(f64, f64)>,
}
