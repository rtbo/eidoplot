//! Drawing module
//!
//! This module contains all the logic to convert an IR figure into rendering commands
//! for a given rendering surface.
//! It is the bridge between the [`ir`] module and the [`render`] module.
use std::fmt;

use text::fontdb;

use crate::style::theme::Theme;
use crate::style::{self, theme};
use crate::{Style, data, geom, ir, render, text};

mod axis;
mod figure;
mod legend;
mod marker;
mod plot;
mod scale;
mod series;
mod ticks;

pub use figure::Figure;

/// Errors that can occur during figure drawing
#[derive(Debug)]
pub enum Error {
    /// Error during rendering
    Render(render::Error),
    /// A series references a missing data source
    MissingDataSrc(String),
    /// An axis reference is unknown
    UnknownAxisRef(ir::axis::Ref),
    /// An axis reference is illegal in the given context
    IllegalAxisRef(ir::axis::Ref),
    /// An axis has no bounds (e.g. all data is NaN)
    UnboundedAxis,
    /// The IR model is inconsistent
    InconsistentIr(String),
    /// Axis bounds are inconsistent.
    /// For example, different data types are mixed on the same axis.
    InconsistentAxisBounds(String),
    /// Data is inconsistent.
    /// For example, columns have different lengths in a context it is not allowed.
    InconsistentData(String),
    /// Font or text related error, e.g. missing glyphs or font not found
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

impl From<ttf_parser::FaceParsingError> for Error {
    fn from(err: ttf_parser::FaceParsingError) -> Self {
        Error::FontOrText(text::Error::FaceParsingError(err))
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

/// Extension trait to prepare an IR figure for drawing
pub trait Drawing {
    /// Prepare a figure for drawing.
    /// The resulting [`Figure`] can then be drawn multiple times on different rendering surfaces.
    /// The texts are shaped, laid out and transformed to paths using the given font database.
    ///
    /// Theme and series colors are not used at this stage, they will be resolved at draw time.
    /// So the same prepared figure can be drawn with different themes and palettes.
    ///
    /// Panics: if `fontdb` is None and none of the bundled font features is enabled.
    fn prepare<D>(
        &self,
        data_source: &D,
        fontdb: Option<&fontdb::Database>,
    ) -> Result<Figure, Error>
    where
        D: data::Source;

    /// Convenience method to prepare and draw a figure in one step.
    ///
    /// Panics if no font database is given and no bundled font feature is enabled.
    fn draw<D, S, T, P>(
        &self,
        data_source: &D,
        fontdb: Option<&fontdb::Database>,
        surface: &mut S,
        style: &Style<T, P>,
    ) -> Result<(), Error>
    where
        D: data::Source,
        S: render::Surface,
        T: Theme,
        P: style::series::Palette,
    {
        self.prepare(data_source, fontdb)?.draw(surface, style)
    }
}

impl Drawing for ir::Figure {
    fn prepare<D>(
        &self,
        data_source: &D,
        fontdb: Option<&fontdb::Database>,
    ) -> Result<Figure, Error>
    where
        D: data::Source,
    {
        if let Some(fontdb) = fontdb {
            let ctx = Ctx::new(data_source, fontdb);
            ctx.setup_figure(self)
        } else {
            #[cfg(any(
                feature = "noto-sans",
                feature = "noto-sans-italic",
                feature = "noto-serif",
                feature = "noto-serif-italic",
                feature = "noto-mono"
            ))]
            {
                let fontdb = crate::bundled_font_db();
                let ctx = Ctx::new(data_source, &fontdb);
                ctx.setup_figure(self)
            }
            #[cfg(not(any(
                feature = "noto-sans",
                feature = "noto-sans-italic",
                feature = "noto-serif",
                feature = "noto-serif-italic",
                feature = "noto-mono"
            )))]
            {
                panic!(concat!(
                    "No font database provided and no bundled font feature enabled. ",
                    "Enable at least one of the bundled font features or provide a font database."
                ));
            }
        }
    }
}

#[derive(Debug)]
struct Ctx<'a, D> {
    data_source: &'a D,
    fontdb: &'a fontdb::Database,
}

impl<'a, D> Ctx<'a, D> {
    pub fn new(data_source: &'a D, fontdb: &'a fontdb::Database) -> Ctx<'a, D> {
        Ctx {
            data_source,
            fontdb,
        }
    }

    pub fn data_source(&self) -> &D {
        self.data_source
    }

    pub fn fontdb(&self) -> &fontdb::Database {
        &self.fontdb
    }
}

#[derive(Debug, Clone)]
struct Text {
    spans: Vec<TextSpan>,
    bbox: geom::Rect,
}

#[derive(Debug, Clone)]
struct TextSpan {
    path: geom::Path,
    fill: Option<theme::Fill>,
    stroke: Option<theme::Line>,
}

impl Text {
    fn from_line_text(
        text: &text::LineText,
        fontdb: &fontdb::Database,
        color: theme::Color,
    ) -> Result<Text, Error> {
        let mut spans = Vec::new();
        text::line::render_line_text_with(text, fontdb, |path| {
            spans.push(TextSpan {
                path: path.clone(),
                fill: Some(color.into()),
                stroke: None,
            });
        });
        Ok(Text {
            spans,
            bbox: text.bbox().cloned().unwrap_or_else(|| geom::Rect::null()),
        })
    }

    fn from_rich_text(
        text: &text::RichText<theme::Color>,
        fontdb: &fontdb::Database,
    ) -> Result<Text, Error> {
        let mut spans = Vec::new();
        text::rich::render_rich_text_with(text, fontdb, |prim| match prim {
            text::RichPrimitive::Fill(path, color) => {
                spans.push(TextSpan {
                    path: path.clone(),
                    fill: Some(color.into()),
                    stroke: None,
                });
            }
            text::RichPrimitive::Stroke(path, color, thickness) => {
                spans.push(TextSpan {
                    path: path.clone(),
                    fill: None,
                    stroke: Some(theme::Line {
                        color: color.into(),
                        width: thickness,
                        opacity: None,
                        pattern: Default::default(),
                    }),
                });
            }
        })?;
        Ok(Text {
            spans,
            bbox: text.bbox().cloned().unwrap_or_else(|| geom::Rect::null()),
        })
    }

    fn draw<S, T, P>(
        &self,
        surface: &mut S,
        style: &Style<T, P>,
        transform: Option<&geom::Transform>,
    ) -> Result<(), render::Error>
    where
        S: render::Surface,
        T: Theme,
    {
        for span in &self.spans {
            let rpath = render::Path {
                path: &span.path,
                fill: span.fill.as_ref().map(|f| f.as_paint(style)),
                stroke: span.stroke.as_ref().map(|s| s.as_stroke(style)),
                transform,
            };
            surface.draw_path(&rpath)?;
        }
        Ok(())
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
