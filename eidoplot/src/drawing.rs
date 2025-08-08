use std::sync::Arc;

use crate::{data, ir, render};

mod ctx;
mod figure;
mod plot;
mod scale;
mod ticks;
mod series;

use ctx::Ctx;

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
        let mut ctx = Ctx::new(surface, fontdb);
        ctx.draw_figure( self)?;
        Ok(())
    }
}

trait CalcViewBounds {
    fn calc_view_bounds(&self) -> (data::ViewBounds, data::ViewBounds);
}
