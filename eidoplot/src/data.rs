#[cfg(feature = "polars")]
pub mod polars;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Sample<'a> {
    #[default]
    Null,
    Num(f64),
    Cat(&'a str),
}

impl Sample<'_> {
    pub fn is_null(&self) -> bool {
        matches!(self, Sample::Null)
    }

    pub fn as_num(&self) -> Option<f64> {
        match self {
            Sample::Num(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_cat(&self) -> Option<&str> {
        match self {
            Sample::Cat(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_owned(&self) -> OwnedSample {
        match self {
            Sample::Null => OwnedSample::Null,
            Sample::Num(v) => OwnedSample::Num(*v),
            Sample::Cat(v) => OwnedSample::Cat(v.to_string()),
        }
    }
}

impl std::cmp::Eq for Sample<'_> {}

impl From<f64> for Sample<'_> {
    fn from(val: f64) -> Self {
        if val.is_finite() {
            Sample::Num(val)
        } else {
            Sample::Num(val)
        }
    }
}

impl From<Option<f64>> for Sample<'_> {
    fn from(val: Option<f64>) -> Self {
        match val {
            Some(v) => v.into(),
            None => Sample::Null,
        }
    }
}

impl From<i64> for Sample<'_> {
    fn from(val: i64) -> Self {
        Sample::Num(val as f64)
    }
}

impl From<Option<i64>> for Sample<'_> {
    fn from(val: Option<i64>) -> Self {
        match val {
            Some(v) => Sample::Num(v as f64),
            None => Sample::Null,
        }
    }
}

impl<'a> From<&'a str> for Sample<'a> {
    fn from(val: &'a str) -> Self {
        Sample::Cat(val)
    }
}

