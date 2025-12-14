use polars::prelude::*;

use crate::data;

impl data::F64Column for Float64Chunked {
    fn len(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(self.iter())
    }
}

impl data::I64Column for Int64Chunked {
    fn len(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<i64>> + '_> {
        Box::new(self.iter())
    }
}

impl data::StrColumn for StringChunked {
    fn len(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
        Box::new(self.iter())
    }
}

#[inline]
fn series_len(s: &Series) -> usize {
    s.len()
}

impl data::Column for Series {
    fn len(&self) -> usize {
        series_len(self)
    }

    fn len_some(&self) -> usize {
        self.len() - self.null_count()
    }

    fn f64(&self) -> Option<&dyn data::F64Column> {
        self.try_f64().map(|s| s as &dyn data::F64Column)
    }

    fn i64(&self) -> Option<&dyn data::I64Column> {
        self.try_i64().map(|s| s as &dyn data::I64Column)
    }

    fn str(&self) -> Option<&dyn data::StrColumn> {
        self.try_str().map(|s| s as &dyn data::StrColumn)
    }
}

impl data::Source for DataFrame {
    fn column(&self, name: &str) -> Option<&dyn data::Column> {
        self.column(name)
            .map(|c| c.as_materialized_series() as &dyn data::Column)
            .ok()
    }
}
