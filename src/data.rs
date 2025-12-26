//! Data source abstractions and implementations.
//!
//! Data provided to series must implement the [`Column`] trait.
//! The columns are grouped in a struct implementing the [`Source`] trait.
//! This data source is provided to the plotting functions.
//!
//! Several column implementations are provided in this module, for common data types
//! like `Vec<f64>`, `Vec<i64>`, `Vec<String>`, `Vec<DateTime>`, etc.
use core::fmt;
use std::sync::Arc;

use crate::time::{DateTime, TimeDelta};

#[cfg(feature = "data-csv")]
mod csv;
#[cfg(feature = "data-csv")]
pub use csv::{CsvColSpec, CsvParseError, CsvParser};

#[cfg(feature = "data-polars")]
pub mod polars;

/// Sample value enum. Useful when the type is not known at compile time.
/// You will typically not used a `Vec<Sample>`. Rather a [`VecColumn`] or similar
/// that iterate over samples in a more efficient way.
///
/// This type borrows string data for categorical samples. See also [`OwnedSample`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Sample<'a> {
    /// Null value
    #[default]
    Null,
    /// Numeric value
    Num(f64),
    /// Categorical value
    Cat(&'a str),
    /// Time value
    Time(DateTime),
    /// Time delta value
    TimeDelta(TimeDelta),
}

