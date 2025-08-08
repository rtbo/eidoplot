use crate::data;
use crate::drawing::{CalcViewBounds, ctx, scale, series, ticks};
use crate::geom;
use crate::ir;
use crate::missing_params;
use crate::render;
use crate::style::{self, defaults};

use ctx::Ctx;
use scale::{CoordMap, CoordMapXy};

impl CalcViewBounds for ir::Plot {
    fn calc_view_bounds(&self) -> (data::ViewBounds, data::ViewBounds) {
        let mut x_bounds = data::ViewBounds::NAN;
        let mut y_bounds = data::ViewBounds::NAN;
        for series in &self.series {
            let (x, y) = series.calc_view_bounds();
            x_bounds.add_bounds(x);
            y_bounds.add_bounds(y);
        }
        (x_bounds, y_bounds)
    }
}

struct Ticks {
    locs: Vec<f64>,
    lbls: Vec<String>,
    annot: Option<String>,
    font: style::Font,
    color: style::Color,
}

struct Axis {
    ortho_sz: f32,
    coord_map: Box<dyn CoordMap>,
    ticks: Option<Ticks>,
    label: Option<String>,
}

impl CoordMap for Axis {
    fn map_coord(&self, v: f64) -> f32 {
        self.coord_map.map_coord(v)
    }
    fn view_bounds(&self) -> data::ViewBounds {
        self.coord_map.view_bounds()
    }
}

struct PlotAxes {
    x: Axis,
    y: Axis,
}

impl PlotAxes {
    fn x_height(&self) -> f32 {
        self.x.ortho_sz
    }
    fn y_width(&self) -> f32 {
        self.y.ortho_sz
    }
}

fn plot_insets(plot: &ir::Plot) -> (f32, f32) {
    match plot.insets {
        Some(ir::plot::Insets::Fixed(x, y)) => (x, y),
        Some(ir::plot::Insets::Auto) => auto_insets(plot),
        None => (0.0, 0.0),
    }
}

fn auto_insets(_plot: &ir::Plot) -> (f32, f32) {
    defaults::PLOT_XY_AUTO_INSETS
}

fn setup_ticks(ticks: &ir::axis::Ticks, vb: data::ViewBounds) -> Ticks {
    let locs = ticks::locate(ticks.locator(), vb);
    let lbl_formatter = ticks::label_formatter(ticks, vb);
    let lbls = locs
        .iter()
        .copied()
        .map(|l| lbl_formatter.format_label(l))
        .collect();
    let annot = lbl_formatter.axis_annotation().map(String::from);
    Ticks {
        locs,
        lbls,
        annot,
        font: ticks.font().clone(),
        color: ticks.color(),
    }
}

impl<'a, S> Ctx<'a, S> {
    pub fn draw_plot(&mut self, plot: &ir::Plot, rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        let rect = rect.pad(&missing_params::PLOT_PADDING);
        let axes = self.setup_plot_axes(plot, &rect);

        let rect = rect
            .shifted_left_side(axes.y_width())
            .shifted_bottom_side(-axes.x_height());

        self.draw_plot_background(plot, &rect)?;
        self.draw_plot_series(plot, &rect, &axes)?;
        self.draw_x_axis(&axes.x, &rect)?;
        self.draw_y_axis(&axes.y, &rect)?;
        self.draw_plot_border(plot.border.as_ref(), &rect)?;

        Ok(())
    }

    fn setup_plot_axes(&mut self, plot: &ir::Plot, rect: &geom::Rect) -> PlotAxes {
        let vb = plot.calc_view_bounds();
        let insets = plot_insets(plot);

        // x-axis height only depends on font size, so it can be computed right-away,
        // y-axis width depends on font width therefore we have to generate tick labels,
        // which somehow depends on the x-axis height (for available space)

        let x_height = self.calculate_x_axis_height(&plot.x_axis);
        let rect = rect.shifted_bottom_side(-x_height);

        let y_cm = scale::map_scale_coord(&plot.y_axis.scale, rect.height(), vb.1, insets.1);
        let y_axis = self.setup_y_axis(&plot.y_axis, y_cm);
        let rect = rect.shifted_left_side(y_axis.ortho_sz);

        let x_cm = scale::map_scale_coord(&plot.x_axis.scale, rect.width(), vb.0, insets.0);
        let x_axis = self.setup_x_axis(&plot.x_axis, x_cm, x_height);

        PlotAxes {
            x: x_axis,
            y: y_axis,
        }
    }

