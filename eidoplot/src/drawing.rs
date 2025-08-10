use std::sync::Arc;

use crate::{ir, render, style};

mod fdb;
mod figure;
mod legend;
mod plot;
mod scale;
mod series;
mod ticks;

use fdb::FontDb;

#[derive(Debug, Default, Clone)]
pub struct Options {
    pub fontdb: Option<Arc<fontdb::Database>>,
}

pub trait FigureExt {
    fn draw<S: render::Surface>(&self, surface: &mut S, opts: Options) -> Result<(), S::Error>;
}

impl FigureExt for ir::Figure {
    fn draw<S: render::Surface>(&self, surface: &mut S, opts: Options) -> Result<(), S::Error> {
        let fontdb = opts.fontdb.unwrap_or_else(crate::bundled_font_db);
        let fontdb = FontDb::new(fontdb);
        let mut ctx = Ctx::new(surface, fontdb);
        ctx.draw_figure(self)?;
        Ok(())
    }
}

#[derive(Debug)]
struct Ctx<'a, S> {
    surface: &'a mut S,
    fontdb: FontDb,
}

impl<'a, S> Ctx<'a, S> {
    pub fn new(surface: &'a mut S, fontdb: FontDb) -> Ctx<'a, S> {
        Ctx { surface, fontdb }
    }

    pub fn fontdb(&self) -> &FontDb {
        &self.fontdb
    }
}

impl<'a, S> render::Surface for Ctx<'a, S>
where
    S: render::Surface,
{
    type Error = S::Error;

    fn prepare(&mut self, size: crate::geom::Size) -> Result<(), Self::Error> {
        self.surface.prepare(size)
    }

    fn fill(&mut self, fill: style::Fill) -> Result<(), Self::Error> {
        self.surface.fill(fill)
    }

    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), Self::Error> {
        self.surface.draw_rect(rect)
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), Self::Error> {
        self.surface.draw_path(path)
    }

    fn draw_text(&mut self, text: &render::Text) -> Result<(), Self::Error> {
        self.surface.draw_text(text)
    }

    fn push_clip(&mut self, clip: &render::Clip) -> Result<(), Self::Error> {
        self.surface.push_clip(clip)
    }

    fn pop_clip(&mut self) -> Result<(), Self::Error> {
        self.surface.pop_clip()
    }
}
