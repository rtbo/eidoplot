use crate::drawing::{Categories, Error};

/// Bounds of an axis
#[derive(Debug, Clone, PartialEq)]
pub enum Bounds {
    /// Numeric bounds, used by both float and integer
    Num(NumBounds),
    /// Category bounds
    Cat(Categories),
}

impl From<NumBounds> for Bounds {
    fn from(value: NumBounds) -> Self {
        Self::Num(value)
    }
}

impl From<Categories> for Bounds {
    fn from(value: Categories) -> Self {
        Self::Cat(value)
    }
}

impl Bounds {
    pub fn unite_with(&mut self, other: &Bounds) -> Result<(), Error> {
        match (self, other) {
            (Bounds::Num(a), Bounds::Num(b)) => {
                a.unite_with(b);
                Ok(())
            }
            (Bounds::Cat(a), Bounds::Cat(b)) => {
                for s in b.iter() {
                    a.push_if_not_present(s);
                }
                Ok(())
            }
            _ => Err(Error::InconsistentAxisBounds(
                "Cannot unite numerical and categorical axis bounds".into(),
            )),
        }
    }
}

/// Bounds of an axis, borrowing internal its data
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundsRef<'a> {
    /// Numeric bounds, used by both float and integer
    Num(NumBounds),
    /// Category bounds
    Cat(&'a Categories),
}

impl From<NumBounds> for BoundsRef<'_> {
    fn from(value: NumBounds) -> Self {
        Self::Num(value)
    }
}

impl<'a> From<&'a Categories> for BoundsRef<'a> {
    fn from(value: &'a Categories) -> Self {
        Self::Cat(value)
    }
}

impl BoundsRef<'_> {
    pub fn as_num(&self) -> Option<NumBounds> {
        match self {
            &BoundsRef::Num(n) => Some(n),
            _ => None,
        }
    }
}

impl std::cmp::PartialEq<Bounds> for BoundsRef<'_> {
    fn eq(&self, other: &Bounds) -> bool {
        match (self, other) {
            (&BoundsRef::Num(a), &Bounds::Num(b)) => a == b,
            (&BoundsRef::Cat(a), Bounds::Cat(b)) => a == b,
            _ => false,
        }
    }
}

impl std::cmp::PartialEq<BoundsRef<'_>> for Bounds {
    fn eq(&self, other: &BoundsRef) -> bool {
        match (self, other) {
            (&Bounds::Num(a), &BoundsRef::Num(b)) => a == b,
            (Bounds::Cat(a), &BoundsRef::Cat(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumBounds(f64, f64);

impl NumBounds {
    pub const NAN: Self = Self(f64::NAN, f64::NAN);
}

impl Default for NumBounds {
    fn default() -> Self {
        Self::NAN
    }
}

impl From<f64> for NumBounds {
    fn from(value: f64) -> Self {
        Self(value, value)
    }
}

impl From<(f64, f64)> for NumBounds {
    fn from(value: (f64, f64)) -> Self {
        Self(value.0.min(value.1), value.0.max(value.1))
    }
}

impl NumBounds {
    pub fn start(&self) -> f64 {
        self.0
    }

    pub fn end(&self) -> f64 {
        self.1
    }

    pub fn span(&self) -> f64 {
        self.1 - self.0
    }

    pub fn contains(&self, point: f64) -> bool {
        // TODO: handle very large and very low values
        const EPS: f64 = 1e-10;
        point >= (self.0 - EPS) && point <= (self.1 + EPS)
    }

    pub fn add_sample(&mut self, point: f64) {
        self.0 = self.0.min(point);
        self.1 = self.1.max(point);
    }

    pub fn unite_with(&mut self, bounds: &NumBounds) {
        self.0 = self.0.min(bounds.0);
        self.1 = self.1.max(bounds.1);
    }
}
