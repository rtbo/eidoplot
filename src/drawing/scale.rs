use crate::drawing::axis;
use crate::{data, ir};

/// Maps coordinates from data space to surface space.
/// The surface space starts at zero for lowest displayed data and goes up for higher data.
/// Typically, only one of the two map_coord_num or map_coord_cat should be implemented,
/// depending on whether the scale is numerical or categorical.
pub trait CoordMap: std::fmt::Debug {
    fn map_coord(&self, sample: data::Sample) -> Option<f32> {
        match sample {
            data::Sample::Num(n) => Some(self.map_coord_num(n)),
            data::Sample::Time(n) => Some(self.map_coord_num(n.timestamp())),
            data::Sample::Cat(c) => Some(self.map_coord_cat(c)),
            _ => None,
        }
    }

    fn map_coord_num(&self, _num: f64) -> f32 {
        unimplemented!("Only for numerical scales");
    }

    fn map_coord_cat(&self, _cat: &str) -> f32 {
        unimplemented!("Only for categorical scales");
    }

    /// Get the size of a category bin (width for horizontal axes, height for vertical axes)
    fn cat_bin_size(&self) -> f32 {
        unimplemented!("Only for categorical scales");
    }

    fn axis_bounds(&self) -> axis::BoundsRef<'_>;
}

#[derive(Debug, Clone, Copy)]
pub struct CoordMapXy<'a> {
    pub x: &'a dyn CoordMap,
    pub y: &'a dyn CoordMap,
}

impl<'a> CoordMapXy<'a> {
    pub fn map_coord(&self, dp: (data::Sample, data::Sample)) -> Option<(f32, f32)> {
        self.x
            .map_coord(dp.0)
            .and_then(|x| self.y.map_coord(dp.1).map(|y| (x, y)))
    }
}

pub fn map_scale_coord_num(
    scale: &ir::axis::Scale,
    plot_size: f32,
    axis_bounds: &axis::NumBounds,
    insets: (f32, f32),
) -> Box<dyn CoordMap> {
    match scale {
        ir::axis::Scale::Auto | ir::axis::Scale::Linear(ir::axis::Range::Auto) => {
            Box::new(LinCoordMap::new(plot_size, insets, *axis_bounds))
        }
        ir::axis::Scale::Linear(range) => {
            let (adj_nb, adj_insets) = adjusted_nb_insets(*range, axis_bounds, insets);
            Box::new(LinCoordMap::new(plot_size, adj_insets, adj_nb))
        }
        ir::axis::Scale::Log(ir::axis::LogScale { base, range }) => {
            let (adj_nb, adj_insets) = adjusted_nb_insets(*range, axis_bounds, insets);
            Box::new(LogCoordMap::new(*base, plot_size, adj_insets, adj_nb))
        }
        ir::axis::Scale::Shared(..) => unreachable!("shared scale to be handled upfront"),
    }
}

fn adjusted_nb_insets(
    range: ir::axis::Range,
    nb: &axis::NumBounds,
    insets: (f32, f32),
) -> (axis::NumBounds, (f32, f32)) {
    match range {
        ir::axis::Range::Auto => (*nb, insets),
        ir::axis::Range::MinAuto(min) => ((min, nb.end()).into(), (0.0, insets.1)),
        ir::axis::Range::AutoMax(max) => ((nb.start(), max).into(), (insets.0, 0.0)),
        ir::axis::Range::MinMax(min, max) => ((min, max).into(), (0.0, 0.0)),
    }
}

#[derive(Debug, Clone, Copy)]
struct LinCoordMap {
    plot_size: f32,
    ab: axis::NumBounds,
}

impl LinCoordMap {
    fn new(plot_size: f32, insets: (f32, f32), ab: axis::NumBounds) -> Self {
        let ab = Self::extend_bounds_with_insets(plot_size, insets, ab);

        LinCoordMap { plot_size, ab }
    }

    fn extend_bounds_with_insets(
        plot_size: f32,
        insets: (f32, f32),
        ab: axis::NumBounds,
    ) -> axis::NumBounds {
        let plot_to_data = ab.span() / (plot_size - insets.0 - insets.1) as f64;
        axis::NumBounds::from((
            ab.start() - insets.0 as f64 * plot_to_data,
            ab.end() + insets.1 as f64 * plot_to_data,
        ))
    }
}

impl CoordMap for LinCoordMap {
    fn map_coord_num(&self, x: f64) -> f32 {
        let ratio = (x - self.ab.start()) / self.ab.span();
        ratio as f32 * self.plot_size
    }

    fn axis_bounds(&self) -> axis::BoundsRef<'_> {
        self.ab.into()
    }
}

