use axis::AsBoundRef;
use scale::{CoordMap, CoordMapXy};

use crate::drawing::{
    Categories, ColumnExt, Ctx, Error, F64ColumnExt, SurfWrapper, axis as axis, legend, marker, scale,
};
use crate::render::{self, Surface as _};
use crate::{data, geom, ir, style};

/// trait implemented by series, or any other item that
/// has to populate the legend
pub trait SeriesExt {
    fn legend_entry(&self) -> Option<legend::Entry<'_>>;
}

impl SeriesExt for ir::series::Line {
    fn legend_entry(&self) -> Option<legend::Entry<'_>> {
        self.name().map(|n| legend::Entry {
            label: n.as_ref(),
            font: None,
            shape: legend::ShapeRef::Line(self.line()),
        })
    }
}

impl SeriesExt for ir::series::Scatter {
    fn legend_entry(&self) -> Option<legend::Entry<'_>> {
        self.name().map(|n| legend::Entry {
            label: n.as_ref(),
            font: None,
            shape: legend::ShapeRef::Marker(self.marker()),
        })
    }
}

impl SeriesExt for ir::series::Histogram {
    fn legend_entry(&self) -> Option<legend::Entry<'_>> {
        self.name().map(|n| legend::Entry {
            label: n.as_ref(),
            font: None,
            shape: legend::ShapeRef::Rect(&self.fill(), self.line()),
        })
    }
}

impl SeriesExt for ir::series::Bars {
    fn legend_entry(&self) -> Option<legend::Entry<'_>> {
        self.name().map(|n| legend::Entry {
            label: n.as_ref(),
            font: None,
            shape: legend::ShapeRef::Rect(self.fill(), self.line()),
        })
    }
}

