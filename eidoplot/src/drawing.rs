use std::sync::Arc;

use crate::{data, ir, render, style};

mod fdb;
mod figure;
mod legend;
mod plot;
mod scale;
mod series;
mod ticks;

use fdb::FontDb;

#[derive(Debug)]
pub enum Error 
{
    Render(render::Error),
    Data(data::Error),
}

impl From<render::Error> for Error
{
    fn from(err: render::Error) -> Self {
        Error::Render(err)
    }
}

impl From<data::Error> for Error 
{
    fn from(err: data::Error) -> Self {
        Error::Data(err)
    }
} 

#[derive(Debug, Default, Clone)]
pub struct Options {
    pub fontdb: Option<Arc<fontdb::Database>>,
}

pub trait FigureExt {
    fn draw<S, D>(&self, surface: &mut S, data_source: &D, opts: Options) -> Result<(), Error>
    where
        S: render::Surface,
        D: data::Source;
}

impl FigureExt for ir::Figure {
    fn draw<S, D>(&self, surface: &mut S, data_source: &D, opts: Options) -> Result<(), Error>
    where
        S: render::Surface,
        D: data::Source,
    {
        let fontdb = opts.fontdb.unwrap_or_else(crate::bundled_font_db);
        let fontdb = FontDb::new(fontdb);
        let mut ctx = Ctx::new(surface, data_source, fontdb);
        ctx.draw_figure(self)?;
        Ok(())
    }
}

#[derive(Debug)]
struct Ctx<'a, S, D> {
    surface: &'a mut S,
    data_source: &'a D,
    fontdb: FontDb,
}

impl<'a, S, D> Ctx<'a, S, D> {
    pub fn new(surface: &'a mut S, data_source: &'a D, fontdb: FontDb) -> Ctx<'a, S, D> {
        Ctx {
            surface,
            data_source,
            fontdb,
        }
    }

    pub fn data_source(&self) -> &D {
        self.data_source
    }

    pub fn fontdb(&self) -> &FontDb {
        &self.fontdb
    }
}

impl<'a, S, D> render::Surface for Ctx<'a, S, D>
where
    S: render::Surface,
{
    fn prepare(&mut self, size: crate::geom::Size) -> Result<(), render::Error> {
        self.surface.prepare(size)
    }

    fn fill(&mut self, fill: style::Fill) -> Result<(), render::Error> {
        self.surface.fill(fill)
    }

    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), render::Error> {
        self.surface.draw_rect(rect)
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), render::Error> {
        self.surface.draw_path(path)
    }

    fn draw_text(&mut self, text: &render::Text) -> Result<(), render::Error> {
        self.surface.draw_text(text)
    }

    fn push_clip(&mut self, clip: &render::Clip) -> Result<(), render::Error> {
        self.surface.push_clip(clip)
    }

    fn pop_clip(&mut self) -> Result<(), render::Error> {
        self.surface.pop_clip()
    }
}
