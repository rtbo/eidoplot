use axis::AsBoundRef;
use scale::{CoordMap, CoordMapXy};

use crate::drawing::plot::Orientation;
use crate::drawing::{Categories, ColumnExt, Error, F64ColumnExt, axis, legend, marker, scale};
use crate::{Style, data, geom, des, render, style};

/// trait implemented by series, or any other item that
/// has to populate the legend
pub trait SeriesExt {
    fn legend_entry(&self) -> Option<legend::Entry<'_>>;
}

impl SeriesExt for des::series::Line {
    fn legend_entry(&self) -> Option<legend::Entry<'_>> {
        self.name().map(|n| legend::Entry {
            label: n.as_ref(),
            font: None,
            shape: legend::ShapeRef::Line(self.stroke()),
        })
    }
}

impl SeriesExt for des::series::Scatter {
    fn legend_entry(&self) -> Option<legend::Entry<'_>> {
        self.name().map(|n| legend::Entry {
            label: n.as_ref(),
            font: None,
            shape: legend::ShapeRef::Marker(self.marker()),
        })
    }
}

impl SeriesExt for des::series::Histogram {
    fn legend_entry(&self) -> Option<legend::Entry<'_>> {
        self.name().map(|n| legend::Entry {
            label: n.as_ref(),
            font: None,
            shape: legend::ShapeRef::Rect(&self.fill(), self.line()),
        })
    }
}

impl SeriesExt for des::series::Bars {
    fn legend_entry(&self) -> Option<legend::Entry<'_>> {
        self.name().map(|n| legend::Entry {
            label: n.as_ref(),
            font: None,
            shape: legend::ShapeRef::Rect(self.fill(), self.line()),
        })
    }
}

impl SeriesExt for des::series::BarSeries {
    fn legend_entry(&self) -> Option<legend::Entry<'_>> {
        self.name().map(|n| legend::Entry {
            label: n.as_ref(),
            font: None,
            shape: legend::ShapeRef::Rect(&self.fill(), self.line()),
        })
    }
}

fn get_column<'a, D>(
    col: &'a des::series::DataCol,
    data_source: &'a D,
) -> Result<&'a dyn data::Column, Error>
where
    D: data::Source + ?Sized,
{
    match col {
        des::series::DataCol::Inline(col) => Ok(col),
        des::series::DataCol::SrcRef(name) => data_source
            .column(name)
            .ok_or_else(|| Error::MissingDataSrc(name.to_string())),
    }
}

fn calc_xy_bounds<D>(
    data_source: &D,
    x_data: &des::series::DataCol,
    y_data: &des::series::DataCol,
) -> Result<(axis::Bounds, axis::Bounds), Error>
where
    D: data::Source + ?Sized,
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
pub(super) struct AxisMatcher<'a> {
    pub(super) plt_idx: usize,
    pub(super) ax_idx: usize,
    pub(super) id: Option<&'a str>,
    pub(super) title: Option<&'a str>,
}