impl Sample<'_> {
    /// Check if the sample is null
    pub fn is_null(&self) -> bool {
        matches!(self, Sample::Null)
    }

    /// Get the sample as a numeric value, if possible
    pub fn as_num(&self) -> Option<f64> {
        match self {
            Sample::Num(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the sample as a categorical value, if possible
    pub fn as_cat(&self) -> Option<&str> {
        match self {
            Sample::Cat(v) => Some(v),
            _ => None,
        }
    }

    /// Get the sample as a time value, if possible
    pub fn as_time(&self) -> Option<DateTime> {
        match self {
            Sample::Time(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the sample as a time delta value, if possible
    pub fn as_time_delta(&self) -> Option<TimeDelta> {
        match self {
            Sample::TimeDelta(v) => Some(*v),
            _ => None,
        }
    }

    /// Convert the sample to an owned sample
    pub fn to_owned(&self) -> OwnedSample {
        match self {
            Sample::Null => OwnedSample::Null,
            Sample::Num(v) => OwnedSample::Num(*v),
            Sample::Cat(v) => OwnedSample::Cat(v.to_string()),
            Sample::Time(v) => OwnedSample::Time(*v),
            Sample::TimeDelta(v) => OwnedSample::TimeDelta(*v),
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

impl From<DateTime> for Sample<'_> {
    fn from(val: DateTime) -> Self {
        Sample::Time(val)
    }
}

impl From<Option<DateTime>> for Sample<'_> {
    fn from(val: Option<DateTime>) -> Self {
        match val {
            Some(v) => Sample::Time(v),
            None => Sample::Null,
        }
    }
}

impl From<TimeDelta> for Sample<'_> {
    fn from(val: TimeDelta) -> Self {
        Sample::TimeDelta(val)
    }
}

impl From<Option<TimeDelta>> for Sample<'_> {
    fn from(val: Option<TimeDelta>) -> Self {
        match val {
            Some(v) => Sample::TimeDelta(v),
            None => Sample::Null,
        }
    }
}

/// Owned version of [`Sample`].
#[derive(Debug, Clone, Default, PartialEq)]
pub enum OwnedSample {
    /// Null value
    #[default]
    Null,
    /// Numeric value
    Num(f64),
    /// Categorical value
    Cat(String),
    /// Time value
    Time(DateTime),
    /// Time delta value
    TimeDelta(TimeDelta),
}

impl OwnedSample {
    /// Check if the sample is null
    pub fn is_null(&self) -> bool {
        matches!(self, OwnedSample::Null)
    }

    /// Get the sample as a numeric value, if possible
    pub fn as_num(&self) -> Option<f64> {
        match self {
            OwnedSample::Num(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the sample as a categorical value, if possible
    pub fn as_cat(&self) -> Option<&str> {
        match self {
            OwnedSample::Cat(v) => Some(v),
            _ => None,
        }
    }

    /// Get the sample as a time value, if possible
    pub fn as_time(&self) -> Option<DateTime> {
        match self {
            OwnedSample::Time(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the sample as a time delta value, if possible
    pub fn as_time_delta(&self) -> Option<TimeDelta> {
        match self {
            OwnedSample::TimeDelta(v) => Some(*v),
            _ => None,
        }
    }

    /// Convert the owned sample to a borrowed sample
    pub fn as_sample(&self) -> Sample<'_> {
        match self {
            OwnedSample::Null => Sample::Null,
            OwnedSample::Num(v) => Sample::Num(*v),
            OwnedSample::Cat(v) => Sample::Cat(v.as_str()),
            OwnedSample::Time(v) => Sample::Time(*v),
            OwnedSample::TimeDelta(v) => Sample::TimeDelta(*v),
        }
    }
}

impl<'a> From<Sample<'a>> for OwnedSample {
    fn from(sample: Sample<'a>) -> Self {
        sample.to_owned()
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

impl From<DateTime> for OwnedSample {
    fn from(val: DateTime) -> Self {
        OwnedSample::Time(val)
    }
}

impl From<Option<DateTime>> for OwnedSample {
    fn from(val: Option<DateTime>) -> Self {
        match val {
            Some(v) => OwnedSample::Time(v),
            None => OwnedSample::Null,
        }
    }
}

impl From<TimeDelta> for OwnedSample {
    fn from(val: TimeDelta) -> Self {
        OwnedSample::TimeDelta(val)
    }
}

impl From<Option<TimeDelta>> for OwnedSample {
    fn from(val: Option<TimeDelta>) -> Self {
        match val {
            Some(v) => OwnedSample::TimeDelta(v),
            None => OwnedSample::Null,
        }
    }
}

/// Trait for a column of unspecified type.
/// This is the base trait for data given to series.
pub trait Column: std::fmt::Debug {
    /// Get the length of the column
    fn len(&self) -> usize;

    /// Get the number of non-null values in the column
    fn len_some(&self) -> usize;

    /// Get an iterator over the samples in the column
    fn sample_iter(&self) -> Box<dyn Iterator<Item = Sample<'_>> + '_> {
        if let Some(iter) = self.as_time_iter() {
            Box::new(iter.map(Sample::from))
        } else if let Some(iter) = self.as_time_delta_iter() {
            Box::new(iter.map(Sample::from))
        } else if let Some(iter) = self.as_i64_iter() {
            Box::new(iter.map(Sample::from))
        } else if let Some(iter) = self.as_f64_iter() {
            Box::new(iter.map(Sample::from))
        } else {
            Box::new(self.as_str_iter().unwrap().map(Sample::from))
        }
    }

    /// Get a copy of the column as a boxed trait object
    /// This should be implemented if a copy can be made in an efficient way.
    /// It is not mandatory that the same type is returned.
    ///
    /// Panics if none of the f64, i64, str, time or time_delta methods return Some.
    fn boxed_copy(&self) -> Box<dyn Column> {
        if let Some(col) = self.f64() {
            let mut vec = Vec::with_capacity(col.len());
            for v in col.f64_iter() {
                vec.push(v.unwrap_or(f64::NAN));
            }
            Box::new(vec)
        } else if let Some(col) = self.i64() {
            Box::new(col.i64_iter().collect::<Vec<_>>())
        } else if let Some(col) = self.str() {
            Box::new(
                col.str_iter()
                    .map(|s| s.map(|s| s.to_string()))
                    .collect::<Vec<_>>(),
            )
        } else if let Some(col) = self.time() {
            Box::new(col.time_iter().collect::<Vec<_>>())
        } else if let Some(col) = self.time_delta() {
            Box::new(col.time_delta_iter().collect::<Vec<_>>())
        } else {
            panic!("Cannot box copy column: no known type");
        }
    }

    /// Get the column as a f64 column, if possible
    fn f64(&self) -> Option<&dyn F64Column> {
        None
    }

    /// Get the column as an i64 column, if possible
    fn i64(&self) -> Option<&dyn I64Column> {
        None
    }

    /// Get the column as a str column, if possible
    fn str(&self) -> Option<&dyn StrColumn> {
        None
    }

    /// Get the column as a time column, if possible
    fn time(&self) -> Option<&dyn TimeColumn> {
        None
    }

    /// Get the column as a time delta column, if possible
    fn time_delta(&self) -> Option<&dyn TimeDeltaColumn> {
        None
    }

    /// Helper to get f64 iterator
    fn as_f64_iter(&self) -> Option<Box<dyn Iterator<Item = Option<f64>> + '_>> {
        self.f64().map(|c| c.f64_iter())
    }

    /// Helper to get i64 iterator
    fn as_i64_iter(&self) -> Option<Box<dyn Iterator<Item = Option<i64>> + '_>> {
        self.i64().map(|c| c.i64_iter())
    }

    /// Helper to get str iterator
    fn as_str_iter(&self) -> Option<Box<dyn Iterator<Item = Option<&str>> + '_>> {
        self.str().map(|c| c.str_iter())
    }

    /// Helper to get time iterator
    fn as_time_iter(&self) -> Option<Box<dyn Iterator<Item = Option<DateTime>> + '_>> {
        self.time().map(|c| c.time_iter())
    }

    /// Helper to get time delta iterator
    fn as_time_delta_iter(&self) -> Option<Box<dyn Iterator<Item = Option<TimeDelta>> + '_>> {
        self.time_delta().map(|c| c.time_delta_iter())
    }
}

/// Trait for a column of f64 values
pub trait F64Column: std::fmt::Debug {
    /// Get the length of the column
    fn len(&self) -> usize;

    /// Get the number of non-null values in the column
    /// That is, the number of values that are not NaN
    fn len_some(&self) -> usize {
        self.f64_iter().filter(|v| v.is_some()).count()
    }

    /// Get an iterator over the f64 values in the column
    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_>;

    /// Get the min and max values in the column.
    /// Returns None if there are only null values.
    fn minmax(&self) -> Option<(f64, f64)> {
        let mut res: Option<(f64, f64)> = None;
        for v in self.f64_iter() {
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

/// Trait for a column of i64 values
pub trait I64Column: std::fmt::Debug {
    /// Get the length of the column
    fn len(&self) -> usize;

    /// Get the number of non-null values in the column
    fn len_some(&self) -> usize {
        self.i64_iter().filter(|v| v.is_some()).count()
    }

    /// Get an iterator over the i64 values in the column
    fn i64_iter(&self) -> Box<dyn Iterator<Item = Option<i64>> + '_>;

    /// Get the min and max values in the column.
    /// Returns None if there are only null values.
    fn minmax(&self) -> Option<(i64, i64)> {
        let mut res: Option<(i64, i64)> = None;
        for v in self.i64_iter() {
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

/// Trait for a column of string values
pub trait StrColumn: std::fmt::Debug {
    /// Get the length of the column
    fn len(&self) -> usize;

    /// Get the number of non-null values in the column
    fn len_some(&self) -> usize {
        self.str_iter().filter(|v| v.is_some()).count()
    }

    /// Get an iterator over the string values in the column
    fn str_iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_>;
}

/// Trait for a column of time values
pub trait TimeColumn: std::fmt::Debug {
    /// Get the length of the column
    fn len(&self) -> usize;

    /// Get the number of non-null values in the column
    fn len_some(&self) -> usize {
        self.time_iter().filter(|v| v.is_some()).count()
    }

    /// Get an iterator over the time values in the column
    fn time_iter(&self) -> Box<dyn Iterator<Item = Option<DateTime>> + '_>;

    /// Get the min and max values in the column.
    /// Returns None if there are only null values.
    fn minmax(&self) -> Option<(DateTime, DateTime)> {
        let mut res: Option<(DateTime, DateTime)> = None;
        for v in self.time_iter() {
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

/// Trait for a column of time delta values
pub trait TimeDeltaColumn: std::fmt::Debug {
    /// Get the length of the column
    fn len(&self) -> usize;

    /// Get the number of non-null values in the column
    fn len_some(&self) -> usize {
        self.time_delta_iter().filter(|v| v.is_some()).count()
    }

    /// Get an iterator over the time delta values in the column
    fn time_delta_iter(&self) -> Box<dyn Iterator<Item = Option<TimeDelta>> + '_>;

    /// Get the min and max values in the column.
    /// Returns None if there are only null values.
    fn minmax(&self) -> Option<(TimeDelta, TimeDelta)> {
        let mut res: Option<(TimeDelta, TimeDelta)> = None;
        for v in self.time_delta_iter() {
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

/// Trait for a data source.
/// This groups multiple columns together by name and provides
/// data access to plotting functions.
pub trait Source: fmt::Debug {
    /// Get the names of the columns in the source
    fn names(&self) -> Vec<&str>;

    /// Get a column by name
    fn column(&self, name: &str) -> Option<&dyn Column>;

    /// Get a copy of this source as a Arc trait object
    /// This should be implemented only if the source is clonable in an efficient way
    /// By default, this method will attempt to copy each column individually.
    fn copy(&self) -> Arc<dyn Source> {
        let names = self.names();
        let mut new_source = NamedOwnedColumns::new();
        for name in names {
            if let Some(col) = self.column(name) {
                new_source.add_column(name, col.boxed_copy());
            }
        }
        Arc::new(new_source)
    }
}

/// Empty source.
/// Use this if your data is inlined in the IR.
impl Source for () {
    fn names(&self) -> Vec<&str> {
        Vec::new()
    }

    fn column(&self, _name: &str) -> Option<&dyn Column> {
        None
    }

    fn copy(&self) -> Arc<dyn Source> {
        Arc::new(())
    }
}

impl Source for Arc<dyn Source> {
    fn names(&self) -> Vec<&str> {
        self.as_ref().names()
    }

    fn column(&self, name: &str) -> Option<&dyn Column> {
        self.as_ref().column(name)
    }

    fn copy(&self) -> Arc<dyn Source> {
        self.as_ref().copy()
    }
}

/// Column implementation for a slice of f64 values
#[derive(Debug, Clone, Copy)]
pub struct FCol<'a>(pub &'a [f64]);

impl F64Column for FCol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.iter().filter(|v| v.is_finite()).count()
    }
    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
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
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.0.to_vec())
    }
}

/// Column implementation for a slice of i64 values
#[derive(Debug, Clone, Copy)]
pub struct ICol<'a>(pub &'a [i64]);

impl I64Column for ICol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn i64_iter(&self) -> Box<dyn Iterator<Item = Option<i64>> + '_> {
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
    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
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
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.0.to_vec())
    }
}

/// Column implementation for a slice of string-like values
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
    fn str_iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
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
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(
            self.0
                .iter()
                .map(|s| s.as_ref().to_string())
                .collect::<Vec<_>>(),
        )
    }
}

/// Column implementation for a slice of DateTime values
#[derive(Debug)]
pub struct TCol<'a>(pub &'a [DateTime]);

impl TimeColumn for TCol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn time_iter(&self) -> Box<dyn Iterator<Item = Option<DateTime>> + '_> {
        Box::new(self.0.iter().copied().map(Some))
    }
}

impl F64Column for TCol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(self.0.iter().map(|dt| dt.timestamp()).map(Some))
    }
}

impl Column for TCol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn f64(&self) -> Option<&dyn F64Column> {
        Some(self)
    }
    fn time(&self) -> Option<&dyn TimeColumn> {
        Some(self)
    }
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.0.to_vec())
    }
}