impl SeriesExt for ir::series::BarSeries {
    fn legend_entry(&self) -> Option<legend::Entry<'_>> {
        self.name().map(|n| legend::Entry {
            label: n.as_ref(),
            font: None,
            shape: legend::ShapeRef::Rect(&self.fill(), self.line()),
        })
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

#[derive(Debug, Clone)]
pub struct Series(SeriesPlot);

#[derive(Debug, Clone)]
enum SeriesPlot {
    Line(Line),
    Scatter(Scatter),
    Histogram(Histogram),
    Bars(Bars),
    BarsGroup(BarsGroup),
}

impl Series {
    pub fn from_ir<D>(index: usize, series: &ir::Series, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source,
    {
        let series = match &series {
            ir::Series::Line(ir) => SeriesPlot::Line(Line::from_ir(index, ir, data_source)?),
            ir::Series::Scatter(ir) => {
                SeriesPlot::Scatter(Scatter::from_ir(index, ir, data_source)?)
            }
            ir::Series::Histogram(ir) => {
                SeriesPlot::Histogram(Histogram::from_ir(index, ir, data_source)?)
            }
            ir::Series::Bars(ir) => SeriesPlot::Bars(Bars::from_ir(index, ir, data_source)?),
            ir::Series::BarsGroup(ir) => {
                SeriesPlot::BarsGroup(BarsGroup::from_ir(index, ir, data_source)?)
            }
        };
        Ok(Series(series))
    }

    pub fn unite_x_bounds<'a, S>(series: S, starter: Option<axis::Bounds>) -> Result<Option<axis::Bounds>, Error> 
    where 
        S: IntoIterator<Item = &'a Series>,
    {
        let mut a: Option<axis::Bounds> = starter;
        for s in series {
            let b = s.bounds();
            if let Some(a) = &mut a {
                a.unite_with(&b.0)?;
            } else {
                a = Some(b.0.to_bounds());
            }
        }
        Ok(a)
    }

    pub fn unite_y_bounds<'a, S>(series: S, starter: Option<axis::Bounds>) -> Result<Option<axis::Bounds>, Error> 
    where 
        S: IntoIterator<Item = &'a Series>,
    {
        let mut a  = starter;
        for s in series {
            let b = s.bounds();
            if let Some(a) = &mut a {
                a.unite_with(&b.1)?;
            } else {
                a = Some(b.1.to_bounds());
            }
        }
        Ok(a)
    }

    fn bounds(&self) -> (axis::BoundsRef<'_>, axis::BoundsRef<'_>) {
        match &self.0 {
            SeriesPlot::Line(line) => (line.ab.0.as_bound_ref(), line.ab.1.as_bound_ref()),
            SeriesPlot::Scatter(scatter) => {
                (scatter.ab.0.as_bound_ref(), scatter.ab.1.as_bound_ref())
            }
            SeriesPlot::Histogram(hist) => (hist.ab.0.into(), hist.ab.1.into()),
            SeriesPlot::Bars(bars) => bars.bounds(),
            SeriesPlot::BarsGroup(bg) => (bg.bounds.0.as_bound_ref(), bg.bounds.1.as_bound_ref()),
        }
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
        match (&ir_series, &series.0) {
            (ir::Series::Line(ir), SeriesPlot::Line(xy)) => {
                self.draw_series_line(ctx, ir, xy, rect, cm)
            }
            (ir::Series::Scatter(ir), SeriesPlot::Scatter(sc)) => {
                self.draw_series_scatter(ctx, ir, sc, rect, cm)
            }
            (ir::Series::Histogram(ir), SeriesPlot::Histogram(hist)) => {
                Ok(self.draw_series_histogram(ctx, ir, hist, rect, cm)?)
            }
            (ir::Series::Bars(ir), SeriesPlot::Bars(bars)) => {
                Ok(self.draw_series_bars(ctx, ir, bars, rect, cm)?)
            }
            (ir::Series::BarsGroup(ir), SeriesPlot::BarsGroup(bg)) => {
                Ok(self.draw_series_bars_group(ctx, ir, bg, rect, cm)?)
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
        let (x_bounds, y_bounds) = calc_xy_bounds(data_source, ir.x_data(), ir.y_data())?;
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
        let x_col = get_column(ir.x_data(), data_source).unwrap();
        let y_col = get_column(ir.y_data(), data_source).unwrap();

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
            stroke: Some(ir.line().as_stroke(&rc)),
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
        let (x_bounds, y_bounds) = calc_xy_bounds(data_source, ir.x_data(), ir.y_data())?;
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
        let path = marker::marker_path(ir.marker());
        let rc = (ctx.theme().palette(), scatter.index);

        // unwraping here as data is checked during setup phase
        let x_col = get_column(ir.x_data(), ctx.data_source()).unwrap();
        let y_col = get_column(ir.y_data(), ctx.data_source()).unwrap();
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
                fill: ir.marker().fill.as_ref().map(|f| f.as_paint(&rc)),
                stroke: ir.marker().stroke.as_ref().map(|l| l.as_stroke(&rc)),
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
        let mut bins = Vec::with_capacity(hist.bins() as usize);

        let col = get_column(hist.data(), data_source)?;
        let col = col.f64().ok_or(Error::InconsistentData(
            "Histogram data must be numeric".into(),
        ))?;
        let x_bounds = col.bounds().ok_or(Error::UnboundedAxis)?;

        let width = x_bounds.span() / hist.bins() as f64;
        let mut val = x_bounds.start();
        while val <= x_bounds.end() {
            bins.push(HistBin {
                range: (val, val + width),
                value: 0.0,
            });
            val += width;
        }

        let samp_add = if hist.density() {
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
            fill: Some(ir.fill().as_paint(&rc)),
            stroke: ir.line().as_ref().map(|l| l.as_stroke(&rc)),
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
        let (x_bounds, y_bounds) = calc_xy_bounds(data_source, ir.x_data(), ir.y_data())?;

        let bounds = match (x_bounds, y_bounds) {
            (axis::Bounds::Num(mut x_bounds), axis::Bounds::Cat(y_bounds)) => {
                x_bounds.add_sample(0.0);
                BarsBounds::Horizontal(x_bounds, y_bounds)
            }
            (axis::Bounds::Cat(x_bounds), axis::Bounds::Num(mut y_bounds)) => {
                y_bounds.add_sample(0.0);
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
        let x_col = get_column(ir.x_data(), ctx.data_source()).unwrap();
        let y_col = get_column(ir.y_data(), ctx.data_source()).unwrap();
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
                    let x_start = rect.left() + x + cat_bin_width * (ir.position().offset - 0.5);
                    let x_end = x_start + cat_bin_width * ir.position().width;
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
                    let y_start = rect.bottom() - y - cat_bin_height * (ir.position().offset - 0.5);
                    let y_end = y_start - cat_bin_height * ir.position().width;
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
            fill: Some(ir.fill().as_paint(&rc)),
            stroke: ir.line().map(|l| l.as_stroke(&rc)),
            transform: None,
        };
        self.draw_path(&path)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BarsGroup {
    fst_index: usize,
    bounds: (axis::Bounds, axis::Bounds),
}

impl BarsGroup {
    fn from_ir<D>(index: usize, ir: &ir::series::BarsGroup, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source,
    {
        let cat_col = get_column(ir.categories(), data_source)?;
        let categories: Categories = cat_col
            .str()
            .ok_or_else(|| {
                Error::InconsistentData("BarsGroup categories must be a string column".to_string())
            })?
            .into();

        let mut bounds_per_cat: Vec<axis::NumBounds> =
            vec![axis::NumBounds::from(0.0); categories.len()];

        for bs in ir.series() {
            let data_col = get_column(bs.data(), data_source)?;
            if data_col.len() != categories.len() {
                return Err(Error::InconsistentData(
                    "BarsGroup data must be the same length as categories".to_string(),
                ));
            }
            let data_col = data_col.f64().ok_or(Error::InconsistentData(
                "BarsGroup data must be numeric".to_string(),
            ))?;

            for (v, bounds) in data_col.iter().zip(bounds_per_cat.iter_mut()) {
                if let Some(v) = v {
                    match ir.arrangement() {
                        ir::series::BarsArrangement::Aside(..) => {
                            bounds.add_sample(v);
                        }
                        ir::series::BarsArrangement::Stack(..) => {
                            if bounds.end().is_finite() {
                                bounds.add_sample(v + bounds.end());
                            } else {
                                bounds.add_sample(v);
                            }
                        }
                    }
                }
            }
        }

        let mut num_bounds = axis::NumBounds::NAN;
        for bounds in &bounds_per_cat {
            num_bounds.unite_with(bounds);
        }

        let bounds = match ir.orientation() {
            ir::series::BarsOrientation::Vertical => {
                (axis::Bounds::Cat(categories), axis::Bounds::Num(num_bounds))
            }
            ir::series::BarsOrientation::Horizontal => {
                (axis::Bounds::Num(num_bounds), axis::Bounds::Cat(categories))
            }
        };

        Ok(BarsGroup {
            fst_index: index,
            bounds,
        })
    }
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    fn draw_series_bars_group<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        ir: &ir::series::BarsGroup,
        bg: &BarsGroup,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), render::Error>
    where
        D: data::Source,
        T: style::Theme,
    {
        let categories = match ir.orientation() {
            ir::series::BarsOrientation::Vertical => bg.bounds.0.as_cat().unwrap(),
            ir::series::BarsOrientation::Horizontal => bg.bounds.1.as_cat().unwrap(),
        };

        match ir.arrangement() {
            ir::series::BarsArrangement::Aside(aside) => {
                self.draw_series_bars_aside(ctx, ir, &aside, bg, categories, rect, cm)
            }
            ir::series::BarsArrangement::Stack(stack) => {
                self.draw_series_bars_stack(ctx, ir, &stack, bg, categories, rect, cm)
            }
        }
    }

    fn draw_series_bars_aside<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        ir: &ir::series::BarsGroup,
        arrangement: &ir::series::BarsAsideArrangement,
        bg: &BarsGroup,
        categories: &Categories,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), render::Error>
    where
        D: data::Source,
        T: style::Theme,
    {
        let num_series = ir.series().len();
        if num_series == 0 {
            return Ok(());
        }
        let num_gaps = num_series - 1;

        let ir::series::BarsAsideArrangement {
            mut offset,
            width,
            gap,
        } = *arrangement;
        let width = (width - gap * num_gaps as f32) / num_series as f32;

        let mut col_idx = bg.fst_index;

        for series in ir.series() {
            let data_col = get_column(series.data(), ctx.data_source()).unwrap();
            let data_col = data_col.f64().unwrap();

            let mut pb = geom::PathBuilder::new();

            for (cat, val) in categories.iter().zip(data_col.iter()) {
                let Some(val) = val else { continue };

                let val_start = 0.0;
                let val_end = val_start + val;

                let cat_coords = ir.orientation().cat_coords(cm, cat, offset, width, rect);
                let val_coords = ir.orientation().val_coords(cm, val_start, val_end, rect);
                ir.orientation()
                    .add_series_path(&mut pb, cat_coords, val_coords);
            }

            let path = pb.finish().expect("Failed to build path");

            let rc = (ctx.theme().palette(), col_idx);
            col_idx += 1;

            let rpath = render::Path {
                path: &path,
                fill: Some(series.fill().as_paint(&rc)),
                stroke: series.line().map(|l| l.as_stroke(&rc)),
                transform: None,
            };
            self.draw_path(&rpath)?;

            offset += width + gap;
        }
        Ok(())
    }

    fn draw_series_bars_stack<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        ir: &ir::series::BarsGroup,
        arrangement: &ir::series::BarsStackArrangement,
        bg: &BarsGroup,
        categories: &Categories,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), render::Error>
    where
        D: data::Source,
        T: style::Theme,
    {
        let mut cat_values = vec![0.0; categories.len()];

        let mut col_idx = bg.fst_index;

        for series in ir.series() {
            let data_col = get_column(series.data(), ctx.data_source()).unwrap();
            let data_col = data_col.f64().unwrap();

            let mut pb = geom::PathBuilder::new();

            for (idx, (cat, val)) in categories.iter().zip(data_col.iter()).enumerate() {
                let Some(val) = val else { continue };

                let val_start = cat_values[idx];
                let val_end = val_start + val;

                cat_values[idx] = val_end;

                let cat_coords = ir.orientation().cat_coords(
                    cm,
                    cat,
                    arrangement.offset,
                    arrangement.width,
                    rect,
                );
                let val_coords = ir.orientation().val_coords(cm, val_start, val_end, rect);
                ir.orientation()
                    .add_series_path(&mut pb, cat_coords, val_coords);
            }

            let path = pb.finish().expect("Failed to build path");

            let rc = (ctx.theme().palette(), col_idx);
            col_idx += 1;

            let rpath = render::Path {
                path: &path,
                fill: Some(series.fill().as_paint(&rc)),
                stroke: series.line().map(|l| l.as_stroke(&rc)),
                transform: None,
            };
            self.draw_path(&rpath)?;
        }
        Ok(())
    }
}

trait BarsOrientationExt {
    fn cat_map<'a>(&self, cm: &'a CoordMapXy) -> &'a dyn CoordMap;
    fn val_map<'a>(&self, cm: &'a CoordMapXy) -> &'a dyn CoordMap;

    fn cat_coords(
        &self,
        cm: &CoordMapXy,
        cat: &str,
        bar_offset: f32,
        bar_size: f32,
        rect: &geom::Rect,
    ) -> (f32, f32);

    fn val_coords(
        &self,
        cm: &CoordMapXy,
        val_start: f64,
        val_end: f64,
        rect: &geom::Rect,
    ) -> (f32, f32);

    fn add_series_path(
        &self,
        pb: &mut geom::PathBuilder,
        cat_coords: (f32, f32),
        val_coords: (f32, f32),
    );
}

impl BarsOrientationExt for ir::series::BarsOrientation {
    fn cat_map<'a>(&self, cm: &'a CoordMapXy) -> &'a dyn CoordMap {
        match self {
            Self::Vertical => cm.x,
            Self::Horizontal => cm.y,
        }
    }

    fn val_map<'a>(&self, cm: &'a CoordMapXy) -> &'a dyn CoordMap {
        match self {
            Self::Vertical => cm.y,
            Self::Horizontal => cm.x,
        }
    }

