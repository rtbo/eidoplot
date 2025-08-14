use std::{fmt, sync::Arc};

use crate::{data, ir, render};

mod axis;
mod fdb;
mod figure;
mod legend;
mod plot;
mod scale;
mod series;
mod ticks;

use fdb::FontDb;

#[derive(Debug)]
pub enum Error {
    Render(render::Error),
    MissingDataSrc(String),
    UnboundedAxis,
    InconsistentAxisBounds(String),
    InconsistentData(String),
}

impl From<render::Error> for Error {
    fn from(err: render::Error) -> Self {
        Error::Render(err)
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
        }
    }
}

impl std::error::Error for Error {}

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
        let ctx = Ctx::new(data_source, fontdb);
        ctx.draw_figure(surface, self)?;
        Ok(())
    }
}

#[derive(Debug)]
struct Ctx<'a, D> {
    data_source: &'a D,
    fontdb: FontDb,
}

impl<'a, D> Ctx<'a, D> {
    pub fn new(data_source: &'a D, fontdb: FontDb) -> Ctx<'a, D> {
        Ctx {
            data_source,
            fontdb,
        }
    }

    pub fn data_source(&self) -> &'a D {
        self.data_source
    }

    pub fn fontdb(&self) -> &FontDb {
        &self.fontdb
    }
}

trait F64ColumnExt: data::F64Column {
    fn bounds(&self) -> Option<axis::NumBounds> {
        self.minmax().map(|mm| mm.into())
    }
}

impl<T> F64ColumnExt for T where T: data::F64Column + ?Sized {}