impl<'a> From<Option<&'a str>> for Sample<'a> {
    fn from(val: Option<&'a str>) -> Self {
        match val {
            Some(val) => Sample::Cat(val),
            None => Sample::Null,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum OwnedSample {
    #[default]
    Null,
    Num(f64),
    Cat(String),
}

impl OwnedSample {
    pub fn is_null(&self) -> bool {
        matches!(self, OwnedSample::Null)
    }

    pub fn as_num(&self) -> Option<f64> {
        match self {
            OwnedSample::Num(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_cat(&self) -> Option<&str> {
        match self {
            OwnedSample::Cat(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_sample(&self) -> Sample<'_> {
        match self {
            OwnedSample::Null => Sample::Null,
            OwnedSample::Num(v) => Sample::Num(*v),
            OwnedSample::Cat(v) => Sample::Cat(v.as_str()),
        }
    }
}

impl std::cmp::Eq for OwnedSample {}

impl From<f64> for OwnedSample {
    fn from(val: f64) -> Self {
        if val.is_finite() {
            OwnedSample::Num(val)
        } else {
            OwnedSample::Num(val)
        }
    }
}

impl From<Option<f64>> for OwnedSample {
    fn from(val: Option<f64>) -> Self {
        match val {
            Some(v) => v.into(),
            None => OwnedSample::Null,
        }
    }
}

impl From<i64> for OwnedSample {
    fn from(val: i64) -> Self {
        OwnedSample::Num(val as f64)
    }
}

impl From<Option<i64>> for OwnedSample {
    fn from(val: Option<i64>) -> Self {
        match val {
            Some(v) => OwnedSample::Num(v as f64),
            None => OwnedSample::Null,
        }
    }
}

impl<'a> From<&'a str> for OwnedSample {
    fn from(val: &'a str) -> Self {
        OwnedSample::Cat(val.to_string())
    }
}

impl From<String> for OwnedSample {
    fn from(val: String) -> Self {
        OwnedSample::Cat(val)
    }
}

impl From<Option<String>> for OwnedSample {
    fn from(val: Option<String>) -> Self {
        match val {
            Some(val) => OwnedSample::Cat(val),
            None => OwnedSample::Null,
        }
    }
}

/// Trait for a column of unspecified type
pub trait Column: std::fmt::Debug {
    fn len(&self) -> usize;

    fn len_some(&self) -> usize;

    fn iter(&self) -> Box<dyn Iterator<Item = Sample<'_>> + '_> {
        if let Some(iter) = self.as_i64_iter() {
            Box::new(iter.map(Sample::from))
        } else if let Some(iter) = self.as_f64_iter() {
            Box::new(iter.map(Sample::from))
        } else {
            Box::new(self.as_str_iter().unwrap().map(Sample::from))
        }
    }

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

pub trait F64Column: std::fmt::Debug {
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

pub trait I64Column: std::fmt::Debug {
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

pub trait StrColumn: std::fmt::Debug {
    fn len(&self) -> usize;

    fn len_some(&self) -> usize {
        self.iter().filter(|v| v.is_some()).count()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_>;
}

/// Trait for a table-like data source
pub trait Source {
    fn column(&self, name: &str) -> Option<&dyn Column>;
}

/// Empty source.
/// Use this if your data is inlined in the IR.
impl Source for () {
    fn column(&self, _name: &str) -> Option<&dyn Column> {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FCol<'a>(pub &'a [f64]);

impl F64Column for FCol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.iter().filter(|v| v.is_finite()).count()
    }
    fn iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(
            self.0
                .iter()
                .copied()
                .map(|f| if f.is_finite() { Some(f) } else { None }),
        )
    }
}

impl Column for FCol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.iter().filter(|v| v.is_finite()).count()
    }
    fn f64(&self) -> Option<&dyn F64Column> {
        Some(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ICol<'a>(pub &'a [i64]);

impl I64Column for ICol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn iter(&self) -> Box<dyn Iterator<Item = Option<i64>> + '_> {
        Box::new(self.0.iter().copied().map(Some))
    }
}

impl F64Column for ICol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(self.0.iter().map(|i| *i as f64).map(Some))
    }
}

impl Column for ICol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn i64(&self) -> Option<&dyn I64Column> {
        Some(self)
    }
    fn f64(&self) -> Option<&dyn F64Column> {
        Some(self)
    }
}

#[derive(Debug)]
pub struct SCol<'a, T>(pub &'a [T]);

impl<T> StrColumn for SCol<'_, T>
where
    T: AsRef<str> + std::fmt::Debug,
{
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
        Box::new(self.0.iter().map(|s| Some(s.as_ref())))
    }
}

impl<T> Column for SCol<'_, T>
where
    T: AsRef<str> + std::fmt::Debug,
{
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn str(&self) -> Option<&dyn StrColumn> {
        Some(self)
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

impl Column for Vec<f64> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_finite()).count()
    }
    fn f64(&self) -> Option<&dyn F64Column> {
        Some(self)
    }
}

impl F64Column for Vec<Option<i64>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(self.as_slice().iter().copied().map(|v| v.map(|v| v as f64)))
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
        Box::new(self.as_slice().iter().copied().map(|v| Some(v as f64)))
    }
}

impl I64Column for Vec<Option<i64>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<i64>> + '_> {
        Box::new(self.as_slice().iter().copied())
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

impl StrColumn for Vec<Option<String>> {
    fn len(&self) -> usize {
        self.len()
    }
    fn iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
        Box::new(self.as_slice().iter().map(|s| s.as_deref()))
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

impl StrColumn for Vec<Option<&str>> {
    fn len(&self) -> usize {
        self.len()
    }
    fn iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
        Box::new(self.as_slice().iter().map(|s| *s))
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

/// Simple collection of named columns, owning the data
#[derive(Debug)]
pub struct NamedOwnedColumns {
    names: Vec<String>,
    columns: Vec<Box<dyn Column>>,
}

impl NamedOwnedColumns {
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            columns: Vec::new(),
        }
    }

    pub fn add_column(&mut self, name: &str, col: Box<dyn Column>) {
        let position = self.names.as_slice().iter().position(|n| n == name);
        if let Some(pos) = position {
            self.columns[pos] = col;
            return;
        }
        self.names.push(name.to_string());
        self.columns.push(col);
    }
}

impl Source for NamedOwnedColumns {
    fn column(&self, name: &str) -> Option<&dyn Column> {
        let Some(idx) = self.names.as_slice().iter().position(|k| k == name) else {
            return None;
        };
        self.columns.get(idx).map(|c| c.as_ref() as &dyn Column)
    }
}