/// Column implementation for a slice of TimeDelta values
#[derive(Debug)]
pub struct TdCol<'a>(pub &'a [TimeDelta]);

impl TimeDeltaColumn for TdCol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn time_delta_iter(&self) -> Box<dyn Iterator<Item = Option<TimeDelta>> + '_> {
        Box::new(self.0.iter().copied().map(Some))
    }
}

impl F64Column for TdCol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(self.0.iter().map(|dt| dt.seconds()).map(Some))
    }
}

impl Column for TdCol<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn len_some(&self) -> usize {
        self.0.len()
    }
    fn f64(&self) -> Option<&dyn F64Column> {
        Some(self)
    }
    fn time_delta(&self) -> Option<&dyn TimeDeltaColumn> {
        Some(self)
    }
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.0.to_vec())
    }
}

impl F64Column for Vec<f64> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_finite()).count()
    }

    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
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
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.clone())
    }
}

impl F64Column for Vec<f32> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_finite()).count()
    }

    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(
            self.as_slice()
                .iter()
                .copied()
                .map(|f| if f.is_finite() { Some(f as f64) } else { None }),
        )
    }
}

impl Column for Vec<f32> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_finite()).count()
    }
    fn f64(&self) -> Option<&dyn F64Column> {
        Some(self)
    }
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.clone())
    }
}

