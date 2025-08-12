use crate::data::{self, SourceIterator};
use crate::drawing::{Ctx, Error, legend, scale};
use crate::geom;
use crate::ir;
use crate::render::{self, Surface};

use scale::CoordMapXy;

pub fn series_has_legend(series: &ir::Series) -> bool {
    series.name.is_some()
}

impl legend::Entry for ir::Series {
    fn label(&self) -> &str {
        self.name
            .as_deref()
            .expect("Should have a name, or not used as legend entry")
    }

    fn font(&self) -> Option<&crate::style::Font> {
        None
    }

    fn shape(&self) -> legend::Shape {
        match &self.plot {
            ir::SeriesPlot::Xy(xy) => legend::Shape::Line(xy.line),
            ir::SeriesPlot::Histogram(hist) => legend::Shape::Rect(hist.fill, hist.line),
        }
    }
}

impl<'a, S, D> Ctx<'a, S, D>
where
    S: render::Surface,
    D: data::Source,
{
    pub fn draw_series_plot(
        &mut self,
        series: &Series,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), Error> {
        match (&series.ir.plot, &series.plot) {
            (ir::SeriesPlot::Xy(ir), SeriesPlot::Xy(xy)) => self.draw_series_xy(ir, xy, rect, cm),
            (ir::SeriesPlot::Histogram(ir), SeriesPlot::Histogram(hist)) => {
                Ok(self.draw_series_histogram(ir, hist, rect, cm)?)
            }
            _ => unreachable!("Should be the same plot type"),
        }
    }
    fn draw_series_xy(
        &mut self,
        ir: &ir::series::Xy,
        _xy: &Xy,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), Error> {
        //let mut pb = geom::PathBuilder::with_capacity(ir.points.len() + 1, ir.points.len());
        let mut pb = geom::PathBuilder::new();
        for (i, dp) in ir.data.iter_src(self.data_source())?.enumerate() {
            let (x, y) = cm.map_coord(dp);
            let x = rect.left() + x;
            let y = rect.bottom() - y;
            if i == 0 {
                pb.move_to(x, y);
            } else {
                pb.line_to(x, y);
            }
        }
        let path = pb.finish().expect("Should be a valid path");
        let path = render::Path {
            path: &path,
            fill: None,
            stroke: Some(ir.line.clone()),
            transform: None,
        };
        self.draw_path(&path)?;
        Ok(())
    }

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

pub struct Series<'a> {
    ir: &'a ir::Series,
    plot: SeriesPlot,
}

impl<'a> Series<'a> {
    pub fn from_ir<D>(series: &'a ir::Series, data_source: &D) -> Result<Self, data::Error>
    where
        D: data::Source,
    {
        let processed = match &series.plot {
            ir::SeriesPlot::Xy(xy) => SeriesPlot::Xy(Xy::from_ir(xy, data_source)?),
            ir::SeriesPlot::Histogram(hist) => {
                SeriesPlot::Histogram(Histogram::from_ir(hist, data_source)?)
            }
        };
        Ok(Series {
            ir: series,
            plot: processed,
        })
    }

    pub fn view_bounds(&self) -> (data::ViewBounds, data::ViewBounds) {
        match &self.plot {
            SeriesPlot::Xy(xy) => xy.vb,
            SeriesPlot::Histogram(hist) => hist.vb,
        }
    }

    pub fn unite_view_bounds(series: &[Series]) -> (data::ViewBounds, data::ViewBounds) {
        let mut x_bounds = data::ViewBounds::NAN;
        let mut y_bounds = data::ViewBounds::NAN;
        for s in series {
            let (x, y) = s.view_bounds();
            x_bounds.add_bounds(x);
            y_bounds.add_bounds(y);
        }
        (x_bounds, y_bounds)
    }
}

enum SeriesPlot {
    Xy(Xy),
    Histogram(Histogram),
}

struct Xy {
    vb: (data::ViewBounds, data::ViewBounds),
}

impl Xy {
    fn from_ir<D>(xy: &ir::series::Xy, data_source: &D) -> Result<Self, data::Error>
    where
        D: data::Source,
    {
        let mut x_bounds = data::ViewBounds::NAN;
        let mut y_bounds = data::ViewBounds::NAN;
        for (x, y) in xy.data.iter_src(data_source)? {
            x_bounds.add_point(x);
            y_bounds.add_point(y);
        }
        Ok(Xy {
            vb: (x_bounds, y_bounds),
        })
    }
}

struct HistBin {
    /// Start and end of this bin
    range: (f64, f64),
    /// Either count or density
    value: f64,
}

struct Histogram {
    vb: (data::ViewBounds, data::ViewBounds),
    bins: Vec<HistBin>,
}

impl Histogram {
    fn from_ir<D>(hist: &ir::series::Histogram, data_source: &D) -> Result<Self, data::Error>
    where
        D: data::Source,
    {
        let mut bins = Vec::with_capacity(hist.bins as usize);

        let mut x_bounds = data::ViewBounds::NAN;
        let mut len = 0;
        for x in hist.data.iter_src(data_source)? {
            x_bounds.add_point(x);
            len += 1;
        }
        let width = x_bounds.span() / hist.bins as f64;
        let mut val = x_bounds.min();
        while val <= x_bounds.max() {
            bins.push(HistBin {
                range: (val, val + width),
                value: 0.0,
            });
            val += width;
        }

        let samp_add = if hist.density {
            1.0 / (len as f64 * width)
        } else {
            1.0
        };

        for x in hist.data.iter_src(data_source)? {
            let idx = (((x - x_bounds.min()) / width).floor() as usize).min(bins.len() - 1);
            bins[idx].value += samp_add;
        }

        let mut y_bounds = data::ViewBounds::NAN;
        for bin in bins.iter() {
            y_bounds.add_point(bin.value);
        }

        Ok(Histogram {
            vb: (x_bounds, y_bounds),
            bins,
        })
    }
}
