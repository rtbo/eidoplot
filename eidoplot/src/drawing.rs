use std::fmt;
use std::sync::Arc;

use text::fontdb;

use crate::style::Theme;
use crate::{data, geom, ir, render, text};

mod axis;
mod figure;
mod legend;
mod marker;
mod plot;
mod scale;
mod series;
mod ticks;

#[derive(Debug)]
pub enum Error {
    Render(render::Error),
    MissingDataSrc(String),
    UnknownAxisRef(ir::axis::Ref),
    IllegalAxisRef(ir::axis::Ref),
    UnboundedAxis,
    InconsistentIr(String),
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
            Error::UnknownAxisRef(axis_ref) => write!(f, "Unknown axis reference: {:?}", axis_ref),
            Error::IllegalAxisRef(axis_ref) => write!(f, "Illegal axis reference: {:?}", axis_ref),
            Error::UnboundedAxis => write!(f, "Unbounded axis, check data"),
            Error::InconsistentIr(reason) => write!(f, "Inconsistent IR: {}", reason),
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

impl From<Arc<fontdb::Database>> for Options {
    fn from(fontdb: Arc<fontdb::Database>) -> Self {
        Options {
            fontdb: Some(fontdb),
        }
    }
}

pub trait SurfaceExt: render::Surface {
    fn draw_figure<D>(
        &mut self,
        figure: &ir::Figure,
        data_source: &D,
        theme: &Theme,
        opts: Options,
    ) -> Result<(), Error>
    where
        D: data::Source,
    {
        let fontdb = opts
            .fontdb
            .unwrap_or_else(|| Arc::new(crate::bundled_font_db()));
        let ctx = Ctx::new(data_source, theme, fontdb);
        let mut wrapper = SurfWrapper { surface: self };
        wrapper.draw_toplevel_figure(&ctx, figure)?;
        Ok(())
    }
}

impl<T> SurfaceExt for T where T: render::Surface {}

#[derive(Debug)]
struct Ctx<'a, D> {
    data_source: &'a D,
    theme: &'a Theme,
    fontdb: Arc<fontdb::Database>,
}

impl<'a, D> Ctx<'a, D> {
    pub fn new(data_source: &'a D, theme: &'a Theme, fontdb: Arc<fontdb::Database>) -> Ctx<'a, D> {
        Ctx {
            data_source,
            theme,
            fontdb,
        }
    }

    pub fn data_source(&self) -> &D {
        self.data_source
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn fontdb(&self) -> &Arc<fontdb::Database> {
        &self.fontdb
    }
}

struct SurfWrapper<'a, S: ?Sized> {
    surface: &'a mut S,
}

impl<'a, S: ?Sized> render::Surface for SurfWrapper<'a, S>
where
    S: render::Surface,
{
    fn prepare(&mut self, size: geom::Size) -> Result<(), render::Error> {
        self.surface.prepare(size)
    }

    fn fill(&mut self, fill: render::Paint) -> Result<(), render::Error> {
        self.surface.fill(fill)
    }

    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), render::Error> {
        self.surface.draw_rect(rect)
    }

    fn draw_line_text(&mut self, text: &render::LineText) -> Result<(), render::Error> {
        self.surface.draw_line_text(text)
    }

    fn draw_rich_text(&mut self, text: &render::RichText) -> Result<(), render::Error> {
        self.surface.draw_rich_text(text)
    }

    fn draw_text(&mut self, text: &render::Text) -> Result<(), render::Error> {
        self.surface.draw_text(text)
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

trait F64ColumnExt: data::F64Column {
    fn bounds(&self) -> Option<axis::NumBounds> {
        self.minmax().map(|(min, max)| (min, max).into())
    }
}

impl<T> F64ColumnExt for T where T: data::F64Column + ?Sized {}

trait ColumnExt: data::Column {
    fn bounds(&self) -> Option<axis::Bounds> {
        if let Some(time) = self.time() {
            time.minmax()
                .map(|(min, max)| axis::Bounds::Time((min, max).into()))
        } else if let Some(num) = self.f64() {
            num.minmax()
                .map(|(min, max)| axis::Bounds::Num((min, max).into()))
        } else if let Some(cats) = self.str() {
            Some(axis::Bounds::Cat(cats.into()))
        } else {
            None
        }
    }
}

impl<T> ColumnExt for T where T: data::Column + ?Sized {}

#[derive(Debug, Clone, PartialEq)]
struct Category(String);

#[derive(Debug, Clone, PartialEq)]
struct Categories {
    cats: Vec<Category>,
}

impl Categories {
    fn new() -> Self {
        Categories { cats: Vec::new() }
    }

    fn len(&self) -> usize {
        self.cats.len()
    }

    fn _is_empty(&self) -> bool {
        self.cats.is_empty()
    }

    fn iter(&self) -> impl Iterator<Item = &str> {
        self.cats.iter().map(|c| c.0.as_str())
    }

    fn _get(&self, idx: usize) -> Option<&str> {
        self.cats.get(idx).map(|c| c.0.as_str())
    }

    fn _contains(&self, cat: &str) -> bool {
        self.cats.iter().any(|c| c.0 == cat)
    }

    fn push_if_not_present(&mut self, cat: &str) {
        if self.cats.iter().any(|c| c.0 == cat) {
            return;
        }
        self.cats.push(Category(cat.to_string()));
    }
}

impl From<&dyn data::StrColumn> for Categories {
    fn from(col: &dyn data::StrColumn) -> Self {
        let mut cats = Categories::new();
        for s in col.iter() {
            if let Some(s) = s {
                cats.push_if_not_present(s);
            }
        }
        cats
    }
}
