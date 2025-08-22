use std::fmt;
use std::sync::Arc;

use eidoplot_text as text;
use text::fontdb;

use crate::{data, ir, render};

mod axis;
mod figure;
mod legend;
mod plot;
mod scale;
mod series;
mod ticks;

#[derive(Debug)]
pub enum Error {
    Render(render::Error),
    MissingDataSrc(String),
    UnboundedAxis,
    InconsistentAxisBounds(String),
    InconsistentData(String),
    FontOrText(text::Error),
}

impl From<render::Error> for Error {
    fn from(err: render::Error) -> Self {
        Error::Render(err)
    }
}

impl From<text::Error> for Error {
    fn from(err: text::Error) -> Self {
        Error::FontOrText(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Render(err) => err.fmt(f),
            Error::MissingDataSrc(name) => write!(f, "Missing data source: {}", name),
            Error::UnboundedAxis => write!(f, "Unbounded axis, check data"),
            Error::InconsistentAxisBounds(reason) => {
                write!(f, "Inconsistent axis bounds: {}", reason)
            }
            Error::InconsistentData(reason) => write!(f, "Inconsistent data: {}", reason),
            Error::FontOrText(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Default, Clone)]
pub struct Options {
    pub fontdb: Option<Arc<fontdb::Database>>,
}

#[derive(Debug)]
struct Ctx<'a, D> {
    data_source: &'a D,
    fontdb: Arc<fontdb::Database>,
}

impl<'a, D> Ctx<'a, D> {
    pub fn new(data_source: &'a D, fontdb: Arc<fontdb::Database>) -> Ctx<'a, D> {
        Ctx {
            data_source,
            fontdb,
        }
    }

    pub fn data_source(&self) -> &'a D {
        self.data_source
    }

    pub fn fontdb(&self) -> &Arc<fontdb::Database> {
        &self.fontdb
    }
}

pub trait SurfaceExt: render::Surface {
    fn draw_figure<D>(
        &mut self,
        figure: &ir::Figure,
        data_source: &D,
        opts: Options,
    ) -> Result<(), Error>
    where
        D: data::Source,
    {
        let fontdb = opts
            .fontdb
            .unwrap_or_else(|| Arc::new(crate::bundled_font_db()));
        let ctx = Ctx::new(data_source, fontdb);
        let mut wrapper = SurfWrapper { surface: self };
        wrapper.draw_toplevel_figure(&ctx, figure)?;
        Ok(())
    }
}

impl<T> SurfaceExt for T where T: render::Surface {}

trait F64ColumnExt: data::F64Column {
    fn bounds(&self) -> Option<axis::NumBounds> {
        self.minmax().map(|mm| mm.into())
    }
}

impl<T> F64ColumnExt for T where T: data::F64Column + ?Sized {}

struct SurfWrapper<'a, S: ?Sized> {
    surface: &'a mut S,
}

impl<'a, S: ?Sized> render::Surface for SurfWrapper<'a, S>
where
    S: render::Surface,
{
    fn prepare(&mut self, size: crate::geom::Size) -> Result<(), render::Error> {
        self.surface.prepare(size)
    }

    fn fill(&mut self, fill: crate::style::Fill) -> Result<(), render::Error> {
        self.surface.fill(fill)
    }

    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), render::Error> {
        self.surface.draw_rect(rect)
    }

    fn draw_text(&mut self, text: &render::Text) -> Result<(), render::Error> {
        self.surface.draw_text(text)
    }

    fn draw_text_layout(&mut self, text: &render::TextLayout) -> Result<(), render::Error> {
        self.surface.draw_text_layout(text)
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), render::Error> {
        self.surface.draw_path(path)
    }

    fn push_clip(&mut self, clip: &render::Clip) -> Result<(), render::Error> {
        self.surface.push_clip(clip)
    }

    fn pop_clip(&mut self) -> Result<(), render::Error> {
        self.surface.pop_clip()
    }
}
