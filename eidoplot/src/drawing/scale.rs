use crate::drawing::axis;
use crate::ir;

pub fn map_scale_coord(
    scale: &ir::axis::Scale,
    mesh_size: f32,
    axis_bounds: axis::NumBounds,
    insets: (f32, f32),
) -> Box<dyn CoordMap> {
    match scale {
        ir::axis::Scale::Linear(ir::axis::Range::Auto) => {
            Box::new(LinCoordMap::new(mesh_size, insets, axis_bounds))
        }
        ir::axis::Scale::Linear(ir::axis::Range::MinAuto(min)) => Box::new(LinCoordMap::new(
            mesh_size,
            (0.0, insets.1),
            (*min, axis_bounds.end()).into(),
        )),
        ir::axis::Scale::Linear(ir::axis::Range::AutoMax(max)) => Box::new(LinCoordMap::new(
            mesh_size,
            (insets.0, 0.0),
            (axis_bounds.start(), *max).into(),
        )),
        ir::axis::Scale::Linear(ir::axis::Range::MinMax(min, max)) => {
            Box::new(LinCoordMap::new(mesh_size, (0.0, 0.0), (*min, *max).into()))
        }
    }
}

/// Maps and unmaps coordinates between data space and surface space.
/// The surface space starts at zero for lowest displayed and goes up for higher data.
pub trait CoordMap {
    fn map_coord(&self, x: f64) -> f32;
    fn axis_bounds(&self) -> axis::NumBounds;
}

pub struct CoordMapXy<'a> {
    pub x: &'a dyn CoordMap,
    pub y: &'a dyn CoordMap,
}

impl<'a> CoordMapXy<'a> {
    pub fn map_coord(&self, dp: (f64, f64)) -> (f32, f32) {
        (self.x.map_coord(dp.0), self.y.map_coord(dp.1))
    }
}

struct LinCoordMap {
    plot_size: f32,
    ab: axis::NumBounds,
}

impl LinCoordMap {
    fn new(plot_size: f32, inset: (f32, f32), ab: axis::NumBounds) -> Self {
        // map the data space to the surface space
        let plot_to_data = ab.span() / (plot_size - inset.0 - inset.1) as f64;
        let ab = axis::NumBounds::from((
            ab.start() - inset.0 as f64 * plot_to_data,
            ab.end() + inset.1 as f64 * plot_to_data,
        ));

        LinCoordMap { plot_size, ab }
    }
}

impl CoordMap for LinCoordMap {
    fn map_coord(&self, x: f64) -> f32 {
        let ratio = (x - self.ab.start()) / self.ab.span();
        ratio as f32 * self.plot_size
    }

    fn axis_bounds(&self) -> axis::NumBounds {
        self.ab
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_scale_coord_linear_auto() {
        let linear_auto = ir::axis::Scale::Linear(ir::axis::Range::Auto);

        let map = map_scale_coord(&linear_auto, 100.0, (0.0, 10.0).into(), (0.0, 0.0));
        assert_eq!(map.map_coord(0.0), 0.0);
        assert_eq!(map.map_coord(5.0), 50.0);
        assert_eq!(map.map_coord(10.0), 100.0);
        assert_eq!(map.axis_bounds(), (0.0, 10.0).into());

        let map = map_scale_coord(&linear_auto, 110.0, (0.0, 10.0).into(), (10.0, 0.0));
        assert_eq!(map.map_coord(0.0), 10.0);
        assert_eq!(map.map_coord(5.0), 60.0);
        assert_eq!(map.map_coord(10.0), 110.0);
        assert_eq!(map.axis_bounds(), (-1.0, 10.0).into());

        let map = map_scale_coord(&linear_auto, 120.0, (0.0, 10.0).into(), (10.0, 10.0));
        assert_eq!(map.map_coord(0.0), 10.0);
        assert_eq!(map.map_coord(5.0), 60.0);
        assert_eq!(map.map_coord(10.0), 110.0);
        assert_eq!(map.axis_bounds(), (-1.0, 11.0).into());
    }
}
