use crate::data;
use crate::drawing::{CalcViewBounds, Ctx, scale, series, ticks};
use crate::geom;
use crate::ir;
use crate::missing_params;
use crate::render;
use crate::style::{self, defaults};

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

pub fn draw_plot<S>(ctx: &mut Ctx<S>, plot: &ir::Plot, rect: &geom::Rect) -> Result<(), S::Error>
where
    S: render::Surface,
{
    let rect = rect.pad(&missing_params::PLOT_PADDING);

    let vb = plot.calc_view_bounds();
    let insets = plot_insets(plot);

    // x-axis height only depends on font size, so it can be computed right-away,
    // y-axis width depends on font width therefore we have to generate tick labels,
    // which somehow depends on the x-axis height (for available space)

    let x_height = ctx.calculate_x_axis_height(&plot.x_axis);
    let rect = rect.shifted_bottom_side(-x_height);

    let y_cm = scale::map_scale_coord(&plot.y_axis.scale, rect.height(), vb.1, insets.1);
    // we have to gather the ticks now
    let y_ticks = plot
        .y_axis
        .ticks
        .as_ref()
        .map(|t| ticks::collect_ticks(t, y_cm.view_bounds()));
    let y_width = ctx.calculate_y_axis_width(&plot.y_axis, y_ticks.as_deref());
    let rect = rect.shifted_left_side(y_width);

    let x_cm = scale::map_scale_coord(&plot.x_axis.scale, rect.width(), vb.0, insets.0);

    // initialize view bounds to view the whole data
    let cm = CoordMapXy { x: x_cm, y: y_cm };

    draw_plot_background(ctx, plot, &rect)?;
    draw_plot_series(ctx, plot, &rect, &cm)?;
    draw_x_axis(ctx, &plot.x_axis, &rect, &*cm.x)?;
    draw_y_axis(ctx, &plot.y_axis, &rect, &*cm.y, y_width)?;
    draw_plot_border(ctx, plot.border.as_ref(), &rect)?;

    Ok(())
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
        series::draw_series_plot(ctx, &series.plot, rect, cm)?;
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
        let font = x_ticks
            .font()
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
        ctx.surface.draw_text(&text)?;
    }

    Ok(())
}

fn draw_y_axis<S>(
    ctx: &mut Ctx<S>,
    y_axis: &ir::Axis,
    rect: &geom::Rect,
    y_cm: &dyn CoordMap,
    y_width: f32,
) -> Result<(), S::Error>
where
    S: render::Surface,
{
    if let Some(y_ticks) = y_axis.ticks.as_ref() {
        draw_y_ticks(ctx, rect, y_ticks, y_cm)?;
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

        let tx = rect.left() - y_width + missing_params::AXIS_LABEL_MARGIN;
        let ty = rect.center_y();
        let transform = Some(geom::Transform::from_translate(tx, ty).pre_rotate(90.0));
        let text = render::Text {
            text: label.clone(),
            font,
            anchor,
            fill: missing_params::AXIS_LABEL_COLOR.into(),
            transform,
        };
        ctx.surface.draw_text(&text)?;
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
