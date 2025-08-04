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
    pub fn coord_mapper(&self, mesh_size: f32, data_bounds: data::Bounds) -> Box<dyn MapCoord> {
        match self {
            Scale::Linear(Range::Auto) => Box::new(LinAutoCoordMap {
                sz: mesh_size,
                db: data_bounds,
            }),
            _ => todo!(),
        }
    }
}

pub trait MapCoord {
    fn map_coord(&self, x: f64) -> f32;
}

struct LinAutoCoordMap {
    sz: f32,
    db: data::Bounds,
}

impl MapCoord for LinAutoCoordMap {
    fn map_coord(&self, x: f64) -> f32 {
        let ratio = (x - self.db.min()) / self.db.span();
        ratio as f32 * self.sz
    }
}
