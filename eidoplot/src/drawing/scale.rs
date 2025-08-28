use crate::data;
use crate::drawing::axis;
use crate::ir;

/// Maps coordinates from data space to surface space.
/// The surface space starts at zero for lowest displayed data and goes up for higher data.
/// Typically, only one of the two map_coord_num or map_coord_cat should be implemented,
/// depending on whether the scale is numerical or categorical.
pub trait CoordMap: std::fmt::Debug {
    fn map_coord(&self, sample: data::Sample) -> Option<f32> {
        match sample {
            data::Sample::Num(n) => Some(self.map_coord_num(n)),
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

    fn axis_bounds(&self) -> axis::BoundsRef<'_>;

    fn set_plot_size(&mut self, plot_size: f32);
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
        ir::axis::Scale::Linear(ir::axis::Range::MinAuto(min)) => Box::new(LinCoordMap::new(
            plot_size,
            (0.0, insets.1),
            (*min, axis_bounds.end()).into(),
        )),
        ir::axis::Scale::Linear(ir::axis::Range::AutoMax(max)) => Box::new(LinCoordMap::new(
            plot_size,
            (insets.0, 0.0),
            (axis_bounds.start(), *max).into(),
        )),
        ir::axis::Scale::Linear(ir::axis::Range::MinMax(min, max)) => {
            Box::new(LinCoordMap::new(plot_size, (0.0, 0.0), (*min, *max).into()))
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LinCoordMap {
    plot_size: f32,
    ab: axis::NumBounds,

    // next fields are there for `set_plot_size`
    orig_ab: axis::NumBounds,
    insets: (f32, f32),
}

impl LinCoordMap {
    fn new(plot_size: f32, insets: (f32, f32), ab: axis::NumBounds) -> Self {
        let orig_ab = ab;
        let ab = Self::extend_bounds_with_insets(plot_size, insets, ab);

        LinCoordMap {
            plot_size,
            ab,
            orig_ab,
            insets,
        }
    }

    fn extend_bounds_with_insets(plot_size: f32, insets: (f32, f32), ab: axis::NumBounds) -> axis::NumBounds {
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

    fn set_plot_size(&mut self, plot_size: f32) {
        self.plot_size = plot_size;
        self.ab = Self::extend_bounds_with_insets(plot_size, self.insets, self.orig_ab);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_scale_coord_linear_auto() {
        let linear_auto = ir::axis::Scale::Linear(ir::axis::Range::Auto);

        let map = map_scale_coord_num(&linear_auto, 100.0, &(0.0, 10.0).into(), (0.0, 0.0));
        assert_eq!(map.map_coord_num(0.0), 0.0);
        assert_eq!(map.map_coord_num(5.0), 50.0);
        assert_eq!(map.map_coord_num(10.0), 100.0);
        assert_eq!(map.axis_bounds(), axis::Bounds::Num((0.0, 10.0).into()));

        let map = map_scale_coord_num(&linear_auto, 110.0, &(0.0, 10.0).into(), (10.0, 0.0));
        assert_eq!(map.map_coord_num(0.0), 10.0);
        assert_eq!(map.map_coord_num(5.0), 60.0);
        assert_eq!(map.map_coord_num(10.0), 110.0);
        assert_eq!(map.axis_bounds(), axis::Bounds::Num((-1.0, 10.0).into()));

        let map = map_scale_coord_num(&linear_auto, 120.0, &(0.0, 10.0).into(), (10.0, 10.0));
        assert_eq!(map.map_coord_num(0.0), 10.0);
        assert_eq!(map.map_coord_num(5.0), 60.0);
        assert_eq!(map.map_coord_num(10.0), 110.0);
        assert_eq!(map.axis_bounds(), axis::Bounds::Num((-1.0, 11.0).into()));
    }
}