impl<'a> AxisMatcher<'a> {
    pub(super) fn matches_ref(
        &self,
        ax_ref: &des::axis::Ref,
        plt_idx: usize,
    ) -> Result<bool, Error> {
        match ax_ref {
            des::axis::Ref::Idx(ax_idx) => Ok(self.ax_idx == *ax_idx && self.plt_idx == plt_idx),
            des::axis::Ref::Id(id) => Ok(self.id == Some(id) || self.title == Some(id)),
            ax_ref => Err(Error::IllegalAxisRef(ax_ref.clone())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Series {
    plot: SeriesPlot,
    x_axis: des::axis::Ref,
    y_axis: des::axis::Ref,
}

#[derive(Debug, Clone)]
enum SeriesPlot {
    Line(Line),
    Scatter(Scatter),
    Histogram(Histogram),
    Bars(Bars),
    BarsGroup(BarsGroup),
}

impl Series {
    pub fn prepare<D>(index: usize, series: &des::Series, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source + ?Sized,
    {
        let plot = match &series {
            des::Series::Line(des) => SeriesPlot::Line(Line::prepare(index, des, data_source)?),
            des::Series::Scatter(des) => {
                SeriesPlot::Scatter(Scatter::prepare(index, des, data_source)?)
            }
            des::Series::Histogram(des) => {
                SeriesPlot::Histogram(Histogram::prepare(index, des, data_source)?)
            }
            des::Series::Bars(des) => SeriesPlot::Bars(Bars::prepare(index, des, data_source)?),
            des::Series::BarsGroup(des) => {
                SeriesPlot::BarsGroup(BarsGroup::prepare(index, des, data_source)?)
            }
        };

        let (x_axis, y_axis) = series.axes();

        Ok(Series {
            plot,
            x_axis: x_axis.clone(),
            y_axis: y_axis.clone(),
        })
    }

    pub fn axes(&self) -> (&des::axis::Ref, &des::axis::Ref) {
        (&self.x_axis, &self.y_axis)
    }

    /// Unites bounds for series whose axis matches with `matcher`
    pub fn unite_bounds<'a, S>(
        or: Orientation,
        series: S,
        starter: Option<axis::Bounds>,
        matcher: &AxisMatcher,
        plt_idx: usize,
    ) -> Result<Option<axis::Bounds>, Error>
    where
        S: IntoIterator<Item = &'a Series>,
    {
        let mut a: Option<axis::Bounds> = starter;
        for s in series {
            let axis = match or {
                Orientation::X => s.x_axis(),
                Orientation::Y => s.y_axis(),
            };
            if !matcher.matches_ref(axis, plt_idx)? {
                continue;
            }

            let b = match or {
                Orientation::X => &s.bounds().0,
                Orientation::Y => &s.bounds().1,
            };

            if let Some(a) = &mut a {
                a.unite_with(b)?;
            } else {
                a = Some(b.to_bounds());
            }
        }
        Ok(a)
    }

    fn bounds(&self) -> (axis::BoundsRef<'_>, axis::BoundsRef<'_>) {
        match &self.plot {
            SeriesPlot::Line(line) => (line.ab.0.as_bound_ref(), line.ab.1.as_bound_ref()),
            SeriesPlot::Scatter(scatter) => {
                (scatter.ab.0.as_bound_ref(), scatter.ab.1.as_bound_ref())
            }
            SeriesPlot::Histogram(hist) => (hist.ab.0.into(), hist.ab.1.into()),
            SeriesPlot::Bars(bars) => bars.bounds(),
            SeriesPlot::BarsGroup(bg) => (bg.bounds.0.as_bound_ref(), bg.bounds.1.as_bound_ref()),
        }
    }

    fn x_axis(&self) -> &des::axis::Ref {
        match &self.plot {
            SeriesPlot::Line(line) => &line.axes.0,
            SeriesPlot::Scatter(scatter) => &scatter.axes.0,
            SeriesPlot::Histogram(hist) => &hist.axes.0,
            SeriesPlot::Bars(bars) => &bars.axes.0,
            SeriesPlot::BarsGroup(bg) => &bg.axes.0,
        }
    }

    fn y_axis(&self) -> &des::axis::Ref {
        match &self.plot {
            SeriesPlot::Line(line) => &line.axes.1,
            SeriesPlot::Scatter(scatter) => &scatter.axes.1,
            SeriesPlot::Histogram(hist) => &hist.axes.1,
            SeriesPlot::Bars(bars) => &bars.axes.1,
            SeriesPlot::BarsGroup(bg) => &bg.axes.1,
        }
    }

    pub fn update_data<D>(
        &mut self,
        data_source: &D,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Result<(), Error>
    where
        D: data::Source + ?Sized,
    {
        match &mut self.plot {
            SeriesPlot::Line(xy) => {
                xy.update_data(data_source, rect, cm);
            }
            SeriesPlot::Scatter(sc) => sc.update_data(data_source, rect, cm),
            SeriesPlot::Histogram(hist) => {
                hist.update_data(rect, cm);
            }
            SeriesPlot::Bars(bars) => {
                bars.update_data(data_source, rect, cm);
            }
            SeriesPlot::BarsGroup(bg) => bg.update_data(data_source, rect, cm),
        }
        Ok(())
    }
}

impl Series {
    pub fn draw<S>(&self, surface: &mut S, style: &Style)
    where
        S: render::Surface,
    {
        match &self.plot {
            SeriesPlot::Line(xy) => xy.draw(surface, style),
            SeriesPlot::Scatter(sc) => sc.draw(surface, style),
            SeriesPlot::Histogram(hist) => hist.draw(surface, style),
            SeriesPlot::Bars(bars) => bars.draw(surface, style),
            SeriesPlot::BarsGroup(bg) => bg.draw(surface, style),
        }
    }
}

#[derive(Debug, Clone)]
struct Line {
    index: usize,
    cols: (des::DataCol, des::DataCol),
    ab: (axis::Bounds, axis::Bounds),
    axes: (des::axis::Ref, des::axis::Ref),
    path: Option<geom::Path>,
    stroke: style::series::Stroke,
    interpolation: des::series::Interpolation,
}

impl Line {
    fn prepare<D>(index: usize, des: &des::series::Line, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source + ?Sized,
    {
        let cols = (des.x_data().clone(), des.y_data().clone());
        let (x_bounds, y_bounds) = calc_xy_bounds(data_source, &cols.0, &cols.1)?;
        Ok(Line {
            index,
            cols,
            ab: (x_bounds, y_bounds),
            axes: (des.x_axis().clone(), des.y_axis().clone()),
            path: None,
            stroke: des.stroke().clone(),
            interpolation: des.interpolation(),
        })
    }

    fn update_data<D>(&mut self, data_source: &D, rect: &geom::Rect, cm: &CoordMapXy)
    where
        D: data::Source + ?Sized,
    {
        // unwraping here as data is checked during setup phase
        let x_col = get_column(&self.cols.0, data_source).unwrap();
        let y_col = get_column(&self.cols.1, data_source).unwrap();

        debug_assert!(x_col.len() == y_col.len());

        let path = match self.interpolation {
            des::series::Interpolation::Linear => {
                self.make_path_linear(rect, x_col, y_col, cm)
            }
            _ => todo!("Interpolation method {:?}", self.interpolation),
        };

        self.path = Some(path);
    }

    fn make_path_linear(&self, rect: &geom::Rect, x: &dyn data::Column, y: &dyn data::Column, cm: &CoordMapXy) -> geom::Path {
        let mut in_a_line = false;
        let mut pb = geom::PathBuilder::with_capacity(x.len() + 1, x.len());
        for (x, y) in x.sample_iter().zip(y.sample_iter()) {
            if x.is_null() || y.is_null() {
                in_a_line = false;
                continue;
            }
            let (x, y) = cm.map_coord((x, y)).expect("Should be valid coordinates");
            let x = rect.left() + x;
            let y = rect.bottom() - y;
            // if x_col.len() == 1024 {
            //     println!("  adding point {} {}", x, y);
            // }
            if in_a_line {
                pb.line_to(x, y);
            } else {
                pb.move_to(x, y);
                in_a_line = true;
            }
        }
        pb.finish().expect("Should be a valid path")
    }

    fn make_path_step_early(&self, rect: &geom::Rect, x: &dyn data::Column, y: &dyn data::Column, cm: &CoordMapXy) -> geom::Path {
        let mut pb = geom::PathBuilder::new();

        let mut prev_y: Option<f64> = None;

        for (x, y) in x.sample_iter().zip(y.sample_iter()) {
            if x.is_null() || y.is_null() {
                prev_y = None;
                continue;
            }
            let (x, y) = cm.map_coord((x, y)).expect("Should be valid coordinates");
            let x = rect.left() + x;
            let y = rect.bottom() - y;

            if let Some(px) = prev_x {
                pb.line_to(x, y);
                pb.line_to(x, y);
            } else {
                pb.move_to(x, y);
            }
            prev_x = Some(x);
        }

        pb.finish().expect("Should be a valid path")
    }

    fn draw<S>(&self, surface: &mut S, style: &Style)
    where
        S: render::Surface,
    {
        let rc = (style, self.index);

        let path = render::Path {
            path: self.path.as_ref().unwrap(),
            fill: None,
            stroke: Some(self.stroke.as_stroke(&rc)),
            transform: None,
        };
        surface.draw_path(&path);
    }
}

#[derive(Debug, Clone)]
struct Scatter {
    index: usize,
    cols: (des::DataCol, des::DataCol),
    ab: (axis::Bounds, axis::Bounds),
    axes: (des::axis::Ref, des::axis::Ref),
    path: geom::Path,
    points: Vec<geom::Point>,
    marker: style::series::Marker,
}

impl Scatter {
    fn prepare<D>(index: usize, des: &des::series::Scatter, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source + ?Sized,
    {
        let cols = (des.x_data().clone(), des.y_data().clone());
        let (x_bounds, y_bounds) = calc_xy_bounds(data_source, &cols.0, &cols.1)?;
        let path = marker::marker_path(des.marker());
        Ok(Scatter {
            index,
            cols,
            ab: (x_bounds, y_bounds),
            axes: (des.x_axis().clone(), des.y_axis().clone()),
            path,
            points: Vec::new(),
            marker: des.marker().clone(),
        })
    }

    fn update_data<D>(&mut self, data_source: &D, rect: &geom::Rect, cm: &CoordMapXy)
    where
        D: data::Source + ?Sized,
    {
        let x_col = get_column(&self.cols.0, data_source).unwrap();
        let y_col = get_column(&self.cols.1, data_source).unwrap();
        debug_assert!(x_col.len() == y_col.len());

        let mut points = Vec::with_capacity(x_col.len());

        for (x, y) in x_col.sample_iter().zip(y_col.sample_iter()) {
            if x.is_null() || y.is_null() {
                continue;
            }
            let (x, y) = cm.map_coord((x, y)).expect("Should be valid coordinates");
            let x = rect.left() + x;
            let y = rect.bottom() - y;
            points.push(geom::Point { x, y });
        }
        self.points = points;
    }

    fn draw<S>(&self, surface: &mut S, style: &Style)
    where
        S: render::Surface,
    {
        let rc = (style, self.index);

        for p in &self.points {
            let transform = geom::Transform::from_translate(p.x, p.y);
            let path = render::Path {
                path: &self.path,
                fill: self.marker.fill.as_ref().map(|f| f.as_paint(&rc)),
                stroke: self.marker.stroke.as_ref().map(|l| l.as_stroke(&rc)),
                transform: Some(&transform),
            };
            surface.draw_path(&path);
        }
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
    axes: (des::axis::Ref, des::axis::Ref),
    bins: Vec<HistBin>,
    path: Option<geom::Path>,
    fill: style::series::Fill,
    line: Option<style::series::Stroke>,
}

impl Histogram {
    fn prepare<D>(
        index: usize,
        hist: &des::series::Histogram,
        data_source: &D,
    ) -> Result<Self, Error>
    where
        D: data::Source + ?Sized,
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

        for x in col.f64_iter() {
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
            axes: (hist.x_axis().clone(), hist.y_axis().clone()),
            bins,
            path: None,
            fill: hist.fill().clone(),
            line: hist.line().cloned(),
        })
    }

    fn update_data(&mut self, rect: &geom::Rect, cm: &CoordMapXy) {
        let mut pb = geom::PathBuilder::new();
        let mut x = rect.left() + cm.x.map_coord_num(self.bins[0].range.0);
        let mut y = rect.bottom() - cm.y.map_coord_num(0.0);
        pb.move_to(x, y);

        for bin in self.bins.iter() {
            y = rect.bottom() - cm.y.map_coord_num(bin.value);
            pb.line_to(x, y);
            x = rect.left() + cm.x.map_coord_num(bin.range.1);
            pb.line_to(x, y);
        }

        y = rect.bottom() - cm.y.map_coord_num(0.0);
        pb.line_to(x, y);

        let path = pb.finish().expect("Should be a valid path");
        self.path = Some(path);
    }

    fn draw<S>(&self, surface: &mut S, style: &Style)
    where
        S: render::Surface,
    {
        let rc = (style, self.index);

        let path = render::Path {
            path: self.path.as_ref().unwrap(),
            fill: Some(self.fill.as_paint(&rc)),
            stroke: self.line.as_ref().map(|l| l.as_stroke(&rc)),
            transform: None,
        };
        surface.draw_path(&path);
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
    cols: (des::DataCol, des::DataCol),
    bounds: BarsBounds,
    axes: (des::axis::Ref, des::axis::Ref),
    position: des::series::BarsPosition,
    path: Option<geom::Path>,
    fill: style::series::Fill,
    line: Option<style::series::Stroke>,
}

impl Bars {
    fn prepare<D>(index: usize, des: &des::series::Bars, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source + ?Sized,
    {
        let cols = (des.x_data().clone(), des.y_data().clone());
        let (x_bounds, y_bounds) = calc_xy_bounds(data_source, &cols.0, &cols.1)?;

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

        Ok(Bars {
            index,
            cols,
            bounds,
            axes: (des.x_axis().clone(), des.y_axis().clone()),
            position: des.position().clone(),
            path: None,
            fill: des.fill().clone(),
            line: des.line().cloned(),
        })
    }

    fn bounds(&self) -> (axis::BoundsRef<'_>, axis::BoundsRef<'_>) {
        match &self.bounds {
            &BarsBounds::Vertical(ref x_bounds, y_bounds) => (x_bounds.into(), y_bounds.into()),
            &BarsBounds::Horizontal(x_bounds, ref y_bounds) => (x_bounds.into(), y_bounds.into()),
        }
    }

    fn update_data<D>(&mut self, data_source: &D, rect: &geom::Rect, cm: &CoordMapXy)
    where
        D: data::Source + ?Sized,
    {
        // unwraping here as data is checked during setup phase
        let x_col = get_column(&self.cols.0, data_source).unwrap();
        let y_col = get_column(&self.cols.1, data_source).unwrap();
        debug_assert!(x_col.len() == y_col.len());

        let mut pb = geom::PathBuilder::new();

        match &self.bounds {
            BarsBounds::Vertical(..) => {
                let cat_bin_width = cm.x.cat_bin_size();
                let y_start = rect.bottom() - cm.y.map_coord_num(0.0);

                for (x, y) in x_col.sample_iter().zip(y_col.sample_iter()) {
                    if x.is_null() || y.is_null() {
                        continue;
                    }

                    let (x, y) = cm.map_coord((x, y)).expect("Should be valid coordinates");
                    let x_start = rect.left() + x + cat_bin_width * (self.position.offset - 0.5);
                    let x_end = x_start + cat_bin_width * self.position.width;
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

                for (x, y) in x_col.sample_iter().zip(y_col.sample_iter()) {
                    if x.is_null() || y.is_null() {
                        continue;
                    }

                    let (x, y) = cm.map_coord((x, y)).expect("Should be valid coordinates");
                    let y_start = rect.bottom() - y - cat_bin_height * (self.position.offset - 0.5);
                    let y_end = y_start - cat_bin_height * self.position.width;
                    let x_end = rect.left() + x;
                    pb.move_to(x_start, y_start);
                    pb.line_to(x_end, y_start);
                    pb.line_to(x_end, y_end);
                    pb.line_to(x_start, y_end);
                }
            }
        }

        let path = pb.finish().expect("Should be a valid path");
        self.path = Some(path);
    }

    fn draw<S>(&self, surface: &mut S, style: &Style)
    where
        S: render::Surface,
    {
        let rc = (style, self.index);

        let path = render::Path {
            path: self.path.as_ref().unwrap(),
            fill: Some(self.fill.as_paint(&rc)),
            stroke: self.line.as_ref().map(|l| l.as_stroke(&rc)),
            transform: None,
        };
        surface.draw_path(&path);
    }
}

#[derive(Debug, Clone)]
pub struct BarsGroup {
    fst_index: usize,
    bounds: (axis::Bounds, axis::Bounds),
    axes: (des::axis::Ref, des::axis::Ref),
    orientation: des::series::BarsOrientation,
    arrangement: des::series::BarsArrangement,
    series: Vec<des::series::BarSeries>,
    series_paths: Vec<geom::Path>,
}

impl BarsGroup {
    fn prepare<D>(index: usize, des: &des::series::BarsGroup, data_source: &D) -> Result<Self, Error>
    where
        D: data::Source + ?Sized,
    {
        let cat_col = get_column(des.categories(), data_source)?;
        let categories: Categories = cat_col
            .str()
            .ok_or_else(|| {
                Error::InconsistentData("BarsGroup categories must be a string column".to_string())
            })?
            .into();

        let mut bounds_per_cat: Vec<axis::NumBounds> =
            vec![axis::NumBounds::from(0.0); categories.len()];

        for bs in des.series() {
            let data_col = get_column(bs.data(), data_source)?;
            if data_col.len() != categories.len() {
                return Err(Error::InconsistentData(
                    "BarsGroup data must be the same length as categories".to_string(),
                ));
            }
            let data_col = data_col.f64().ok_or(Error::InconsistentData(
                "BarsGroup data must be numeric".to_string(),
            ))?;

            for (v, bounds) in data_col.f64_iter().zip(bounds_per_cat.iter_mut()) {
                if let Some(v) = v {
                    match des.arrangement() {
                        des::series::BarsArrangement::Aside(..) => {
                            bounds.add_sample(v);
                        }
                        des::series::BarsArrangement::Stack(..) => {
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

        let bounds = match des.orientation() {
            des::series::BarsOrientation::Vertical => {
                (axis::Bounds::Cat(categories), axis::Bounds::Num(num_bounds))
            }
            des::series::BarsOrientation::Horizontal => {
                (axis::Bounds::Num(num_bounds), axis::Bounds::Cat(categories))
            }
        };

        Ok(BarsGroup {
            fst_index: index,
            bounds,
            axes: (des.x_axis().clone(), des.y_axis().clone()),
            orientation: des.orientation().clone(),
            arrangement: des.arrangement().clone(),
            series: des.series().to_vec(),
            series_paths: Vec::new(),
        })
    }

    fn update_data<D>(&mut self, data_source: &D, rect: &geom::Rect, cm: &CoordMapXy)
    where
        D: data::Source + ?Sized,
    {
        let categories = match self.orientation {
            des::series::BarsOrientation::Vertical => self.bounds.0.as_cat().unwrap(),
            des::series::BarsOrientation::Horizontal => self.bounds.1.as_cat().unwrap(),
        };

        let paths = match self.arrangement {
            des::series::BarsArrangement::Aside(aside) => {
                self.build_paths_aside(data_source, &aside, categories, rect, cm)
            }
            des::series::BarsArrangement::Stack(stack) => {
                self.build_paths_stack(data_source, &stack, categories, rect, cm)
            }
        };
        self.series_paths = paths;
    }

    fn build_paths_aside<D>(
        &self,
        data_source: &D,
        arrangement: &des::series::BarsAsideArrangement,
        categories: &Categories,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Vec<geom::Path>
    where
        D: data::Source + ?Sized,
    {
        let num_series = self.series.len();
        if num_series == 0 {
            return Vec::new();
        }
        let num_gaps = num_series - 1;

        let des::series::BarsAsideArrangement {
            mut offset,
            width,
            gap,
        } = *arrangement;
        let width = (width - gap * num_gaps as f32) / num_series as f32;

        let mut paths = Vec::with_capacity(num_series);

        for series in &self.series {
            let data_col = get_column(series.data(), data_source).unwrap();
            let data_col = data_col.f64().unwrap();

            let mut pb = geom::PathBuilder::new();

            for (cat, val) in categories.iter().zip(data_col.f64_iter()) {
                let Some(val) = val else { continue };

                let val_start = 0.0;
                let val_end = val_start + val;

                let cat_coords = self.orientation.cat_coords(cm, cat, offset, width, rect);
                let val_coords = self.orientation.val_coords(cm, val_start, val_end, rect);
                self.orientation
                    .add_series_path(&mut pb, cat_coords, val_coords);
            }

            let path = pb.finish().expect("Failed to build path");
            paths.push(path);

            offset += width + gap;
        }
        paths
    }

    fn build_paths_stack<D>(
        &self,
        data_source: &D,
        arrangement: &des::series::BarsStackArrangement,
        categories: &Categories,
        rect: &geom::Rect,
        cm: &CoordMapXy,
    ) -> Vec<geom::Path>
    where
        D: data::Source + ?Sized,
    {
        let mut cat_values = vec![0.0; categories.len()];

        let mut paths = Vec::with_capacity(self.series.len());

        for series in &self.series {
            let data_col = get_column(series.data(), data_source).unwrap();
            let data_col = data_col.f64().unwrap();

            let mut pb = geom::PathBuilder::new();

            for (idx, (cat, val)) in categories.iter().zip(data_col.f64_iter()).enumerate() {
                let Some(val) = val else { continue };

                let val_start = cat_values[idx];
                let val_end = val_start + val;

                cat_values[idx] = val_end;

                let cat_coords = self.orientation.cat_coords(
                    cm,
                    cat,
                    arrangement.offset,
                    arrangement.width,
                    rect,
                );
                let val_coords = self.orientation.val_coords(cm, val_start, val_end, rect);
                self.orientation
                    .add_series_path(&mut pb, cat_coords, val_coords);
            }

            let path = pb.finish().expect("Failed to build path");
            paths.push(path);
        }
        paths
    }

    fn draw<S>(&self, surface: &mut S, style: &Style)
    where
        S: render::Surface,
    {
        let mut col_idx = self.fst_index;

        for (series, path) in self.series.iter().zip(self.series_paths.iter()) {
            let rc = (style, col_idx);
            col_idx += 1;

            let rpath = render::Path {
                path,
                fill: Some(series.fill().as_paint(&rc)),
                stroke: series.line().map(|l| l.as_stroke(&rc)),
                transform: None,
            };
            surface.draw_path(&rpath);
        }
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

impl BarsOrientationExt for des::series::BarsOrientation {
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
