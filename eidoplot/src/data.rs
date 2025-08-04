#[derive(Debug, Clone, Copy)]
pub struct ViewBounds(f64, f64);

impl ViewBounds {
    pub const NAN: Self = Self(f64::NAN, f64::NAN);
}

impl Default for ViewBounds {
    fn default() -> Self {
        Self::NAN
    }
}

impl From<f64> for ViewBounds {
    fn from(value: f64) -> Self {
        Self(value, value)
    }
}

impl From<(f64, f64)> for ViewBounds {
    fn from(value: (f64, f64)) -> Self {
        Self(value.0, value.1)
    }
}

impl ViewBounds {
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

    pub fn add_bounds(&mut self, bounds: ViewBounds) {
        self.0 = self.0.min(bounds.0);
        self.1 = self.1.max(bounds.1);
    }
}
