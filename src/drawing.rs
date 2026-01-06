//! Drawing module
//!
//! This module contains all the logic to convert a design figure into rendering commands
//! for a given rendering surface.
//! It is the bridge between the [`des`] module and the [`render`] module.
use std::fmt;

use text::fontdb;

use crate::style::theme;
use crate::{Style, data, des, geom, render, text};

mod annot;
mod axis;
mod figure;
mod hit_test;
mod legend;
mod marker;
mod plot;
mod scale;
mod series;
mod ticks;
pub mod zoom;

pub use figure::Figure;
pub use hit_test::PlotHit;

/// Errors that can occur during figure drawing
#[derive(Debug)]
pub enum Error {
    /// A series references a missing data source
    MissingDataSrc(String),
    /// An axis reference is unknown
    UnknownAxisRef(des::axis::Ref),
    /// An axis reference is illegal in the given context
    IllegalAxisRef(des::axis::Ref),
    /// An axis has no bounds (e.g. all data is NaN)
    UnboundedAxis,
    /// The design model is inconsistent
    InconsistentDesign(String),
    /// Axis bounds are inconsistent.
    /// For example, different data types are mixed on the same axis.
    InconsistentAxisBounds(String),
    /// Data is inconsistent.
    /// For example, columns have different lengths in a context it is not allowed.
    InconsistentData(String),
    /// Font or text related error, e.g. missing glyphs or font not found
    FontOrText(text::Error),
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
            Error::MissingDataSrc(name) => write!(f, "Missing data source: {}", name),
            Error::UnknownAxisRef(axis_ref) => write!(f, "Unknown axis reference: {:?}", axis_ref),
            Error::IllegalAxisRef(axis_ref) => write!(f, "Illegal axis reference: {:?}", axis_ref),
            Error::UnboundedAxis => write!(f, "Unbounded axis, check data"),
            Error::InconsistentDesign(reason) => write!(f, "Inconsistent Design: {}", reason),
            Error::InconsistentAxisBounds(reason) => {
                write!(f, "Inconsistent axis bounds: {}", reason)
            }
            Error::InconsistentData(reason) => write!(f, "Inconsistent data: {}", reason),
            Error::FontOrText(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

#[inline]
fn fig_x_to_plot_x(plot_rect: &geom::Rect, fig_x: f32) -> f32 {
    fig_x - plot_rect.x()
}

#[inline]
fn fig_y_to_plot_y(plot_rect: &geom::Rect, fig_y: f32) -> f32 {
    plot_rect.bottom() - fig_y
}

/// Extension trait to prepare a design figure for drawing
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
        D: data::Source + ?Sized;

    /// Convenience method to prepare and draw a figure in one step.
    ///
    /// Panics if no font database is given and no bundled font feature is enabled.
    fn draw<D, S>(
        &self,
        data_source: &D,
        fontdb: Option<&fontdb::Database>,
        surface: &mut S,
        style: &Style,
    ) -> Result<(), Error>
    where
        D: data::Source + ?Sized,
        S: render::Surface,
    {
        self.prepare(data_source, fontdb)?.draw(surface, style);
        Ok(())
    }
}

impl Drawing for des::Figure {
    fn prepare<D>(
        &self,
        data_source: &D,
        fontdb: Option<&fontdb::Database>,
    ) -> Result<Figure, Error>
    where
        D: data::Source + ?Sized,
    {
        with_ctx(data_source, fontdb, |ctx| ctx.setup_figure(self))
    }
}

#[derive(Debug)]
struct Ctx<'a, D: ?Sized> {
    data_source: &'a D,
    fontdb: &'a fontdb::Database,
}

fn with_ctx<D, F, R>(data_source: &D, fontdb: Option<&fontdb::Database>, f: F) -> R
where
    D: data::Source + ?Sized,
    F: FnOnce(&Ctx<'_, D>) -> R,
{
    if let Some(fontdb) = fontdb {
        let ctx = Ctx {
            data_source,
            fontdb,
        };
        f(&ctx)
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
            let ctx = Ctx {
                data_source,
                fontdb: &fontdb,
            };
            f(&ctx)
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

impl<'a, D: ?Sized> Ctx<'a, D> {
    fn data_source(&self) -> &D {
        self.data_source
    }

    fn fontdb(&self) -> &fontdb::Database {
        &self.fontdb
    }
}

#[derive(Debug, Clone)]
struct Text {
    text: String,
    spans: Vec<TextSpan>,
    bbox: Option<geom::Rect>,
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
            text: text.text().to_string(),
            spans,
            bbox: text.bbox().cloned(),
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
            text: text.text().to_string(),
            spans,
            bbox: text.bbox().cloned(),
        })
    }

    fn width(&self) -> f32 {
        self.bbox.map_or(0.0, |r| r.width())
    }

    fn height(&self) -> f32 {
        self.bbox.map_or(0.0, |r| r.height())
    }

    fn _visual_bbox(&self) -> Option<geom::Rect> {
        let mut bbox: Option<geom::Rect> = None;
        for s in self.spans.iter() {
            let r = s.path.bounds();
            let r = geom::Rect::from_trbl(r.top(), r.right(), r.bottom(), r.left());
            bbox = geom::Rect::unite_opt(Some(&r), bbox.as_ref());
        }
        bbox
    }

    fn draw<S>(
        &self,
        surface: &mut S,
        style: &Style,
        transform: Option<&geom::Transform>,
    ) where
        S: render::Surface,
    {
        for span in &self.spans {
            let rpath = render::Path {
                path: &span.path,
                fill: span.fill.as_ref().map(|f| f.as_paint(style)),
                stroke: span.stroke.as_ref().map(|s| s.as_stroke(style)),
                transform,
            };
            surface.draw_path(&rpath);
        }
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
        #[cfg(feature = "time")]
        if let Some(time) = self.time() {
            return time
                .minmax()
                .map(|(min, max)| axis::Bounds::Time((min, max).into()));
        }

        if let Some(num) = self.f64() {
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

    fn get(&self, idx: usize) -> Option<&str> {
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
        for s in col.str_iter() {
            if let Some(s) = s {
                cats.push_if_not_present(s);
            }
        }
        cats
    }
}
