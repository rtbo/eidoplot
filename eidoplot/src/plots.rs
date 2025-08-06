use crate::axis::scale::MapCoord;
use crate::axis::tick::Ticks;
use crate::axis::{Axis, scale};
use crate::geom;
use crate::render::{self, TextAnchor};
use crate::style;
use crate::style::color;
use crate::{backend, missing_params};
use crate::{data, text};

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

        self.draw_background(surface, &rect)?;
        self.draw_series(surface, &rect, &coord_map)?;

        if let Some(x_ticks) = self.x_axis.ticks.as_ref() {
            let x_vb = coord_map.x.view_bounds();
            self.draw_x_ticks(surface, &rect, x_ticks, x_vb, &*coord_map.x)?;
        }
        if let Some(y_ticks) = self.y_axis.ticks.as_ref() {
            let y_vb = coord_map.y.view_bounds();
            self.draw_y_ticks(surface, &rect, y_ticks, y_vb, &*coord_map.y)?;
        }

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
        surface.push_clip(&render::Clip {
            path: rect.to_path(),
            transform: None,
        })?;
        for series in &self.series {
            series.plot.draw(surface, &rect, &coord_map)?;
        }
        surface.pop_clip()?;
        Ok(())
    }

    fn draw_x_ticks<S>(
        &self,
        surface: &mut S,
        rect: &geom::Rect,
        x_ticks: &Ticks,
        x_vb: data::ViewBounds,
        x_cm: &dyn MapCoord,
    ) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        let ticks = x_ticks.locator().ticks(x_vb);
        let transform = geom::Transform::from_translate(rect.left(), rect.bottom());
        self.draw_ticks_path(surface, &ticks, &x_vb, x_cm, &transform)?;

        let lbl_format = x_ticks.formatter().label_format(x_ticks.locator(), x_vb);
        let fill = x_ticks.color().into();

        for xt in ticks.iter().copied() {
            let text = lbl_format.format_label(xt);
            let text = text::Text::new(text, x_ticks.font().clone());

            let x = x_cm.map_coord(xt);
            let x = rect.left() + x;
            let pos = geom::Point {
                x,
                y: rect.bottom() + missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN,
            };
            let anchor = TextAnchor {
                pos,
                align: render::TextAlign::Center,
                baseline: render::TextBaseline::Hanging,
            };
            let text = render::Text {
                text,
                anchor,
                fill,
                transform: None,
            };

            surface.draw_text(&text)?;
        }

        if let Some(annot) = lbl_format.axis_annotation() {
            let text = text::Text::new(annot, x_ticks.font().clone());
            let pos = geom::Point {
                x: rect.right(),
                y: rect.bottom()
                    + missing_params::TICK_SIZE
                    + 2.0 * missing_params::TICK_LABEL_MARGIN
                    + x_ticks.font().size(),
            };
            let anchor = TextAnchor {
                pos,
                align: render::TextAlign::End,
                baseline: render::TextBaseline::Hanging,
            };
            let text = render::Text {
                text,
                anchor,
                fill,
                transform: None,
            };
            surface.draw_text(&text)?;
        }

        Ok(())
    }

    fn draw_y_ticks<S>(
        &self,
        surface: &mut S,
        rect: &geom::Rect,
        y_ticks: &Ticks,
        y_vb: data::ViewBounds,
        y_cm: &dyn MapCoord,
    ) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        let ticks = y_ticks.locator().ticks(y_vb);
        let transform =
            geom::Transform::from_translate(rect.left(), rect.bottom()).pre_rotate(90.0);
        self.draw_ticks_path(surface, &ticks, &y_vb, y_cm, &transform)?;
        Ok(())
    }

    fn draw_ticks_path<S>(
        &self,
        surface: &mut S,
        ticks: &[f64],
        vb: &data::ViewBounds,
        cm: &dyn MapCoord,
        transform: &geom::Transform,
    ) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        let ticks_path = ticks_path(&ticks, &vb, cm, None);
        let ticks_path = render::Path {
            path: ticks_path,
            fill: None,
            stroke: Some(color::BLACK.into()),
            transform: Some(*transform),
        };
        surface.draw_path(&ticks_path)?;
        Ok(())
    }
}

/// Build the ticks path along X axis.
/// Y axis will use the same function and rotate 90°
fn ticks_path(
    ticks: &[f64],
    vb: &data::ViewBounds,
    cm: &dyn scale::MapCoord,
    reuse_alloc: Option<geom::Path>,
) -> geom::Path {
    let sz = missing_params::TICK_SIZE;
    let mut path = reuse_alloc
        .map(|p| p.clear())
        .unwrap_or_else(geom::PathBuilder::new);
    for tick in ticks {
        if !vb.contains(*tick) {
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
