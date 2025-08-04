#[derive(Debug, Clone, Copy)]
pub struct Bounds(f64, f64);

impl Bounds {
    pub const NAN: Self = Self(f64::NAN, f64::NAN);
}

impl Default for Bounds {
    fn default() -> Self {
        Self::NAN
    }
}

impl From<f64> for Bounds {
    fn from(value: f64) -> Self {
        Self(value, value)
    }
}

impl Bounds {
    pub fn min(&self) -> f64 {
        self.0
    }

    pub fn max(&self) -> f64 {
        self.1
    }

    pub fn span(&self) -> f64 {
        self.1 - self.0
    }

    pub fn center(&self) -> f64 {
        (self.0 + self.1) / 2.0
    }

    pub fn contains(&self, point: f64) -> bool {
        // TODO: handle very large and very low values
        const EPS: f64 = 1e-10;
        point >= (self.0 - EPS) && point <= (self.1 + EPS)
    }

    pub fn add_point(&mut self, point: f64) {
        self.0 = self.0.min(point);
        self.1 = self.1.max(point);
    }

    pub fn add_bounds(&mut self, bounds: Bounds) {
        self.0 = self.0.min(bounds.0);
        self.1 = self.1.max(bounds.1);
    }
}
