use std::collections::HashMap;


/// Trait for a column of a specific type
pub trait Column {
    fn len(&self) -> usize;

    fn len_some(&self) -> usize;

    fn f64(&self) -> Option<&dyn F64Column> {
        None
    }

    fn i64(&self) -> Option<&dyn I64Column> {
        None
    }

    fn str(&self) -> Option<&dyn StrColumn> {
        None
    }

    fn as_f64_iter(&self) -> Option<Box<dyn Iterator<Item = Option<f64>> + '_>> {
        self.f64().map(|c| c.iter())
    }

    fn as_i64_iter(&self) -> Option<Box<dyn Iterator<Item = Option<i64>> + '_>> {
        self.i64().map(|c| c.iter())
    }
    fn as_str_iter(&self) -> Option<Box<dyn Iterator<Item = Option<&str>> + '_>> {
        self.str().map(|c| c.iter())
    }
}

pub trait F64Column {
    fn len(&self) -> usize;

    fn len_some(&self) -> usize {
        self.iter().filter(|v| v.is_some()).count()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_>;

    fn minmax(&self) -> Option<(f64, f64)> {
        let mut res: Option<(f64, f64)> = None;
        for v in self.iter() {
            match (v, res) {
                (None, _) => continue,
                (Some(v), Some((min, max))) => {
                    res = Some((min.min(v), max.max(v)));
                }
                (Some(v), None) => {
                    res = Some((v, v));
                }
            }
        }
        res
    }
}

pub trait I64Column {
    fn len(&self) -> usize;

    fn len_some(&self) -> usize {
        self.iter().filter(|v| v.is_some()).count()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<i64>> + '_>;

    fn minmax(&self) -> Option<(i64, i64)> {
        let mut res: Option<(i64, i64)> = None;
        for v in self.iter() {
            match (v, res) {
                (None, _) => continue,
                (Some(v), Some((min, max))) => {
                    res = Some((min.min(v), max.max(v)));
                }
                (Some(v), None) => {
                    res = Some((v, v));
                }
            }
        }
        res
    }
}

pub trait StrColumn {
    fn len(&self) -> usize;

    fn len_some(&self) -> usize {
        self.iter().filter(|v| v.is_some()).count()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_>;
}

#[derive(Debug, Clone)]
pub enum VecColumn {
    F64(Vec<f64>),
    I64(Vec<i64>),
    Str(Vec<String>),
}

impl From<Vec<f64>> for VecColumn {
    fn from(v: Vec<f64>) -> Self {
        VecColumn::F64(v)
    }
}

impl From<Vec<i64>> for VecColumn {
    fn from(v: Vec<i64>) -> Self {
        VecColumn::I64(v)
    }
}

impl From<Vec<String>> for VecColumn {
    fn from(v: Vec<String>) -> Self {
        VecColumn::Str(v)
    }
}

impl Column for VecColumn {
    fn len(&self) -> usize {
        match self {
            VecColumn::F64(v) => v.len(),
            VecColumn::I64(v) => v.len(),
            VecColumn::Str(v) => v.len(),
        }
    }

    fn len_some(&self) -> usize {
        match self {
            VecColumn::F64(v) => v.len_some(),
            VecColumn::I64(v) => v.len(),
            VecColumn::Str(v) => v.len_some(),
        }
    }

    fn f64(&self) -> Option<&dyn F64Column> {
        match self {
            VecColumn::F64(v) => Some(v),
            VecColumn::I64(v) => Some(v),
            _ => None,
        }
    }

    fn i64(&self) -> Option<&dyn I64Column> {
        match self {
            VecColumn::I64(v) => Some(v),
            _ => None,
        }
    }

    fn str(&self) -> Option<&dyn StrColumn> {
        match self {
            VecColumn::Str(v) => Some(v),
            _ => None,
        }
    }
}

impl F64Column for Vec<f64> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_finite()).count()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(
            self.as_slice()
                .iter()
                .copied()
                .map(|f| if f.is_finite() { Some(f) } else { None }),
        )
    }
}

impl F64Column for Vec<i64> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(self.as_slice().iter().copied().map(|i| Some(i as f64)))
    }
}

impl I64Column for Vec<i64> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<i64>> + '_> {
        Box::new(self.as_slice().iter().copied().map(Some))
    }
}

impl StrColumn for Vec<String> {
    fn len(&self) -> usize {
        self.len()
    }
    fn iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
        Box::new(self.as_slice().iter().map(|s| Some(s.as_str())))
    }
}

impl StrColumn for Vec<&str> {
    fn len(&self) -> usize {
        self.len()
    }
    fn iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
        Box::new(self.as_slice().iter().map(|s| Some(*s)))
    }
}

/// Trait for a table-like data source
pub trait Source {
    fn column_names(&self) -> Vec<&str>;
    fn column(&self, name: &str) -> Option<&dyn Column>;
    fn len(&self) -> usize;
}

pub struct VecSource {
    columns: HashMap<String, Box<dyn Column>>,
    len: usize,
}

impl VecSource {
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
            len: 0,
        }
    }

    pub fn add_column(&mut self, name: &str, col: Box<dyn Column>) {
        self.len = self.len.max(col.len());
        self.columns.insert(name.to_string(), col);
    }

    pub fn with_column(mut self, name: &str, col: Box<dyn Column>) -> Self {
        self.add_column(name, col); 
        self
    }

    pub fn with_f64_column(mut self, name: &str, col: Vec<f64>) -> Self {
        self.add_column(name, Box::new(VecColumn::F64(col)));
        self
    }
}

impl Source for VecSource {
    fn column_names(&self) -> Vec<&str> {
        self.columns.keys().map(|k| k.as_str()).collect()
    }

    fn column(&self, name: &str) -> Option<&dyn Column> 
    {
        self.columns.get(name).map(|c| &**c)
    }

    fn len(&self) -> usize {
        self.len
    }
}
