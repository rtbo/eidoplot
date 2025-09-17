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
    pub fn unite_with<B>(&mut self, other: &B) -> Result<(), Error>
    where
        B: AsBoundRef,
    {
        let other = other.as_bound_ref();

        match (self, other) {
            (Bounds::Num(a), BoundsRef::Num(b)) => {
                a.unite_with(&b);
                Ok(())
            }
            (Bounds::Cat(a), BoundsRef::Cat(b)) => {
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

impl BoundsRef<'_> {
    pub fn to_bounds(&self) -> Bounds {
        match self {
            &BoundsRef::Num(n) => n.into(),
            &BoundsRef::Cat(c) => c.clone().into(),
        }
    }
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

pub trait AsBoundRef {
    fn as_bound_ref(&self) -> BoundsRef<'_>;
    fn as_cat(&self) -> Option<&Categories>;
}

impl AsBoundRef for Bounds {
    fn as_bound_ref(&self) -> BoundsRef<'_> {
        match self {
            &Bounds::Num(n) => n.into(),
            &Bounds::Cat(ref c) => c.into(),
        }
    }

    fn as_cat(&self) -> Option<&Categories> {
        match self {
            Bounds::Num(..) => None,
            Bounds::Cat(c) => Some(c),
        }
    }
}

impl AsBoundRef for BoundsRef<'_> {
    fn as_bound_ref(&self) -> BoundsRef<'_> {
        *self
    }

    fn as_cat(&self) -> Option<&Categories> {
        match self {
            BoundsRef::Num(..) => None,
            &BoundsRef::Cat(c) => Some(c),
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

    pub fn log_span(&self, base: f64) -> f64 {
        self.1.log(base) - self.0.log(base)
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

#[cfg(test)]
impl crate::tests::Near for NumBounds {
    fn near_abs(&self, other: &Self, tol: f64) -> bool {
        self.0.near_abs(&other.0, tol) && self.1.near_abs(&other.1, tol)
    }

    fn near_rel(&self, other: &Self, err: f64) -> bool {
        self.0.near_rel(&other.0, err) && self.1.near_rel(&other.1, err)
    }
}

#[cfg(test)]
impl crate::tests::Near for Bounds {
    fn near_abs(&self, other: &Self, tol: f64) -> bool {
        match (self, other) {
            (&Bounds::Num(a), &Bounds::Num(b)) => a.near_abs(&b, tol),
            (Bounds::Cat(a), Bounds::Cat(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (ac, bc) in a.iter().zip(b.iter()) {
                    if ac != bc {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn near_rel(&self, other: &Self, err: f64) -> bool {
        match (self, other) {
            (&Bounds::Num(a), &Bounds::Num(b)) => a.near_rel(&b, err),
            (Bounds::Cat(a), Bounds::Cat(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (ac, bc) in a.iter().zip(b.iter()) {
                    if ac != bc {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
impl crate::tests::Near for BoundsRef<'_> {
    fn near_abs(&self, other: &Self, tol: f64) -> bool {
        match (self, other) {
            (&BoundsRef::Num(a), &BoundsRef::Num(b)) => a.near_abs(&b, tol),
            (&BoundsRef::Cat(a), &BoundsRef::Cat(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (ac, bc) in a.iter().zip(b.iter()) {
                    if ac != bc {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn near_rel(&self, other: &Self, err: f64) -> bool {
        match (self, other) {
            (&BoundsRef::Num(a), &BoundsRef::Num(b)) => a.near_rel(&b, err),
            (&BoundsRef::Cat(a), &BoundsRef::Cat(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (ac, bc) in a.iter().zip(b.iter()) {
                    if ac != bc {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}
