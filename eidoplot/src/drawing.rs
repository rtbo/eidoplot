use std::sync::Arc;

use crate::style::{self, defaults};
use crate::{data, geom, ir, missing_params, render};

mod ctx;
mod scale;
mod ticks;

use ctx::Ctx;
use scale::CoordMap;

#[derive(Debug, Default, Clone)]
pub struct Options {
    pub fontdb: Option<Arc<fontdb::Database>>,
}

pub trait FigureExt {
    fn draw<S: render::Surface>(&self, surface: &mut S, opts: Options) -> Result<(), S::Error>;
}

trait CalcViewBounds {
    fn calc_view_bounds(&self) -> (data::ViewBounds, data::ViewBounds);
}

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

impl FigureExt for ir::Figure {
    fn draw<S: render::Surface>(&self, surface: &mut S, opts: Options) -> Result<(), S::Error> {
        let fontdb = opts.fontdb.unwrap_or_else(crate::default_font_db);
        let mut ctx = Ctx { surface, fontdb };
        draw_figure(&mut ctx, self)?;
        Ok(())
    }
}

fn draw_figure<S>(ctx: &mut Ctx<S>, fig: &ir::Figure) -> Result<(), S::Error>
where
    S: render::Surface,
{
    ctx.surface.prepare(fig.size())?;
    if let Some(fill) = fig.fill() {
        ctx.surface.fill(fill)?;
    }

    let mut rect = geom::Rect::from_ps(geom::Point::ORIGIN, fig.size());
    let layout = fig.layout().cloned().unwrap_or_default();
    if let Some(padding) = layout.padding() {
        rect = rect.pad(padding);
    }

    if let Some(title) = fig.title() {
        let mut title = title.clone();

        if title.font().is_none() {
            title = title.with_font(style::Font::new(
                defaults::TITLE_FONT_FAMILY.into(),
                defaults::TITLE_FONT_SIZE,
            ));
        }
        let font = title.font().cloned().unwrap();
        let font_size = font.size();
        let title_rect = geom::Rect::from_xywh(
            rect.x(),
            rect.y(),
            rect.width(),
            font_size + 2.0 * missing_params::FIG_TITLE_MARGIN,
        );
        let text = render::Text {
            text: title.text().to_string(),
            font: title.font().unwrap().clone(),
            fill: missing_params::FIG_TITLE_COLOR.into(),
            anchor: render::TextAnchor {
                pos: title_rect.center(),
                align: render::TextAlign::Center,
                baseline: render::TextBaseline::Center,
            },
            transform: None,
        };
        ctx.surface.draw_text(&text)?;
        rect = rect.shifted_top(title_rect.height());
    }

    draw_figure_plots(ctx, fig.plots(), &rect)?;

    Ok(())
}

fn draw_figure_plots<S>(
    ctx: &mut Ctx<S>,
    plots: &ir::figure::Plots,
    rect: &geom::Rect,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    match plots {
        ir::figure::Plots::Plot(plot) => draw_plot(ctx, plot, rect),
        ir::figure::Plots::Subplots(subplots) => {
            let w =
                (rect.width() - subplots.space * (subplots.cols - 1) as f32) / subplots.cols as f32;
            let h = (rect.height() - subplots.space * (subplots.rows - 1) as f32)
                / subplots.rows as f32;
            let mut y = rect.y();
            for c in 0..subplots.cols {
                let mut x = rect.x();
                for r in 0..subplots.rows {
                    let cols = subplots.cols as u32;
                    let idx = (r * cols + c) as usize;
                    let plot = &subplots.plots[idx];
                    draw_plot(ctx, plot, &geom::Rect::from_xywh(x, y, w, h))?;
                    x += w + subplots.space;
                }
                y += h + subplots.space;
            }
            Ok(())
        }
    }
}

struct CoordMapXy {
    x: Box<dyn scale::CoordMap>,
    y: Box<dyn scale::CoordMap>,
}

impl CoordMapXy {
    fn map_coord(&self, dp: (f64, f64)) -> (f32, f32) {
        (self.x.map_coord(dp.0), self.y.map_coord(dp.1))
    }
}

fn draw_plot<S>(ctx: &mut Ctx<S>, plot: &ir::Plot, rect: &geom::Rect) -> Result<(), S::Error>
where
    S: render::Surface,
{
    let axis_padding = missing_params::PLOT_AXIS_PADDING;
    let rect = rect.pad(&axis_padding);

    // initialize view bounds to view the whole data
    let vb = plot.calc_view_bounds();
    let cm = CoordMapXy {
        x: scale::map_scale_coord(&plot.x_axis.scale, rect.width(), vb.0),
        y: scale::map_scale_coord(&plot.y_axis.scale, rect.height(), vb.1),
    };

    draw_plot_background(ctx, plot, &rect)?;
    draw_plot_series(ctx, plot, &rect, &cm)?;
    draw_x_axis(ctx, &plot.x_axis, &rect, &*cm.x)?;
    draw_y_axis(ctx, &plot.y_axis, &rect, &*cm.y)?;
    draw_plot_border(ctx, plot.border.as_ref(), &rect)?;

    Ok(())
}

fn draw_plot_background<S>(
    ctx: &mut Ctx<S>,
    plot: &ir::Plot,
    rect: &geom::Rect,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    if let Some(fill) = plot.fill.as_ref() {
        ctx.surface.draw_rect(&render::Rect {
            rect: *rect,
            fill: Some(fill.clone()),
            stroke: None,
            transform: None,
        })?;
    }
    Ok(())
}

