use crate::drawing::{
    Categories, ColumnExt, Ctx, Error, F64ColumnExt, SurfWrapper, axis, legend, marker, scale,
};
use crate::render::{self, Surface as _};
use crate::{data, geom, ir, style};

use axis::AsBoundRef;
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

    fn font(&self) -> Option<&ir::legend::EntryFont> {
        None
    }

    fn shape(&self) -> legend::Shape {
        match &self.plot {
            ir::SeriesPlot::Line(xy) => legend::Shape::Line(xy.line.clone()),
            ir::SeriesPlot::Scatter(sc) => legend::Shape::Marker(sc.marker.clone()),
            ir::SeriesPlot::Histogram(hist) => legend::Shape::Rect(hist.fill, hist.line.clone()),
            ir::SeriesPlot::Bars(bars) => legend::Shape::Rect(bars.fill, bars.line.clone()),
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

fn calc_xy_bounds<D>(
    data_source: &D,
    x_data: &ir::series::DataCol,
    y_data: &ir::series::DataCol,
) -> Result<(axis::Bounds, axis::Bounds), Error>
where
    D: data::Source,
{
    let x_col = get_column(x_data, data_source)?;
    let y_col = get_column(y_data, data_source)?;

    if x_col.len() != y_col.len() {
        return Err(Error::InconsistentData(
            "X and Y data must be the same length".to_string(),
        ));
    }

    let x_bounds = x_col.bounds().ok_or(Error::UnboundedAxis)?;
    let y_bounds = y_col.bounds().ok_or(Error::UnboundedAxis)?;

    Ok((x_bounds, y_bounds))
}

pub struct Series {
    plot: SeriesPlot,
}

#[derive(Debug, Clone)]
enum SeriesPlot {
    Line(Line),
    Scatter(Scatter),
    Histogram(Histogram),
    Bars(Bars),
}

impl Series {
    pub fn from_ir<D>(index: usize, series: &ir::Series, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source,
    {
        let processed = match &series.plot {
            ir::SeriesPlot::Line(line) => {
                SeriesPlot::Line(Line::from_ir(index, line, data_source)?)
            }
            ir::SeriesPlot::Scatter(sc) => {
                SeriesPlot::Scatter(Scatter::from_ir(index, sc, data_source)?)
            }
            ir::SeriesPlot::Histogram(hist) => {
                SeriesPlot::Histogram(Histogram::from_ir(index, hist, data_source)?)
            }
            ir::SeriesPlot::Bars(bars) => {
                SeriesPlot::Bars(Bars::from_ir(index, bars, data_source)?)
            }
        };
        Ok(Series { plot: processed })
    }

    pub fn bounds(&self) -> (axis::BoundsRef<'_>, axis::BoundsRef<'_>) {
        match &self.plot {
            SeriesPlot::Line(line) => (line.ab.0.as_bound_ref(), line.ab.1.as_bound_ref()),
            SeriesPlot::Scatter(scatter) => (scatter.ab.0.as_bound_ref(), scatter.ab.1.as_bound_ref()),
            SeriesPlot::Histogram(hist) => (hist.ab.0.into(), hist.ab.1.into()),
            SeriesPlot::Bars(bars) => bars.bounds(),
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
                a = Some((b.0.to_bounds(), b.1.to_bounds()));
            }
        }
        Ok(a)
    }
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    pub fn draw_series_plot<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        ir_series: &ir::Series,
        series: &Series,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), Error>
    where
        D: data::Source,
        T: style::Theme,
    {
        match (&ir_series.plot, &series.plot) {
            (ir::SeriesPlot::Line(ir), SeriesPlot::Line(xy)) => {
                self.draw_series_line(ctx, ir, xy, rect, cm)
            }
            (ir::SeriesPlot::Scatter(ir), SeriesPlot::Scatter(sc)) => {
                self.draw_series_scatter(ctx, ir, sc, rect, cm)
            }
            (ir::SeriesPlot::Histogram(ir), SeriesPlot::Histogram(hist)) => {
                Ok(self.draw_series_histogram(ctx, ir, hist, rect, cm)?)
            }
            (ir::SeriesPlot::Bars(ir), SeriesPlot::Bars(bars)) => {
                Ok(self.draw_series_bars(ctx, ir, bars, rect, cm)?)
            }
            _ => unreachable!("Should be the same plot type"),
        }
    }
}

#[derive(Debug, Clone)]
struct Line {
    index: usize,
    ab: (axis::Bounds, axis::Bounds),
}

impl Line {
    fn from_ir<D>(index: usize, ir: &ir::series::Line, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source,
    {
        let (x_bounds, y_bounds) = calc_xy_bounds(data_source, &ir.x_data, &ir.y_data)?;
        Ok(Line {
            index,
            ab: (x_bounds, y_bounds),
        })
    }

    fn build_path<D>(
        &self,
        ir: &ir::series::Line,
        data_source: &D,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> geom::Path
    where
        D: data::Source,
    {
        // unwraping here as data is checked during setup phase
        let x_col = get_column(&ir.x_data, data_source).unwrap();
        let y_col = get_column(&ir.y_data, data_source).unwrap();

        debug_assert!(x_col.len() == y_col.len());

        let mut in_a_line = false;

        let mut pb = geom::PathBuilder::with_capacity(x_col.len() + 1, x_col.len());
        for (x, y) in x_col.iter().zip(y_col.iter()) {
            if x.is_null() || y.is_null() {
                in_a_line = false;
                continue;
            }
            let (x, y) = cm.map_coord((x, y)).expect("Should be valid coordinates");
            let x = rect.left() + x;
            let y = rect.bottom() - y;
            if in_a_line {
                pb.line_to(x, y);
            } else {
                pb.move_to(x, y);
                in_a_line = true;
            }
        }
        pb.finish().expect("Should be a valid path")
    }
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    fn draw_series_line<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        ir: &ir::series::Line,
        line: &Line,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), Error>
    where
        D: data::Source,
        T: style::Theme,
    {
        let path = line.build_path(ir, ctx.data_source(), rect, cm);
        let rc = (ctx.theme().palette(), line.index);

        let path = render::Path {
            path: &path,
            fill: None,
            stroke: Some(ir.line.as_stroke(&rc)),
            transform: None,
        };
        self.draw_path(&path)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Scatter {
    index: usize,
    ab: (axis::Bounds, axis::Bounds),
}

impl Scatter {
    fn from_ir<D>(index: usize, ir: &ir::series::Scatter, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source,
    {
        let (x_bounds, y_bounds) = calc_xy_bounds(data_source, &ir.x_data, &ir.y_data)?;
        Ok(Scatter {
            index,
            ab: (x_bounds, y_bounds),
        })
    }
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    fn draw_series_scatter<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        ir: &ir::series::Scatter,
        scatter: &Scatter,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), Error>
    where
        D: data::Source,
        T: style::Theme,
    {
        let path = marker::marker_path(&ir.marker);
        let rc = (ctx.theme().palette(), scatter.index);

        // unwraping here as data is checked during setup phase
        let x_col = get_column(&ir.x_data, ctx.data_source()).unwrap();
        let y_col = get_column(&ir.y_data, ctx.data_source()).unwrap();
        debug_assert!(x_col.len() == y_col.len());

        for (x, y) in x_col.iter().zip(y_col.iter()) {
            if x.is_null() || y.is_null() {
                continue;
            }
            let (x, y) = cm.map_coord((x, y)).expect("Should be valid coordinates");
            let x = rect.left() + x;
            let y = rect.bottom() - y;
            let transform = geom::Transform::from_translate(x, y);
            let path = render::Path {
                path: &path,
                fill: ir.marker.fill.as_ref().map(|f| f.as_paint(&rc)),
                stroke: ir.marker.stroke.as_ref().map(|l| l.as_stroke(&rc)),
                transform: Some(&transform),
            };
            self.draw_path(&path)?;
        }

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
    index: usize,
    ab: (axis::NumBounds, axis::NumBounds),
    bins: Vec<HistBin>,
}

impl Histogram {
    fn from_ir<D>(
        index: usize,
        hist: &ir::series::Histogram,
        data_source: &D,
    ) -> Result<Self, Error>
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
            index,
            ab: (x_bounds, y_bounds),
            bins,
        })
    }
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    fn draw_series_histogram<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        ir: &ir::series::Histogram,
        hist: &Histogram,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), render::Error>
    where
        T: style::Theme,
    {
        let rc = (ctx.theme().palette(), hist.index);

        let mut pb = geom::PathBuilder::new();
        let mut x = rect.left() + cm.x.map_coord_num(hist.bins[0].range.0);
        let mut y = rect.bottom() - cm.y.map_coord_num(0.0);
        pb.move_to(x, y);

        for bin in hist.bins.iter() {
            y = rect.bottom() - cm.y.map_coord_num(bin.value);
            pb.line_to(x, y);
            x = rect.left() + cm.x.map_coord_num(bin.range.1);
            pb.line_to(x, y);
        }

        y = rect.bottom() - cm.y.map_coord_num(0.0);
        pb.line_to(x, y);

        let path = pb.finish().expect("Should be a valid path");
        let path = render::Path {
            path: &path,
            fill: Some(ir.fill.as_paint(&rc)),
            stroke: ir.line.as_ref().map(|l| l.as_stroke(&rc)),
            transform: None,
        };
        self.draw_path(&path)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum BarsBounds {
    Vertical(Categories, axis::NumBounds),
    Horizontal(axis::NumBounds, Categories),
}

#[derive(Debug, Clone)]
struct Bars {
    index: usize,
    bounds: BarsBounds,
}

impl Bars {
    fn from_ir<D>(index: usize, ir: &ir::series::Bars, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source,
    {
        let (x_bounds, y_bounds) = calc_xy_bounds(data_source, &ir.x_data, &ir.y_data)?;

        let bounds = match (x_bounds, y_bounds) {
            (axis::Bounds::Num(x_bounds), axis::Bounds::Cat(y_bounds)) => {
                BarsBounds::Horizontal(x_bounds, y_bounds)
            }
            (axis::Bounds::Cat(x_bounds), axis::Bounds::Num(y_bounds)) => {
                BarsBounds::Vertical(x_bounds, y_bounds)
            }
            _ => {
                return Err(Error::InconsistentData(
                    "One of X and Y data must be numeric and the other categorical".to_string(),
                ));
            }
        };

        Ok(Bars { index, bounds })
    }

    fn bounds(&self) -> (axis::BoundsRef<'_>, axis::BoundsRef<'_>) {
        match &self.bounds {
            &BarsBounds::Vertical(ref x_bounds, y_bounds) => (x_bounds.into(), y_bounds.into()),
            &BarsBounds::Horizontal(x_bounds, ref y_bounds) => (x_bounds.into(), y_bounds.into()),
        }
    }
}


impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    fn draw_series_bars<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        ir: &ir::series::Bars,
        bars: &Bars,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), render::Error>
    where
        D: data::Source,
        T: style::Theme,
    {
        let rc = (ctx.theme().palette(), bars.index);

        // unwraping here as data is checked during setup phase
        let x_col = get_column(&ir.x_data, ctx.data_source()).unwrap();
        let y_col = get_column(&ir.y_data, ctx.data_source()).unwrap();
        debug_assert!(x_col.len() == y_col.len());

        let mut pb = geom::PathBuilder::new();

        match &bars.bounds {
            BarsBounds::Vertical(..) => {
                let cat_bin_width = cm.x.cat_bin_size();
                let y_start = rect.bottom() - cm.y.map_coord_num(0.0);

                for (x, y) in x_col.iter().zip(y_col.iter()) {
                    if x.is_null() || y.is_null() {
                        continue;   
                    }

                    let (x, y) = cm.map_coord((x, y)).expect("Should be valid coordinates");
                    let x_start = rect.left() + x + cat_bin_width * (ir.position.offset - 0.5);
                    let x_end = x_start + cat_bin_width * ir.position.width;
                    let y_end = rect.bottom() - y;
                    pb.move_to(x_start, y_start);
                    pb.line_to(x_start, y_end);
                    pb.line_to(x_end, y_end);
                    pb.line_to(x_end, y_start);
                }
            }
            BarsBounds::Horizontal(..) => {
                let cat_bin_height = cm.y.cat_bin_size();
                let x_start = rect.left() + cm.x.map_coord_num(0.0);

                for (x, y) in x_col.iter().zip(y_col.iter()) {
                    if x.is_null() || y.is_null() {
                        continue;   
                    }

                    let (x, y) = cm.map_coord((x, y)).expect("Should be valid coordinates");
                    let y_start = rect.bottom() - y - cat_bin_height * (ir.position.offset - 0.5);
                    let y_end = y_start - cat_bin_height * ir.position.width;
                    let x_end = rect.left() + x;
                    pb.move_to(x_start, y_start);
                    pb.line_to(x_end, y_start);
                    pb.line_to(x_end, y_end);
                    pb.line_to(x_start, y_end);
                }
            }
        }

        let path = pb.finish().expect("Should be a valid path");
        let path = render::Path {
            path: &path,
            fill: Some(ir.fill.as_paint(&rc)),
            stroke: ir.line.as_ref().map(|l| l.as_stroke(&rc)),
            transform: None,
        };
        self.draw_path(&path)?;
        Ok(())
    }
}