impl F64Column for Vec<Option<i64>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_some()).count()
    }

    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(self.as_slice().iter().copied().map(|v| v.map(|v| v as f64)))
    }
}

impl I64Column for Vec<Option<i64>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_some()).count()
    }

    fn i64_iter(&self) -> Box<dyn Iterator<Item = Option<i64>> + '_> {
        Box::new(self.as_slice().iter().copied())
    }
}

impl Column for Vec<Option<i64>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_some()).count()
    }

    fn i64(&self) -> Option<&dyn I64Column> {
        Some(self)
    }

    fn f64(&self) -> Option<&dyn F64Column> {
        Some(self)
    }

    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.clone())
    }
}

impl F64Column for Vec<i64> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.len()
    }

    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(self.as_slice().iter().copied().map(|v| Some(v as f64)))
    }
}

impl I64Column for Vec<i64> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.len()
    }

    fn i64_iter(&self) -> Box<dyn Iterator<Item = Option<i64>> + '_> {
        Box::new(self.as_slice().iter().copied().map(Some))
    }
}

impl Column for Vec<i64> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.len()
    }

    fn i64(&self) -> Option<&dyn I64Column> {
        Some(self)
    }

    fn f64(&self) -> Option<&dyn F64Column> {
        Some(self)
    }

    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.clone())
    }
}