fn draw_plot_border<S>(
    ctx: &mut Ctx<S>,
    border: Option<&ir::plot::Border>,
    rect: &geom::Rect,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    match border {
        None => Ok(()),
        Some(ir::plot::Border::Box(stroke)) => ctx.surface.draw_rect(&render::Rect {
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
            ctx.surface.draw_path(&path)
        }
        Some(ir::plot::Border::AxisArrow { .. }) => {
            todo!("Draw axis arrow")
        }
    }
}

fn draw_plot_series<S>(
    ctx: &mut Ctx<S>,
    plot: &ir::Plot,
    rect: &geom::Rect,
    cm: &CoordMapXy,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    ctx.surface.push_clip(&render::Clip {
        path: rect.to_path(),
        transform: None,
    })?;
    for series in &plot.series {
        draw_series_plot(ctx, &series.plot, rect, cm)?;
    }
    ctx.surface.pop_clip()?;
    Ok(())
}

fn draw_x_axis<S>(
    ctx: &mut Ctx<S>,
    x_axis: &ir::Axis,
    rect: &geom::Rect,
    x_cm: &dyn scale::CoordMap,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    let mut label_y = rect.bottom() + missing_params::AXIS_LABEL_MARGIN;
    if let Some(x_ticks) = x_axis.ticks.as_ref() {
        draw_x_ticks(ctx, &rect, x_ticks, x_cm)?;
        label_y +=
            missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN + x_ticks.font().size();
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
        ctx.surface.draw_text(&text)?;
    }
    Ok(())
}

fn draw_x_ticks<S>(
    ctx: &mut Ctx<S>,
    rect: &geom::Rect,
    x_ticks: &ir::axis::Ticks,
    x_cm: &dyn scale::CoordMap,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    let x_vb = x_cm.view_bounds();
    let ticks = ticks::locate(x_ticks.locator(), x_vb);
    let transform = geom::Transform::from_translate(rect.left(), rect.bottom());
    draw_ticks_path(ctx, &ticks, &x_vb, x_cm, &transform)?;

    let lbl_formatter = ticks::label_formatter(&x_ticks, x_vb);
    let fill = x_ticks.color().into();

    for xt in ticks.iter().copied() {
        let text = lbl_formatter.format_label(xt);
        let font = x_ticks.font().clone();

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
            text,
            font,
            anchor,
            fill,
            transform: None,
        };

        ctx.surface.draw_text(&text)?;
    }

    if let Some(annot) = lbl_formatter.axis_annotation() {
        let font = x_ticks.font().clone();
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
        ctx.surface.draw_text(&text)?;
    }

    Ok(())
}

fn draw_y_axis<S>(
    ctx: &mut Ctx<S>,
    y_axis: &ir::Axis,
    rect: &geom::Rect,
    y_cm: &dyn CoordMap,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    if let Some(y_ticks) = y_axis.ticks.as_ref() {
        draw_y_ticks(ctx, rect, y_ticks, y_cm)?;
    }
    Ok(())
}

fn draw_y_ticks<S>(
    ctx: &mut Ctx<S>,
    rect: &geom::Rect,
    y_ticks: &ir::axis::Ticks,
    y_cm: &dyn CoordMap,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    let y_vb = y_cm.view_bounds();
    let ticks = ticks::locate(y_ticks.locator(), y_vb);
    let transform = geom::Transform::from_translate(rect.left(), rect.bottom()).pre_rotate(90.0);
    draw_ticks_path(ctx, &ticks, &y_vb, y_cm, &transform)?;

    let lbl_formatter = ticks::label_formatter(&y_ticks, y_vb);
    let fill = y_ticks.color().into();

    for yt in ticks.iter().copied() {
        if !y_vb.contains(yt) {
            continue;
        }
        let text = lbl_formatter.format_label(yt);
        let font = y_ticks.font().clone();
        let x = rect.left() - missing_params::TICK_SIZE - missing_params::TICK_LABEL_MARGIN;
        let y = rect.bottom() - y_cm.map_coord(yt);
        let pos = geom::Point::new(x, y);
        let anchor = render::TextAnchor {
            pos,
            align: render::TextAlign::End,
            baseline: render::TextBaseline::Center,
        };
        let text = render::Text {
            text,
            font,
            anchor,
            fill,
            transform: None,
        };
        ctx.surface.draw_text(&text)?;
    }
    Ok(())
}

/// Build the ticks path along X axis.
/// Y axis will use the same function and rotate 90°
fn ticks_path(
    ticks: &[f64],
    vb: &data::ViewBounds,
    cm: &dyn scale::CoordMap,
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

fn draw_ticks_path<S>(
    ctx: &mut Ctx<S>,
    ticks: &[f64],
    vb: &data::ViewBounds,
    cm: &dyn CoordMap,
    transform: &geom::Transform,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    let ticks_path = ticks_path(&ticks, &vb, cm, None);
    let ticks_path = render::Path {
        path: ticks_path,
        fill: None,
        stroke: Some(missing_params::TICK_COLOR.into()),
        transform: Some(*transform),
    };
    ctx.surface.draw_path(&ticks_path)?;
    Ok(())
}

fn draw_series_plot<S>(
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