/// Simple collection of named columns, referencing external data
#[derive(Debug)]
pub struct NamedColumns<'a> {
    names: Vec<String>,
    columns: Vec<&'a dyn Column>,
}

impl<'a> NamedColumns<'a> {
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            columns: Vec::new(),
        }
    }

    pub fn add_column(&mut self, name: &str, col: &'a dyn Column) {
        let position = self.names.as_slice().iter().position(|n| n == name);
        if let Some(pos) = position {
            self.columns[pos] = col;
            return;
        }
        self.names.push(name.to_string());
        self.columns.push(col);
    }
}

impl<'a> Source for NamedColumns<'a> {
    fn column(&self, name: &str) -> Option<&dyn Column> {
        let Some(idx) = self.names.as_slice().iter().position(|k| k == name) else {
            return None;
        };
        self.columns.get(idx).map(|c| *c as &dyn Column)
    }
}

// Simple vector base implementation
#[derive(Debug, Clone)]
pub enum VecColumn {
    F64(Vec<f64>),
    I64(Vec<Option<i64>>),
    Str(Vec<Option<String>>),
}

impl From<Vec<f64>> for VecColumn {
    fn from(v: Vec<f64>) -> Self {
        VecColumn::F64(v)
    }
}

impl From<Vec<Option<i64>>> for VecColumn {
    fn from(v: Vec<Option<i64>>) -> Self {
        VecColumn::I64(v)
    }
}

impl From<Vec<Option<String>>> for VecColumn {
    fn from(v: Vec<Option<String>>) -> Self {
        VecColumn::Str(v)
    }
}

impl From<Vec<i64>> for VecColumn {
    fn from(v: Vec<i64>) -> Self {
        let v: Vec<Option<i64>> = v.into_iter().map(Some).collect();
        VecColumn::I64(v)
    }
}

