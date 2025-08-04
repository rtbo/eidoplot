use crate::data;

/// Describe the bounds of an axis in data space
#[derive(Debug, Clone, Copy)]
pub enum Range {
    Auto,
    MinAuto(f64),
    AutoMax(f64),
    MinMax(f64, f64),
}

/// Describes the type of an axis
#[derive(Debug, Clone, Copy)]
pub enum Scale {
    Linear(Range),
}

impl Default for Scale {
    fn default() -> Self {
        Scale::Linear(Range::Auto)
    }
}

impl Scale {
    pub fn coord_mapper(&self, mesh_size: f32, data_bounds: data::ViewBounds) -> Box<dyn MapCoord> {
        match self {
            Scale::Linear(Range::Auto) => Box::new(LinCoordMap {
                sz: mesh_size,
                vb: data_bounds,
            }),
            Scale::Linear(Range::MinAuto(min)) => {
                Box::new(LinCoordMap {
                    sz: mesh_size,
                    vb: (*min, data_bounds.max()).into(),
                })
            },
            Scale::Linear(Range::AutoMax(max)) => {
                Box::new(LinCoordMap {
                    sz: mesh_size,
                    vb: (data_bounds.min(), *max).into(),
                })
            },
            Scale::Linear(Range::MinMax(min, max)) => {
                Box::new(LinCoordMap {
                    sz: mesh_size,
                    vb: (*min, *max).into(),
                })
            },
        }
    }
}

/// Maps and unmaps coordinates between data space and surface space.
/// The surface space starts at zero for lowest displayed and goes up for higher data.
pub trait MapCoord {
    fn map_coord(&self, x: f64) -> f32;
    fn unmap_coord(&self, x: f32) -> f64;
    fn view_bounds(&self) -> data::ViewBounds;
}

struct LinCoordMap {
    sz: f32,
    vb: data::ViewBounds,
}

impl MapCoord for LinCoordMap {
    fn map_coord(&self, x: f64) -> f32 {
        let ratio = (x - self.vb.min()) / self.vb.span();
        ratio as f32 * self.sz
    }
    fn unmap_coord(&self, x: f32) -> f64 {
        let ratio = (x / self.sz) as f64;
        self.vb.min() + ratio * self.vb.span()
    }

    fn view_bounds(&self) -> data::ViewBounds {
        self.vb
    }
}