#[derive(Debug, Clone, Copy)]
struct LogCoordMap {
    base: f64,
    plot_size: f32,
    ab: axis::NumBounds,
}

impl LogCoordMap {
    fn new(base: f64, plot_size: f32, insets: (f32, f32), ab: axis::NumBounds) -> Self {
        let ab = Self::extend_bounds_with_insets(base, plot_size, insets, ab);

        LogCoordMap {
            base,
            plot_size,
            ab,
        }
    }

    fn extend_bounds_with_insets(
        base: f64,
        plot_size: f32,
        insets: (f32, f32),
        ab: axis::NumBounds,
    ) -> axis::NumBounds {
        let plot_to_data = ab.log_span(base) / (plot_size - insets.0 - insets.1) as f64;

        axis::NumBounds::from((
            ab.start() / base.powf(insets.0 as f64 * plot_to_data),
            ab.end() * base.powf(insets.1 as f64 * plot_to_data),
        ))
    }
}

impl CoordMap for LogCoordMap {
    fn map_coord_num(&self, x: f64) -> f32 {
        let start = self.ab.start().log(self.base);
        let end = self.ab.end().log(self.base);
        let x = x.log(self.base);
        let ratio = (x - start) / (end - start);
        ratio as f32 * self.plot_size
    }

    fn axis_bounds(&self) -> axis::BoundsRef<'_> {
        self.ab.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{Near, assert_near};

    #[test]
    fn test_map_scale_coord_linear_auto() {
        let linear_auto = ir::axis::Scale::Linear(ir::axis::Range::Auto);

        let map = map_scale_coord_num(&linear_auto, 100.0, &(0.0, 10.0).into(), (0.0, 0.0));
        assert_near!(rel, map.map_coord_num(0.0), 0.0);
        assert_near!(rel, map.map_coord_num(5.0), 50.0);
        assert_near!(rel, map.map_coord_num(10.0), 100.0);
        assert_near!(
            rel,
            map.axis_bounds(),
            axis::BoundsRef::Num((0.0, 10.0).into())
        );

        let map = map_scale_coord_num(&linear_auto, 110.0, &(0.0, 10.0).into(), (10.0, 0.0));
        assert_near!(rel, map.map_coord_num(0.0), 10.0);
        assert_near!(rel, map.map_coord_num(5.0), 60.0);
        assert_near!(rel, map.map_coord_num(10.0), 110.0);
        assert_near!(
            rel,
            map.axis_bounds(),
            axis::BoundsRef::Num((-1.0, 10.0).into())
        );

        let map = map_scale_coord_num(&linear_auto, 120.0, &(0.0, 10.0).into(), (10.0, 10.0));
        assert_near!(rel, map.map_coord_num(0.0), 10.0);
        assert_near!(rel, map.map_coord_num(5.0), 60.0);
        assert_near!(rel, map.map_coord_num(10.0), 110.0);
        assert_near!(
            rel,
            map.axis_bounds(),
            axis::BoundsRef::Num((-1.0, 11.0).into())
        );
    }

    #[test]
    fn test_map_scale_coord_log_auto() {
        let log_auto = ir::axis::Scale::Log(ir::axis::LogScale {
            base: 10.0,
            range: ir::axis::Range::Auto,
        });
        let axis_bounds = (1e-5, 1e5).into();

        let map = map_scale_coord_num(&log_auto, 100.0, &axis_bounds, (0.0, 0.0));
        assert_near!(rel, map.map_coord_num(1e-5), 0.0);
        assert_near!(rel, map.map_coord_num(1.0), 50.0);
        assert_near!(rel, map.map_coord_num(1e5), 100.0);
        assert_near!(
            rel,
            map.axis_bounds(),
            axis::BoundsRef::Num((1e-5, 1e5).into())
        );

        let map = map_scale_coord_num(&log_auto, 110.0, &axis_bounds, (10.0, 0.0));
        assert_near!(rel, map.map_coord_num(1e-5), 10.0);
        assert_near!(rel, map.map_coord_num(1.0), 60.0);
        assert_near!(rel, map.map_coord_num(1e5), 110.0);
        assert_near!(
            rel,
            map.axis_bounds(),
            axis::BoundsRef::Num((1e-6, 1e5).into())
        );

        let map = map_scale_coord_num(&log_auto, 120.0, &axis_bounds, (10.0, 10.0));
        assert_near!(rel, map.map_coord_num(1e-5), 10.0);
        assert_near!(rel, map.map_coord_num(1.0), 60.0);
        assert_near!(rel, map.map_coord_num(1e5), 110.0);
        assert_near!(
            rel,
            map.axis_bounds(),
            axis::BoundsRef::Num((1e-6, 1e6).into())
        );
    }
}