impl From<Vec<String>> for VecColumn {
    fn from(v: Vec<String>) -> Self {
        let v: Vec<Option<String>> = v.into_iter().map(Some).collect();
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
            VecColumn::F64(v) => <dyn F64Column>::len_some(v),
            VecColumn::I64(v) => <dyn I64Column>::len_some(v),
            VecColumn::Str(v) => v.len_some(),
        }
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Sample<'_>> + '_> {
        match self {
            VecColumn::F64(v) => Box::new(v.as_slice().iter().map(|v| (*v).into())),
            VecColumn::I64(v) => Box::new(v.as_slice().iter().map(|v| (*v).into())),
            VecColumn::Str(v) => Box::new(v.iter().map(|v| v.into())),
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

/// Simple table source backed by vectors
pub struct TableSource {
    heads: Vec<String>,
    columns: Vec<VecColumn>,
    len: usize,
}

impl TableSource {
    pub fn new() -> Self {
        Self {
            heads: Vec::new(),
            columns: Vec::new(),
            len: 0,
        }
    }

    pub fn add_column(&mut self, name: &str, col: VecColumn) {
        self.len = self.len.max(col.len());
        self.heads.push(name.to_string());
        self.columns.push(col);
        for col in &mut self.columns {
            while col.len() < self.len {
                match col {
                    VecColumn::F64(vec) => vec.push(f64::NAN),
                    VecColumn::I64(vec) => vec.push(None),
                    VecColumn::Str(vec) => vec.push(None),
                }
            }
        }
    }

    pub fn with_column(mut self, name: &str, col: VecColumn) -> Self {
        self.add_column(name, col);
        self
    }

    pub fn with_f64_column(mut self, name: &str, col: Vec<f64>) -> Self {
        self.add_column(name, VecColumn::F64(col));
        self
    }

    pub fn with_i64_column(mut self, name: &str, col: Vec<Option<i64>>) -> Self {
        self.add_column(name, VecColumn::I64(col));
        self
    }

    pub fn with_str_column(mut self, name: &str, col: Vec<Option<String>>) -> Self {
        self.add_column(name, VecColumn::Str(col));
        self
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Source for TableSource {
    fn column(&self, name: &str) -> Option<&dyn Column> {
        let Some(idx) = self.heads.as_slice().iter().position(|k| k == name) else {
            return None;
        };
        self.columns.get(idx).map(|c| c as &dyn Column)
    }
}

/// Custom Debug implementation to pretty-print the table
impl std::fmt::Debug for TableSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rows = self.len();
        let cols = self.heads.len();

        // Determine which columns to show
        let (col_indices, show_ellipsis) = if cols > 8 {
            let mut idxs = (0..4).collect::<Vec<_>>();
            idxs.extend((cols - 4)..cols);
            (idxs, true)
        } else {
            ((0..cols).collect::<Vec<_>>(), false)
        };

        // Helper to get cell as string
        fn cell_string(col: &VecColumn, row: usize) -> String {
            match col {
                VecColumn::F64(v) => {
                    let val = v.get(row).copied();
                    match val {
                        Some(x) if x.is_finite() => format!("{:.6}", x),
                        _ => "(null)".to_string(),
                    }
                }
                VecColumn::I64(v) => match v.get(row).copied().flatten() {
                    Some(x) => format!("{}", x),
                    None => "(null)".to_string(),
                },
                VecColumn::Str(v) => match v.get(row) {
                    Some(Some(s)) => s.clone(),
                    _ => "(null)".to_string(),
                },
            }
        }

        // Compute max width for each shown column (header, and up to 5+5 rows)
        let mut col_widths: Vec<usize> = col_indices.iter().map(|&i| self.heads[i].len()).collect();

        let row_indices: Vec<usize> = if rows <= 10 {
            (0..rows).collect()
        } else {
            (0..5).chain((rows - 5)..rows).collect()
        };

        for (col_pos, &col_idx) in col_indices.iter().enumerate() {
            // Check header
            col_widths[col_pos] = col_widths[col_pos].max(self.heads[col_idx].len());
            // Check cell values
            for &row in &row_indices {
                let cell = cell_string(&self.columns[col_idx], row);
                col_widths[col_pos] = col_widths[col_pos].max(cell.len());
            }
        }
        let ellipsis_width = 6; // width for "..."

        // Print header
        writeln!(f, "VecSource: {} rows x {} columns", rows, cols)?;
        for (col_pos, &i) in col_indices.iter().enumerate() {
            write!(
                f,
                "| {:^width$} ",
                &self.heads[i],
                width = col_widths[col_pos]
            )?;
        }
        if show_ellipsis {
            write!(f, "| {:^width$} ", "...", width = ellipsis_width)?;
        }
        writeln!(f, "|")?;

        // Print separator
        for (col_pos, _) in col_indices.iter().enumerate() {
            write!(f, "|{:=^width$}", "", width = col_widths[col_pos] + 2)?;
        }
        if show_ellipsis {
            write!(f, "|{:=^width$}", "", width = ellipsis_width + 2)?;
        }
        writeln!(f, "|")?;

        // Helper to print a row
        let print_row = |f: &mut std::fmt::Formatter<'_>, row: usize| -> std::fmt::Result {
            for (col_pos, &i) in col_indices.iter().enumerate() {
                let cell = cell_string(&self.columns[i], row);
                write!(f, "| {:>width$} ", cell, width = col_widths[col_pos])?;
            }
            if show_ellipsis {
                write!(f, "| {:^width$} ", "...", width = ellipsis_width)?;
            }
            writeln!(f, "|")
        };

        if rows <= 10 {
            for row in 0..rows {
                print_row(f, row)?;
            }
        } else {
            // Print first 5
            for row in 0..5 {
                print_row(f, row)?;
            }
            // Ellipsis for rows
            for (col_pos, _) in col_indices.iter().enumerate() {
                write!(f, "| {:^width$} ", "...", width = col_widths[col_pos])?;
            }
            if show_ellipsis {
                write!(f, "| {:^width$} ", "...", width = ellipsis_width)?;
            }
            writeln!(f, "|")?;
            // Print last 5
            for row in (rows - 5)..rows {
                print_row(f, row)?;
            }
        }
        Ok(())
    }
}
