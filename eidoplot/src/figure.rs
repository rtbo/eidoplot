use crate::backend;
use crate::geom;
use crate::missing_params;
use crate::plots::Plots;
use crate::render;
use crate::style;
use crate::style::color;
use crate::text::Font;
use crate::text::Text;
use crate::text::DEFAULT_FONT_FAMILY;

#[derive(Debug, Clone, Copy)]
pub struct FigSize {
    pub w: f32,
    pub h: f32,
}

impl Default for FigSize {
    fn default() -> Self {
        FigSize { w: 800.0, h: 600.0 }
    }
}

#[derive(Debug, Clone)]
pub struct Figure {
    pub size: FigSize,
    pub title: Option<Text>,
    pub fill: Option<style::Fill>,
    pub padding: geom::Padding,
    pub plots: Option<Plots>,
}

impl Default for Figure {
    fn default() -> Self {
        Figure {
            size: FigSize::default(),
            title: None,
            fill: Some(color::WHITE.into()),
            padding: 10.0.into(),
            plots: None,
        }
    }
}

impl Figure {
    fn rect(&self) -> geom::Rect {
        geom::Rect::from_xywh(0.0, 0.0, self.size.w, self.size.h)
    }
}

const DEFAULT_TITLE_FONT_FAMILY: &str = DEFAULT_FONT_FAMILY;
const DEFAULT_TITLE_FONT_SIZE: f32 = 32.0;

impl Figure {
    pub fn draw<S>(&self, surface: &mut S) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        surface.prepare(self.size.w, self.size.h)?;
        if let Some(fill) = &self.fill {
            surface.fill(fill.color)?;
        }

        let mut rect = self.rect().pad(&self.padding);

        if let Some(title) = &self.title {
            let mut title = title.clone();
            if title.font().is_none() {
                title = title.with_font(Font::new(
                    DEFAULT_TITLE_FONT_FAMILY.into(),
                    DEFAULT_TITLE_FONT_SIZE,
                ));
            }

            let font_size = title.font().unwrap().size();
            let title_rect = geom::Rect::from_xywh(
                rect.x(),
                rect.y() + missing_params::FIG_TITLE_MARGIN,
                rect.width(),
                font_size + 2.0 * missing_params::FIG_TITLE_MARGIN,
            );
            let text = render::Text {
                text: title.clone(),
                fill: missing_params::FIG_TITLE_COLOR.into(),
                anchor: render::TextAnchor {
                    pos: title_rect.center(),
                    align: render::TextAlign::Center,
                    baseline: render::TextBaseline::Center,
                },
                transform: None,
            };
            surface.draw_text(&text)?;
            rect = rect.pad(&geom::Padding::Custom {
                t: title_rect.height(),
                r: 0.0,
                b: 0.0,
                l: 0.0,
            });
        }

        if let Some(plots) = &self.plots {
            plots.draw(surface, &rect)?
        }
        Ok(())
    }
}
