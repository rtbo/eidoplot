use crate::{drawing::axis, ir};

pub fn map_scale_coord(
    scale: &ir::axis::Scale,
    mesh_size: f32,
    axis_bounds: axis::NumBounds,
    insets: (f32, f32),
) -> Box<dyn CoordMap> {
    match scale {
        ir::axis::Scale::Linear(ir::axis::Range::Auto) => Box::new(LinCoordMap {
            offset: insets.0,
            scale: mesh_size - (insets.0 + insets.1),
            ab: axis_bounds,
        }),
        ir::axis::Scale::Linear(ir::axis::Range::MinAuto(min)) => Box::new(LinCoordMap {
            offset: 0.0,
            scale: mesh_size - insets.1,
            ab: (*min, axis_bounds.end()).into(),
        }),
        ir::axis::Scale::Linear(ir::axis::Range::AutoMax(max)) => Box::new(LinCoordMap {
            offset: insets.0,
            scale: mesh_size - insets.0,
            ab: (axis_bounds.start(), *max).into(),
        }),
        ir::axis::Scale::Linear(ir::axis::Range::MinMax(min, max)) => Box::new(LinCoordMap {
            offset: 0.0,
            scale: mesh_size,
            ab: (*min, *max).into(),
        }),
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
    offset: f32,
    scale: f32,
    ab: axis::NumBounds,
}

impl CoordMap for LinCoordMap {
    fn map_coord(&self, x: f64) -> f32 {
        let ratio = (x - self.ab.start()) / self.ab.span();
        ratio as f32 * self.scale + self.offset
    }

    fn axis_bounds(&self) -> axis::NumBounds {
        self.ab
    }
}