impl StrColumn for Vec<Option<String>> {
    fn len(&self) -> usize {
        self.len()
    }
    fn str_iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
        Box::new(self.as_slice().iter().map(|s| s.as_deref()))
    }
}

impl Column for Vec<Option<String>> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_some()).count()
    }
    fn str(&self) -> Option<&dyn StrColumn> {
        Some(self)
    }
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.clone())
    }
}

impl StrColumn for Vec<String> {
    fn len(&self) -> usize {
        self.len()
    }
    fn str_iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
        Box::new(self.as_slice().iter().map(|s| Some(s.as_str())))
    }
}

impl Column for Vec<String> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.len()
    }
    fn str(&self) -> Option<&dyn StrColumn> {
        Some(self)
    }
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.clone())
    }
}

impl StrColumn for Vec<Option<&str>> {
    fn len(&self) -> usize {
        self.len()
    }
    fn str_iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
        Box::new(self.as_slice().iter().map(|s| *s))
    }
}

impl Column for Vec<Option<&str>> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_some()).count()
    }
    fn str(&self) -> Option<&dyn StrColumn> {
        Some(self)
    }
}

impl StrColumn for Vec<&str> {
    fn len(&self) -> usize {
        self.len()
    }
    fn str_iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
        Box::new(self.as_slice().iter().map(|s| Some(*s)))
    }
}

impl Column for Vec<&str> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.len()
    }
    fn str(&self) -> Option<&dyn StrColumn> {
        Some(self)
    }
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }
}

impl TimeColumn for Vec<DateTime> {
    fn len(&self) -> usize {
        self.len()
    }

    fn time_iter(&self) -> Box<dyn Iterator<Item = Option<DateTime>> + '_> {
        Box::new(self.as_slice().iter().map(|v| Some(*v)))
    }
}

impl F64Column for Vec<DateTime> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.len()
    }
    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(self.as_slice().iter().map(|v| Some(v.timestamp())))
    }
}

impl Column for Vec<DateTime> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.len()
    }
    fn time(&self) -> Option<&dyn TimeColumn> {
        Some(self)
    }
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.clone())
    }
}

impl TimeColumn for Vec<Option<DateTime>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn time_iter(&self) -> Box<dyn Iterator<Item = Option<DateTime>> + '_> {
        Box::new(self.as_slice().iter().copied())
    }
}

impl F64Column for Vec<Option<DateTime>> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_some()).count()
    }
    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(
            self.as_slice()
                .iter()
                .copied()
                .map(|v| v.map(|v| v.timestamp())),
        )
    }
}

impl Column for Vec<Option<DateTime>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_some()).count()
    }

    fn time(&self) -> Option<&dyn TimeColumn> {
        Some(self)
    }

    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.clone())
    }
}

impl TimeDeltaColumn for Vec<TimeDelta> {
    fn len(&self) -> usize {
        self.len()
    }

    fn time_delta_iter(&self) -> Box<dyn Iterator<Item = Option<TimeDelta>> + '_> {
        Box::new(self.as_slice().iter().map(|v| Some(*v)))
    }
}

impl F64Column for Vec<TimeDelta> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.len()
    }
    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(self.as_slice().iter().map(|v| Some(v.seconds())))
    }
}

impl Column for Vec<TimeDelta> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.len()
    }
    fn time_delta(&self) -> Option<&dyn TimeDeltaColumn> {
        Some(self)
    }
    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.clone())
    }
}