    fn setup_y_axis(&mut self, y_axis: &ir::Axis, coord_map: Box<dyn CoordMap>) -> Axis {
        let ticks = y_axis
            .ticks
            .as_ref()
            .map(|t| setup_ticks(t, coord_map.view_bounds()));

        let lbls = ticks.as_ref().map(|ts| ts.lbls.iter());

        let y_width = self.calculate_y_axis_width(y_axis, lbls);

        Axis {
            ortho_sz: y_width,
            coord_map,
            ticks,
            label: y_axis.label.clone(),
        }
    }

    fn setup_x_axis(
        &mut self,
        x_axis: &ir::Axis,
        coord_map: Box<dyn CoordMap>,
        x_height: f32,
    ) -> Axis {
        let ticks = x_axis
            .ticks
            .as_ref()
            .map(|t| setup_ticks(t, coord_map.view_bounds()));

        Axis {
            ortho_sz: x_height,
            coord_map,
            ticks,
            label: x_axis.label.clone(),
        }
    }
}

impl<'a, S> Ctx<'a, S> {
    fn draw_plot_background(&mut self, plot: &ir::Plot, rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        if let Some(fill) = plot.fill.as_ref() {
            self.surface.draw_rect(&render::Rect {
                rect: *rect,
                fill: Some(fill.clone()),
                stroke: None,
                transform: None,
            })?;
        }
        Ok(())
    }

    fn draw_plot_border(
        &mut self,
        border: Option<&ir::plot::Border>,
        rect: &geom::Rect,
    ) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        match border {
            None => Ok(()),
            Some(ir::plot::Border::Box(stroke)) => self.surface.draw_rect(&render::Rect {
                rect: *rect,
                fill: None,
                stroke: Some(stroke.clone()),
                transform: None,
            }),
            Some(ir::plot::Border::Axis(stroke)) => {
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
                self.surface.draw_path(&path)
            }
            Some(ir::plot::Border::AxisArrow { .. }) => {
                todo!("Draw axis arrow")
            }
        }
    }

    fn draw_plot_series(
        &mut self,
        plot: &ir::Plot,
        rect: &geom::Rect,
        axes: &PlotAxes,
    ) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        self.surface.push_clip(&render::Clip {
            path: rect.to_path(),
            transform: None,
        })?;

        let cm = CoordMapXy {
            x: &axes.x,
            y: &axes.y,
        };

        for series in &plot.series {
            series::draw_series_plot(self, &series.plot, rect, &cm)?;
        }
        self.surface.pop_clip()?;
        Ok(())
    }
}

