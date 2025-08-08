use crate::drawing::Ctx;
use crate::geom;
use crate::ir;
use crate::missing_params;
use crate::render;
use crate::style::{self, defaults};

pub fn draw_figure<S>(ctx: &mut Ctx<S>, fig: &ir::Figure) -> Result<(), S::Error>
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
        rect = rect.shifted_top_side(title_rect.height());
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
        ir::figure::Plots::Plot(plot) => ctx.draw_plot(plot, rect),
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
                    ctx.draw_plot(plot, &geom::Rect::from_xywh(x, y, w, h))?;
                    x += w + subplots.space;
                }
                y += h + subplots.space;
            }
            Ok(())
        }
    }
}
