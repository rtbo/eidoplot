use crate::data;
use crate::drawing::{Ctx, legend, scale};
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
            ir::plot::SeriesPlot::Xy(xy) => legend::Shape::Line(xy.line),
            ir::plot::SeriesPlot::Histogram(hist) => legend::Shape::Rect(hist.fill, hist.line),
        }
    }
}

impl<'a, S> Ctx<'a, S>
where
    S: render::Surface,
{
    pub fn draw_series_plot(
        &mut self,
        series: &Series,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), S::Error> {
        match (&series.ir.plot, &series.plot) {
            (ir::plot::SeriesPlot::Xy(ir), SeriesPlot::Xy(xy)) => self.draw_series_xy(ir, xy, rect, cm),
            (ir::plot::SeriesPlot::Histogram(ir), SeriesPlot::Histogram(hist)) => {
                self.draw_series_histogram(ir, hist, rect, cm)
            }
            _ => unreachable!("Should be the same plot type"),
        }
    }
    fn draw_series_xy(
        &mut self,
        ir: &ir::plot::XySeries,
        _xy: &Xy,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), S::Error> {
        let mut pb = geom::PathBuilder::with_capacity(ir.points.len() + 1, ir.points.len());
        for (i, dp) in ir.points.iter().enumerate() {
            let (x, y) = cm.map_coord(*dp);
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
        ir: &ir::plot::HistogramSeries,
        hist: &Histogram,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), S::Error> {
        let mut pb = geom::PathBuilder::new();
        let mut x = rect.left() + cm.x.map_coord(hist.bins[0].range.0);
        let mut y = rect.bottom();
        pb.move_to(x, y);

        for bin in hist.bins.iter() {
            y = rect.bottom() - cm.y.map_coord(bin.value);
            pb.line_to(x, y);
            x = rect.left() + cm.x.map_coord(bin.range.1);
            pb.line_to(x, y);
        }

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
    pub fn from_ir(series: &'a ir::Series) -> Self {
        let processed = match &series.plot {
            ir::plot::SeriesPlot::Xy(xy) => SeriesPlot::Xy(Xy::from_ir(xy)),
            ir::plot::SeriesPlot::Histogram(hist) => {
                SeriesPlot::Histogram(Histogram::from_ir(hist))
            }
        };
        Series {
            ir: series,
            plot: processed,
        }
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
    fn from_ir(xy: &ir::plot::XySeries) -> Self {
        let mut x_bounds = data::ViewBounds::NAN;
        let mut y_bounds = data::ViewBounds::NAN;
        for (x, y) in &xy.points {
            x_bounds.add_point(*x);
            y_bounds.add_point(*y);
        }
        Xy {
            vb: (x_bounds, y_bounds),
        }
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
    fn from_ir(hist: &ir::plot::HistogramSeries) -> Self {
        let mut bins = Vec::with_capacity(hist.bins as usize);

        let mut x_bounds = data::ViewBounds::NAN;
        for x in hist.points.iter() {
            x_bounds.add_point(*x);
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
            1.0 / (hist.points.len() as f64 * width)
        } else {
            1.0
        };

        for x in hist.points.iter() {
            let idx = ((x - x_bounds.min()) / width).floor() as usize;
            bins[idx].value += samp_add;
        }

        let mut y_bounds = data::ViewBounds::NAN;
        for bin in bins.iter() {
            y_bounds.add_point(bin.value);
        }

        Histogram {
            vb: (x_bounds, y_bounds),
            bins,
        }
    }
}
