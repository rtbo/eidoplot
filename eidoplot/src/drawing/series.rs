use scale::CoordMapXy;

use crate::drawing::{Ctx, Error, F64ColumnExt, SurfWrapper, axis, legend, scale};
use crate::render::{self, Surface as _};
use crate::{data, geom, ir};

pub fn series_has_legend(series: &ir::Series) -> bool {
    series.name.is_some()
}

impl legend::Entry for ir::Series {
    fn label(&self) -> &str {
        self.name
            .as_deref()
            .expect("Should have a name, or not used as legend entry")
    }

    fn font(&self) -> Option<&ir::legend::EntryFont> {
        None
    }

    fn shape(&self) -> legend::Shape {
        match &self.plot {
            ir::SeriesPlot::Xy(xy) => legend::Shape::Line(xy.line),
            ir::SeriesPlot::Histogram(hist) => legend::Shape::Rect(hist.fill, hist.line),
        }
    }
}

fn get_column<'a, D>(
    col: &'a ir::series::DataCol,
    data_source: &'a D,
) -> Result<&'a dyn data::Column, Error>
where
    D: data::Source,
{
    match col {
        ir::series::DataCol::Inline(col) => Ok(col),
        ir::series::DataCol::SrcRef(name) => data_source
            .column(name)
            .ok_or_else(|| Error::MissingDataSrc(name.to_string())),
    }
}

pub struct Series {
    plot: SeriesPlot,
}

#[derive(Debug, Clone)]
enum SeriesPlot {
    Xy(Xy),
    Histogram(Histogram),
}

