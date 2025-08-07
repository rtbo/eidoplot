use crate::{data, ir::axis};

pub fn map_scale_coord(
    scale: &axis::Scale,
    mesh_size: f32,
    data_bounds: data::ViewBounds,
    insets: f32,
) -> Box<dyn CoordMap> {
    match scale {
        axis::Scale::Linear(axis::Range::Auto) => Box::new(LinCoordMap {
            offset: insets,
            scale: mesh_size - 2.0*insets,
            vb: data_bounds,
        }),
        axis::Scale::Linear(axis::Range::MinAuto(min)) => Box::new(LinCoordMap {
            offset: 0.0,
            scale: mesh_size - insets,
            vb: (*min, data_bounds.max()).into(),
        }),
        axis::Scale::Linear(axis::Range::AutoMax(max)) => Box::new(LinCoordMap {
            offset: insets,
            scale: mesh_size - insets,
            vb: (data_bounds.min(), *max).into(),
        }),
        axis::Scale::Linear(axis::Range::MinMax(min, max)) => Box::new(LinCoordMap {
            offset: 0.0,
            scale: mesh_size,
            vb: (*min, *max).into(),
        }),
    }
}

/// Maps and unmaps coordinates between data space and surface space.
/// The surface space starts at zero for lowest displayed and goes up for higher data.
pub trait CoordMap {
    fn map_coord(&self, x: f64) -> f32;
    fn view_bounds(&self) -> data::ViewBounds;
}

struct LinCoordMap {
    offset: f32,
    scale: f32,
    vb: data::ViewBounds,
}

impl CoordMap for LinCoordMap {
    fn map_coord(&self, x: f64) -> f32 {
        let ratio = (x - self.vb.min()) / self.vb.span();
        ratio as f32 * self.scale + self.offset
    }

    fn view_bounds(&self) -> data::ViewBounds {
        self.vb
    }
}