impl TimeDeltaColumn for Vec<Option<TimeDelta>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn time_delta_iter(&self) -> Box<dyn Iterator<Item = Option<TimeDelta>> + '_> {
        Box::new(self.as_slice().iter().copied())
    }
}

impl F64Column for Vec<Option<TimeDelta>> {
    fn len(&self) -> usize {
        self.len()
    }
    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_some()).count()
    }
    fn f64_iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(
            self.as_slice()
                .iter()
                .copied()
                .map(|v| v.map(|v| v.seconds())),
        )
    }
}

impl Column for Vec<Option<TimeDelta>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_some(&self) -> usize {
        self.as_slice().iter().filter(|v| v.is_some()).count()
    }

    fn time_delta(&self) -> Option<&dyn TimeDeltaColumn> {
        Some(self)
    }

    fn boxed_copy(&self) -> Box<dyn Column> {
        Box::new(self.clone())
    }
}

/// Simple collection of named columns, owning the data
#[derive(Debug)]
pub struct NamedOwnedColumns {
    names: Vec<String>,
    columns: Vec<Box<dyn Column>>,
}

impl NamedOwnedColumns {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            columns: Vec::new(),
        }
    }

    /// Add a column with the given name
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
    fn names(&self) -> Vec<&str> {
        self.names.iter().map(|s| s.as_str()).collect()
    }

    fn column(&self, name: &str) -> Option<&dyn Column> {
        let Some(idx) = self.names.as_slice().iter().position(|k| k == name) else {
            return None;
        };
        self.columns.get(idx).map(|c| c.as_ref() as &dyn Column)
    }

    fn copy(&self) -> Arc<dyn Source> {
        let mut new_source = NamedOwnedColumns::new();
        for (name, col) in self.names.iter().zip(self.columns.iter()) {
            new_source.add_column(name, col.boxed_copy());
        }
        Arc::new(new_source)
    }
}

/// Simple collection of named columns, referencing external data
#[derive(Debug)]
pub struct NamedColumns<'a> {
    names: Vec<String>,
    columns: Vec<&'a dyn Column>,
}

impl<'a> NamedColumns<'a> {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            columns: Vec::new(),
        }
    }

    /// Add a column with the given name
    pub fn add_column(&mut self, name: &str, col: &'a dyn Column) {
        let position = self.names.iter().position(|n| n == name);
        if let Some(pos) = position {
            self.columns[pos] = col;
            return;
        }
        self.names.push(name.to_string());
        self.columns.push(col);
    }
}

impl<'a> Source for NamedColumns<'a> {
    fn names(&self) -> Vec<&str> {
        self.names.iter().map(|s| s.as_str()).collect()
    }

    fn column(&self, name: &str) -> Option<&dyn Column> {
        let Some(idx) = self.names.as_slice().iter().position(|k| k == name) else {
            return None;
        };
        self.columns.get(idx).map(|c| *c as &dyn Column)
    }

    fn copy(&self) -> Arc<dyn Source> {
        let mut new_source = NamedOwnedColumns::new();
        for (name, col) in self.names.iter().zip(self.columns.iter()) {
            new_source.add_column(name, col.boxed_copy());
        }
        Arc::new(new_source)
    }
}