    fn cat_coords(
        &self,
        cm: &CoordMapXy,
        cat: &str,
        bar_offset: f32,
        bar_size: f32,
        rect: &geom::Rect,
    ) -> (f32, f32) {
        let cat_map = self.cat_map(cm);
        let bin_size = cat_map.cat_bin_size();
        let coord = cat_map.map_coord_cat(cat);
        let start = match self {
            Self::Vertical => rect.left() + coord + bin_size * (bar_offset - 0.5),
            Self::Horizontal => rect.bottom() - coord - bin_size * (bar_offset - 0.5),
        };
        let end = match self {
            Self::Vertical => start + bin_size * bar_size,
            Self::Horizontal => start - bin_size * bar_size,
        };
        (start, end)
    }

    fn val_coords(
        &self,
        cm: &CoordMapXy,
        val_start: f64,
        val_end: f64,
        rect: &geom::Rect,
    ) -> (f32, f32) {
        let val_map = self.val_map(cm);
        let start = val_map.map_coord_num(val_start);
        let end = val_map.map_coord_num(val_end);
        match self {
            Self::Vertical => (rect.bottom() - start, rect.bottom() - end),
            Self::Horizontal => (rect.left() + start, rect.left() + end),
        }
    }

    fn add_series_path(
        &self,
        pb: &mut geom::PathBuilder,
        cat_coords: (f32, f32),
        val_coords: (f32, f32),
    ) {
        match self {
            Self::Vertical => {
                pb.move_to(cat_coords.0, val_coords.0);
                pb.line_to(cat_coords.1, val_coords.0);
                pb.line_to(cat_coords.1, val_coords.1);
                pb.line_to(cat_coords.0, val_coords.1);
            }
            Self::Horizontal => {
                pb.move_to(val_coords.0, cat_coords.0);
                pb.line_to(val_coords.1, cat_coords.0);
                pb.line_to(val_coords.1, cat_coords.1);
                pb.line_to(val_coords.0, cat_coords.1);
            }
        }
    }
}