impl Series {
    pub fn from_ir<D>(series: &ir::Series, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source,
    {
        let processed = match &series.plot {
            ir::SeriesPlot::Xy(xy) => SeriesPlot::Xy(Xy::from_ir(xy, data_source)?),
            ir::SeriesPlot::Histogram(hist) => {
                SeriesPlot::Histogram(Histogram::from_ir(hist, data_source)?)
            }
        };
        Ok(Series { plot: processed })
    }

    pub fn bounds(&self) -> (axis::Bounds, axis::Bounds) {
        match &self.plot {
            SeriesPlot::Xy(xy) => (xy.ab.0.into(), xy.ab.1.into()),
            SeriesPlot::Histogram(hist) => (hist.ab.0.into(), hist.ab.1.into()),
        }
    }

    pub fn unite_bounds(series: &[Series]) -> Result<Option<(axis::Bounds, axis::Bounds)>, Error> {
        let mut a: Option<(axis::Bounds, axis::Bounds)> = None;
        for s in series {
            let b = s.bounds();
            if let Some(a) = &mut a {
                a.0.unite_with(&b.0)?;
                a.1.unite_with(&b.1)?;
            } else {
                a = Some(b);
            }
        }
        Ok(a)
    }
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    pub fn draw_series_plot<D>(
        &mut self,
        ctx: &Ctx<D>,
        ir_series: &ir::Series,
        series: &Series,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), Error>
    where
        D: data::Source,
    {
        match (&ir_series.plot, &series.plot) {
            (ir::SeriesPlot::Xy(ir), SeriesPlot::Xy(xy)) => {
                self.draw_series_xy(ctx, ir, xy, rect, cm)
            }
            (ir::SeriesPlot::Histogram(ir), SeriesPlot::Histogram(hist)) => {
                Ok(self.draw_series_histogram(ir, hist, rect, cm)?)
            }
            _ => unreachable!("Should be the same plot type"),
        }
    }
}

#[derive(Debug, Clone)]
struct Xy {
    ab: (axis::NumBounds, axis::NumBounds),
}

impl Xy {
    fn from_ir<D>(ir: &ir::series::Xy, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source,
    {
        let x_col = get_column(&ir.x_data, data_source)?
            .f64()
            .ok_or_else(|| Error::InconsistentData("XY data must be numeric".into()))?;

        let y_col = get_column(&ir.y_data, data_source)?
            .f64()
            .ok_or_else(|| Error::InconsistentData("XY data must be numeric".into()))?;

        if x_col.len() != y_col.len() {
            return Err(Error::InconsistentData(
                "X and Y data must be the same length".to_string(),
            ));
        }

        let x_bounds = x_col.bounds().ok_or(Error::UnboundedAxis)?;
        let y_bounds = y_col.bounds().ok_or(Error::UnboundedAxis)?;

        Ok(Xy {
            ab: (x_bounds, y_bounds),
        })
    }

    fn build_path<D>(
        &self,
        ir: &ir::series::Xy,
        data_source: &D,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> geom::Path
    where
        D: data::Source,
    {
        // unwraping here as data is checked during setup phase
        let x_col = get_column(&ir.x_data, data_source).unwrap().f64().unwrap();

        let y_col = get_column(&ir.y_data, data_source).unwrap().f64().unwrap();

        debug_assert!(x_col.len() == y_col.len());

        let mut in_a_line = false;

        let mut pb = geom::PathBuilder::with_capacity(x_col.len() + 1, x_col.len());
        for (x, y) in x_col.iter().zip(y_col.iter()) {
            match (x, y) {
                (Some(x), Some(y)) => {
                    let (x, y) = cm.map_coord((x, y));
                    let x = rect.left() + x;
                    let y = rect.bottom() - y;
                    if in_a_line {
                        pb.line_to(x, y);
                    } else {
                        pb.move_to(x, y);
                        in_a_line = true;
                    }
                }
                _ => in_a_line = false,
            }
        }
        pb.finish().expect("Should be a valid path")
    }
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    fn draw_series_xy<D>(
        &mut self,
        ctx: &Ctx<D>,
        ir: &ir::series::Xy,
        xy: &Xy,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), Error>
    where
        D: data::Source,
    {
        let path = xy.build_path(ir, ctx.data_source(), rect, cm);

        let path = render::Path {
            path: &path,
            fill: None,
            stroke: Some(ir.line.clone()),
            transform: None,
        };
        self.draw_path(&path)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct HistBin {
    /// Start and end of this bin
    range: (f64, f64),
    /// Either count or density
    value: f64,
}

#[derive(Debug, Clone)]
struct Histogram {
    ab: (axis::NumBounds, axis::NumBounds),
    bins: Vec<HistBin>,
}

impl Histogram {
    fn from_ir<D>(hist: &ir::series::Histogram, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source,
    {
        let mut bins = Vec::with_capacity(hist.bins as usize);

        let col = get_column(&hist.data, data_source)?;
        let col = col.f64().ok_or(Error::InconsistentData(
            "Histogram data must be numeric".into(),
        ))?;
        let x_bounds = col.bounds().ok_or(Error::UnboundedAxis)?;

        let width = x_bounds.span() / hist.bins as f64;
        let mut val = x_bounds.start();
        while val <= x_bounds.end() {
            bins.push(HistBin {
                range: (val, val + width),
                value: 0.0,
            });
            val += width;
        }

        let samp_add = if hist.density {
            1.0 / (col.len_some() as f64 * width)
        } else {
            1.0
        };

        for x in col.iter() {
            if let Some(x) = x {
                let idx = (((x - x_bounds.start()) / width).floor() as usize).min(bins.len() - 1);
                bins[idx].value += samp_add;
            }
        }

        let mut y_bounds = axis::NumBounds::NAN;
        for bin in bins.iter() {
            y_bounds.add_sample(bin.value);
        }

        Ok(Histogram {
            ab: (x_bounds, y_bounds),
            bins,
        })
    }
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    fn draw_series_histogram(
        &mut self,
        ir: &ir::series::Histogram,
        hist: &Histogram,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), render::Error> {
        let mut pb = geom::PathBuilder::new();
        let mut x = rect.left() + cm.x.map_coord(hist.bins[0].range.0);
        let mut y = rect.bottom() - cm.y.map_coord(0.0);
        pb.move_to(x, y);

        for bin in hist.bins.iter() {
            y = rect.bottom() - cm.y.map_coord(bin.value);
            pb.line_to(x, y);
            x = rect.left() + cm.x.map_coord(bin.range.1);
            pb.line_to(x, y);
        }

        y = rect.bottom() - cm.y.map_coord(0.0);
        pb.line_to(x, y);

        let path = pb.finish().expect("Should be a valid path");
        let path = render::Path {
            path: &path,
            fill: Some(ir.fill.clone()),
            stroke: ir.line.clone(),
            transform: None,
        };
        self.draw_path(&path)?;
        Ok(())
    }
}