impl<'a, S> Ctx<'a, S> {
    fn draw_x_axis(&mut self, x_axis: &Axis, rect: &geom::Rect) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        let mut label_y = rect.bottom() + missing_params::AXIS_LABEL_MARGIN;
        if let Some(x_ticks) = x_axis.ticks.as_ref() {
            self.draw_x_ticks(&rect, x_ticks, x_axis)?;
            label_y += missing_params::TICK_SIZE
                + missing_params::TICK_LABEL_MARGIN
                + x_ticks.font.size();
        }
        if let Some(label) = &x_axis.label {
            let font = style::Font::new(
                missing_params::AXIS_LABEL_FONT_FAMILY.into(),
                missing_params::AXIS_LABEL_FONT_SIZE,
            );
            let anchor = render::TextAnchor {
                pos: geom::Point::new(rect.center_x(), label_y),
                align: render::TextAlign::Center,
                baseline: render::TextBaseline::Hanging,
            };
            let text = render::Text {
                text: label.clone(),
                font,
                anchor,
                fill: missing_params::AXIS_LABEL_COLOR.into(),
                transform: None,
            };
            self.surface.draw_text(&text)?;
        }
        Ok(())
    }

    fn draw_x_ticks(
        &mut self,
        rect: &geom::Rect,
        x_ticks: &Ticks,
        x_cm: &dyn scale::CoordMap,
    ) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        let transform = geom::Transform::from_translate(rect.left(), rect.bottom());
        self.draw_ticks_path(&x_ticks.locs, x_cm, &transform)?;

        let fill = x_ticks.color.into();

        for (xt, lbl) in x_ticks.locs.iter().copied().zip(x_ticks.lbls.iter()) {
            let font = x_ticks.font.clone();

            let x = x_cm.map_coord(xt);
            let x = rect.left() + x;
            let pos = geom::Point::new(
                x,
                rect.bottom() + missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN,
            );
            let anchor = render::TextAnchor {
                pos,
                align: render::TextAlign::Center,
                baseline: render::TextBaseline::Hanging,
            };
            let text = render::Text {
                text: lbl.clone(),
                font,
                anchor,
                fill,
                transform: None,
            };

            self.surface.draw_text(&text)?;
        }

        if let Some(annot) = x_ticks.annot.as_ref() {
            let font = x_ticks
                .font
                .clone()
                .with_family(missing_params::AXIS_ANNOT_FONT_FAMILY.into());
            let pos = geom::Point::new(
                rect.right(),
                rect.bottom()
                    + missing_params::TICK_SIZE
                    + missing_params::TICK_LABEL_MARGIN
                    + font.size(),
            );
            let anchor = render::TextAnchor {
                pos,
                align: render::TextAlign::End,
                baseline: render::TextBaseline::Hanging,
            };
            let text = render::Text {
                text: annot.into(),
                font,
                anchor,
                fill,
                transform: None,
            };
            self.surface.draw_text(&text)?;
        }

        Ok(())
    }

    fn draw_y_axis(
        &mut self,
        y_axis: &Axis,
        rect: &geom::Rect,
    ) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        if let Some(y_ticks) = y_axis.ticks.as_ref() {
            self.draw_y_ticks(rect, y_ticks, y_axis)?;
        }
        if let Some(label) = y_axis.label.as_ref() {
            let font = style::Font::new(
                missing_params::AXIS_LABEL_FONT_FAMILY.into(),
                missing_params::AXIS_LABEL_FONT_SIZE,
            );

            // we render at origin, but translate to correct position and rotate
            let anchor = render::TextAnchor {
                pos: geom::Point::ORIGIN,
                align: render::TextAlign::Center,
                baseline: render::TextBaseline::Hanging,
            };

            let tx = rect.left() - y_axis.ortho_sz + missing_params::AXIS_LABEL_MARGIN;
            let ty = rect.center_y();
            let transform = Some(geom::Transform::from_translate(tx, ty).pre_rotate(90.0));
            let text = render::Text {
                text: label.clone(),
                font,
                anchor,
                fill: missing_params::AXIS_LABEL_COLOR.into(),
                transform,
            };
            self.surface.draw_text(&text)?;
        }
        Ok(())
    }

    fn draw_y_ticks(
        &mut self,
        rect: &geom::Rect,
        y_ticks: &Ticks,
        y_cm: &dyn CoordMap,
    ) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        let transform =
            geom::Transform::from_translate(rect.left(), rect.bottom()).pre_rotate(90.0);
        self.draw_ticks_path(&y_ticks.locs, y_cm, &transform)?;

        let fill = y_ticks.color.into();
        let y_vb = y_cm.view_bounds();

        for (yt, lbl) in y_ticks.locs.iter().copied().zip(y_ticks.lbls.iter()) {
            if !y_vb.contains(yt) {
                continue;
            }
            let font = y_ticks.font.clone();
            let x = rect.left() - missing_params::TICK_SIZE - missing_params::TICK_LABEL_MARGIN;
            let y = rect.bottom() - y_cm.map_coord(yt);
            let pos = geom::Point::new(x, y);
            let anchor = render::TextAnchor {
                pos,
                align: render::TextAlign::End,
                baseline: render::TextBaseline::Center,
            };
            let text = render::Text {
                text: lbl.clone(),
                font,
                anchor,
                fill,
                transform: None,
            };
            self.surface.draw_text(&text)?;
        }
        Ok(())
    }

    fn draw_ticks_path(
        &mut self,
        ticks: &[f64],
        cm: &dyn CoordMap,
        transform: &geom::Transform,
    ) -> Result<(), S::Error>
    where
        S: render::Surface,
    {
        let ticks_path = ticks_path(&ticks, cm, None);
        let ticks_path = render::Path {
            path: ticks_path,
            fill: None,
            stroke: Some(missing_params::TICK_COLOR.into()),
            transform: Some(*transform),
        };
        self.surface.draw_path(&ticks_path)?;
        Ok(())
    }
}

/// Build the ticks path along X axis.
/// Y axis will use the same function and rotate 90°
fn ticks_path(
    ticks: &[f64],
    cm: &dyn scale::CoordMap,
    reuse_alloc: Option<geom::Path>,
) -> geom::Path {
    let sz = missing_params::TICK_SIZE;
    let mut path = reuse_alloc
        .map(|p| p.clear())
        .unwrap_or_else(geom::PathBuilder::new);
    let vb = cm.view_bounds();
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