/// Column implementation backed by vectors, type known at runtime
#[derive(Debug, Clone)]
pub enum VecColumn {
    /// f64 column
    F64(Vec<f64>),
    /// i64 column
    I64(Vec<Option<i64>>),
    /// string column
    Str(Vec<Option<String>>),
    /// time column
    Time(Vec<Option<DateTime>>),
    /// time delta column
    TimeDelta(Vec<Option<TimeDelta>>),
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
            VecColumn::Time(v) => v.len(),
            VecColumn::TimeDelta(v) => v.len(),
        }
    }

    fn len_some(&self) -> usize {
        match self {
            VecColumn::F64(v) => <dyn F64Column>::len_some(v),
            VecColumn::I64(v) => <dyn I64Column>::len_some(v),
            VecColumn::Str(v) => <dyn StrColumn>::len_some(v),
            VecColumn::Time(v) => <dyn TimeColumn>::len_some(v),
            VecColumn::TimeDelta(v) => <dyn TimeDeltaColumn>::len_some(v),
        }
    }

    fn sample_iter(&self) -> Box<dyn Iterator<Item = Sample<'_>> + '_> {
        match self {
            VecColumn::F64(v) => Box::new(v.iter().map(|v| (*v).into())),
            VecColumn::I64(v) => Box::new(v.iter().map(|v| (*v).into())),
            VecColumn::Str(v) => Box::new(v.iter().map(|v| match v {
                Some(s) => Sample::Cat(s.as_str()),
                None => Sample::Null,
            })),
            VecColumn::Time(v) => Box::new(v.iter().map(|v| (*v).into())),
            VecColumn::TimeDelta(v) => Box::new(v.iter().map(|v| (*v).into())),
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

    fn time(&self) -> Option<&dyn TimeColumn> {
        match self {
            VecColumn::Time(v) => Some(v),
            _ => None,
        }
    }

    fn time_delta(&self) -> Option<&dyn TimeDeltaColumn> {
        match self {
            VecColumn::TimeDelta(v) => Some(v),
            _ => None,
        }
    }

    fn boxed_copy(&self) -> Box<dyn Column> {
        match self {
            VecColumn::F64(v) => Box::new(v.clone()),
            VecColumn::I64(v) => Box::new(v.clone()),
            VecColumn::Str(v) => Box::new(v.clone()),
            VecColumn::Time(v) => Box::new(v.clone()),
            VecColumn::TimeDelta(v) => Box::new(v.clone()),
        }
    }
}

/// Simple table source backed by vectors
/// This source owns the data and ensure that all columns have the same length
#[derive(Clone)]
pub struct TableSource {
    heads: Vec<String>,
    columns: Vec<VecColumn>,
    len: usize,
}

impl TableSource {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self {
            heads: Vec::new(),
            columns: Vec::new(),
            len: 0,
        }
    }

    /// Get the column names
    pub fn heads(&self) -> &[String] {
        &self.heads
    }

    /// Add a column with the given name
    /// If the column is shorter than existing columns, it will be padded with null values.
    /// If the column is longer than existing columns, existing columns will be padded with null values
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
                    VecColumn::Time(vec) => vec.push(None),
                    VecColumn::TimeDelta(vec) => vec.push(None),
                }
            }
        }
    }

    /// Add a column with the given name, returning self for chaining
    pub fn with_column(mut self, name: &str, col: VecColumn) -> Self {
        self.add_column(name, col);
        self
    }

    /// Add a f64 column with the given name, returning self for chaining
    pub fn with_f64_column(mut self, name: &str, col: Vec<f64>) -> Self {
        self.add_column(name, VecColumn::F64(col));
        self
    }

    /// Add an i64 column with the given name, returning self for chaining
    pub fn with_i64_column(mut self, name: &str, col: Vec<Option<i64>>) -> Self {
        self.add_column(name, VecColumn::I64(col));
        self
    }

    /// Add a string column with the given name, returning self for chaining
    pub fn with_str_column(mut self, name: &str, col: Vec<Option<String>>) -> Self {
        self.add_column(name, VecColumn::Str(col));
        self
    }

    /// Add a time column with the given name, returning self for chaining
    pub fn with_time_column(mut self, name: &str, col: Vec<Option<DateTime>>) -> Self {
        self.add_column(name, VecColumn::Time(col));
        self
    }

    /// Add a time delta column with the given name, returning self for chaining
    pub fn with_time_delta_column(mut self, name: &str, col: Vec<Option<TimeDelta>>) -> Self {
        self.add_column(name, VecColumn::TimeDelta(col));
        self
    }

    /// Get the number of rows in the table
    pub fn len(&self) -> usize {
        self.len
    }
}

impl Source for TableSource {
    fn names(&self) -> Vec<&str> {
        self.heads.iter().map(|s| s.as_str()).collect()
    }

    fn column(&self, name: &str) -> Option<&dyn Column> {
        let Some(idx) = self.heads.as_slice().iter().position(|k| k == name) else {
            return None;
        };
        self.columns.get(idx).map(|c| c as &dyn Column)
    }

    fn copy(&self) -> Arc<dyn Source> {
        Arc::new(self.clone())
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
                VecColumn::Time(v) => match v.get(row) {
                    Some(Some(t)) => format!("{}", t),
                    _ => "(null)".to_string(),
                },
                VecColumn::TimeDelta(v) => match v.get(row) {
                    Some(Some(t)) => format!("{}", t),
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
