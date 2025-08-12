use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    NoSuchColumn(Vec<String>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoSuchColumn(names) => {
                if names.len() == 1 {
                    write!(f, "no such column: {}", names[0])
                } else if names.len() > 1 {
                    write!(f, "no such columns: {}", names.join(", "))
                } else {
                    unreachable!("names.len() == 0")
                }
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    F64,
    I64,
    Str,
}

pub trait AsType {
    fn as_type(&self) -> Type;
}

impl AsType for f64 {
    fn as_type(&self) -> Type {
        Type::F64
    }
}

impl AsType for i64 {
    fn as_type(&self) -> Type {
        Type::I64
    }
}

impl AsType for String {
    fn as_type(&self) -> Type {
        Type::Str
    }
}

pub trait Column {
    fn len(&self) -> usize;
    fn typ(&self) -> Type;
}

pub struct ArrayCol<T> {
    data: Vec<T>,
}

impl<T> ArrayCol<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }
}

impl<T> AsRef<[T]> for ArrayCol<T> {
    fn as_ref(&self) -> &[T] {
        &self.data
    }
}

impl<T> Column for ArrayCol<T>
where
    T: AsType,
{
    fn len(&self) -> usize {
        self.data.len()
    }

    fn typ(&self) -> Type {
        self.data[0].as_type()
    }
}

#[derive(Clone)]
pub enum VecCol {
    F64(Vec<f64>),
    I64(Vec<i64>),
    Str(Vec<String>),
}

impl fmt::Debug for VecCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VecCol::F64(x) => {
                write!(f, "Column::F64(")?;
                debug_col_vec(f, x)?;
                write!(f, ")")?;
                Ok(())
            }
            VecCol::I64(x) => {
                write!(f, "Column::I64(")?;
                debug_col_vec(f, x)?;
                write!(f, ")")?;
                Ok(())
            }
            VecCol::Str(x) => {
                write!(f, "Column::Str(")?;
                write!(f, "[{} elements]", x.len())?;
                write!(f, ")")?;
                Ok(())
            }
        }
    }
}

fn debug_col_vec<T>(f: &mut fmt::Formatter<'_>, x: &Vec<T>) -> fmt::Result
where
    T: fmt::Display + PartialOrd + Copy,
{
    if x.is_empty() {
        write!(f, "[]")?;
    } else {
        let (min, max) = get_minmax(x).unwrap();
        write!(f, "[{} elements, min={}, max={}]", x.len(), min, max)?;
    }
    Ok(())
}

fn get_minmax<T>(x: &[T]) -> Option<(T, T)>
where
    T: PartialOrd + Copy,
{
    if x.is_empty() {
        None
    } else {
        let mut min = x[0];
        let mut max = x[0];
        for v in x[1..].iter().copied() {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        Some((min, max))
    }
}

impl From<Vec<f64>> for VecCol {
    fn from(value: Vec<f64>) -> Self {
        Self::F64(value)
    }
}

impl From<Vec<i64>> for VecCol {
    fn from(value: Vec<i64>) -> Self {
        Self::I64(value)
    }
}

impl From<Vec<String>> for VecCol {
    fn from(value: Vec<String>) -> Self {
        Self::Str(value)
    }
}

pub enum ColumnRef<'a> {
    F64(&'a [f64]),
    I64(&'a [i64]),
    Str(&'a [String]),
}

pub trait Source {
    fn col<'a>(&'a self, name: &str) -> Option<&'a [f64]>;
}

pub struct ColumnMapSource(HashMap<String, VecCol>);

impl ColumnMapSource {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn with_col<C>(mut self, name: String, col: C) -> Self
    where
        C: Into<VecCol>,
    {
        self.add_col(name, col);
        self
    }

    pub fn add_col<C>(&mut self, name: String, col: C)
    where
        C: Into<VecCol>,
    {
        self.0.insert(name, col.into());
    }
}

pub trait SourceIterator {
    type Item;
    fn iter_src<'a, S>(
        &'a self,
        source: &'a S,
    ) -> Result<impl Iterator<Item = Self::Item> + 'a, Error>
    where
        S: Source;
}

#[derive(Debug, Clone)]
pub enum X {
    Inline(Vec<f64>),
    Src(String),
}

impl SourceIterator for X {
    type Item = f64;

    fn iter_src<'a, S>(&'a self, source: &'a S) -> Result<impl Iterator<Item = f64> + 'a, Error>
    where
        S: Source,
    {
        match self {
            X::Inline(x) => Ok(x.iter().copied()),
            X::Src(x_col) => source
                .col(x_col)
                .ok_or_else(|| Error::NoSuchColumn(vec![x_col.clone()]))
                .map(|x| x.iter().copied()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Xy {
    Inline(Vec<f64>, Vec<f64>),
    Src(String, String),
}

impl SourceIterator for Xy {
    type Item = (f64, f64);

    fn iter_src<'a, S>(
        &'a self,
        source: &'a S,
    ) -> Result<impl Iterator<Item = (f64, f64)> + 'a, Error>
    where
        S: Source,
    {
        match self {
            Xy::Inline(x, y) => Ok(x.iter().copied().zip(y.iter().copied())),
            Xy::Src(x_col, y_col) => match (source.col(&x_col), source.col(&y_col)) {
                (Some(x), Some(y)) => Ok(x.iter().copied().zip(y.iter().copied())),
                (x, y) => {
                    let mut cols = vec![];
                    if x.is_none() {
                        cols.push(x_col.clone());
                    }
                    if y.is_none() {
                        cols.push(y_col.clone());
                    }
                    Err(Error::NoSuchColumn(cols))
                }
            },
        }
    }
}

impl Source for () {
    fn col(&self, _name: &str) -> Option<&[f64]> {
        None
    }
}

pub struct VecMapSource(HashMap<String, Vec<f64>>);

impl VecMapSource {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn with_col(mut self, name: String, col: Vec<f64>) -> Self {
        self.add_col(name, col);
        self
    }

    pub fn add_col(&mut self, name: String, col: Vec<f64>) {
        self.0.insert(name, col);
    }
}

impl Source for VecMapSource {
    fn col(&self, name: &str) -> Option<&[f64]> {
        self.0.get(name).map(Vec::as_slice)
    }
}

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
        Self(value.0.min(value.1), value.0.max(value.1))
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
