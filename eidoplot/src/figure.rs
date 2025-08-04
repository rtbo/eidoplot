use crate::backend;
use crate::geom;
use crate::plots::Plots;
use crate::style;
use crate::style::color;

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
    pub title: Option<String>,
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

impl Figure {
    pub fn draw<S>(&self, surface: &mut S) -> Result<(), S::Error>
    where
        S: backend::Surface,
    {
        surface.prepare(self.size.w, self.size.h)?;
        if let Some(fill) = &self.fill {
            surface.fill(fill.color)?;
        }
        if let Some(plots) = &self.plots {
            let rect = self.rect().pad(&self.padding);
            plots.draw(surface, &rect)?
        }
        Ok(())
    }
}
