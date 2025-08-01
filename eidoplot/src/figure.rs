use crate::backend;
use crate::plots::Plots;
use crate::style;

pub struct FigSize {
    pub w: f32,
    pub h: f32,
}

impl Default for FigSize {
    fn default() -> Self {
        FigSize { w: 800.0, h: 600.0 }
    }
}

pub struct Figure {
    pub size: FigSize,
    pub title: Option<String>,
    pub plots: Plots,
    pub fill: Option<style::Fill>,
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
        self.plots.draw(surface)?;
        Ok(())
    }
}
