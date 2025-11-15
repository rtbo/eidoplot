use eidoplot::time::DateTime;
use eidoplot::data::{TableSource, VecColumn};

/// Create a linearly spaced vector of `num` elements between `start` and `end`
pub fn linspace(start: f64, end: f64, num: usize) -> Vec<f64> {
    let step = (end - start) / (num as f64 - 1.0);
    (0..num).map(|i| start + i as f64 * step).collect()
}

/// Create a log-spaced vector of `num` elements between `start` and `end`
pub fn logspace(start: f64, end: f64, num: usize) -> Vec<f64> {
    let log_start = start.log10();
    let log_end = end.log10();
    let step = (log_end - log_start) / (num as f64 - 1.0);
    (0..num)
        .map(|i| 10f64.powf(log_start + i as f64 * step))
        .collect()
}

/// Create a linearly spaced time vector of `num` elements between `start` and `end`
pub fn timespace(start: DateTime, end: DateTime, num: usize) -> Vec<DateTime> {
    let step = (end - start) / (num as f64 - 1.0);
    let mut result = Vec::with_capacity(num);
    let mut cur = start;
    for _ in 0..num {
        result.push(cur);
        cur += step;
    }
    result
}

#[derive(Debug, Clone, Copy)]
pub enum CsvParseError {
    ColCount { line: usize },
    ColType { line: usize },
    AllNull { col: usize },
}

impl std::fmt::Display for CsvParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvParseError::ColCount { line } => {
                write!(f, "Inconsistent column count at line {line}")
            }
            CsvParseError::ColType { line } => {
                write!(f, "Inconsistent column type at line {line}")
            }
            CsvParseError::AllNull { col } => write!(f, "Only null values in column {col}"),
        }
    }
}

impl std::error::Error for CsvParseError {}

pub fn parse_csv_data(data: &str, sep: char) -> Result<TableSource, CsvParseError> {
    let mut lines = data.lines();
    let Some(head_line) = lines.next() else {
        return Ok(TableSource::new());
    };
    let mut builder = VecSourceRowBuilder::new();
    for line in lines {
        let mut row = Vec::<VecValue>::new();
        for data in line.split(sep).map(|s| s.trim()) {
            if data.is_empty() {
                row.push(VecValue::Null);
            } else if let Ok(d) = data.parse::<i64>() {
                row.push(VecValue::I64(d));
            } else if let Ok(d) = data.parse::<f64>() {
                row.push(VecValue::F64(d));
            } else {
                row.push(VecValue::Str(data.to_string()));
            }
        }
        builder.push_row(VecRow { columns: row });
    }

    let header: Vec<&str> = head_line.split(sep).map(|s| s.trim()).collect();
    builder.finish(&header)
}

#[derive(Debug, Clone)]
enum VecValue {
    Null,
    F64(f64),
    I64(i64),
    Str(String),
}

#[derive(Debug)]
struct VecRow {
    columns: Vec<VecValue>,
}

// At the moment, all rows are collected until finish is called.
// This is really not efficient and bad performance is probably noticeable with large CSV files.
// TODO: more efficient implementation. E.g. a VecColumn wrapper that can accumulate a count of null values
// until an actual value is known.
#[derive(Debug)]
struct VecSourceRowBuilder {
    rows: Vec<VecRow>,
}

impl VecSourceRowBuilder {
    pub fn new() -> Self {
        VecSourceRowBuilder { rows: Vec::new() }
    }

    pub fn push_row(&mut self, row: VecRow) {
        self.rows.push(row);
    }

    pub fn finish(self, heads: &[&str]) -> Result<TableSource, CsvParseError> {
        let columns: Result<Vec<_>, _> = (0..heads.len())
            .into_iter()
            .map(|ci| build_empty_column(&self.rows, ci))
            .collect();
        let mut columns = columns?;
        for (ri, row) in self.rows.into_iter().enumerate() {
            for (ci, col) in row.columns.into_iter().enumerate() {
                if ci >= columns.len() {
                    return Err(CsvParseError::ColCount { line: ri + 2 });
                }
                match (&mut columns[ci], col) {
                    (VecColumn::F64(vec), VecValue::F64(v)) => vec.push(v),
                    (VecColumn::F64(vec), VecValue::Null) => vec.push(f64::NAN),
                    (VecColumn::I64(vec), VecValue::I64(v)) => vec.push(Some(v)),
                    (VecColumn::I64(vec), VecValue::Null) => vec.push(None),
                    (VecColumn::Str(vec), VecValue::Str(v)) => vec.push(Some(v)),
                    (VecColumn::Str(vec), VecValue::Null) => vec.push(None),
                    _ => {
                        return Err(CsvParseError::ColType { line: ri + 2 });
                    }
                }
            }
        }

        let mut src = TableSource::new();
        for (ci, col) in columns.into_iter().enumerate() {
            src.add_column(heads[ci], col);
        }
        Ok(src)
    }
}

fn build_empty_column(rows: &[VecRow], col_idx: usize) -> Result<VecColumn, CsvParseError> {
    for (ri, row) in rows.iter().enumerate() {
        if ri >= row.columns.len() {
            return Err(CsvParseError::ColCount { line: ri + 2 });
        }
        match &row.columns[col_idx] {
            VecValue::F64(_) => return Ok(VecColumn::F64(Vec::with_capacity(rows.len()))),
            VecValue::I64(_) => return Ok(VecColumn::I64(Vec::with_capacity(rows.len()))),
            VecValue::Str(_) => return Ok(VecColumn::Str(Vec::with_capacity(rows.len()))),
            _ => (),
        }
    }
    Err(CsvParseError::AllNull { col: col_idx })
}

#[cfg(test)]
mod tests {

    use eidoplot::data::Source;

    pub const CSV_DATA: &str = "Int,Float,Str\n1,1.0,one\n2,2.0,two\n3,3.0,three\n";

    #[test]
    fn test_parse_csv_data() {
        let src = super::parse_csv_data(CSV_DATA, ',').unwrap();
        assert_eq!(src.len(), 3);
        let int_col = src
            .column("Int")
            .and_then(|c| c.i64())
            .unwrap()
            .iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();
        let float_col = src
            .column("Float")
            .and_then(|c| c.f64())
            .unwrap()
            .iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();
        let str_col = src
            .column("Str")
            .and_then(|c| c.str())
            .unwrap()
            .iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(int_col, &[1, 2, 3]);
        assert_eq!(float_col, &[1.0, 2.0, 3.0]);
        assert_eq!(str_col, &["one", "two", "three"]);
    }

    pub const CSV_NULL_DATA: &str = "Int,Float,Str\n1,1.0,one\n2,,two\n3,3.0,three\n";

    #[test]
    fn test_parse_csv_data_null() {
        let src = super::parse_csv_data(CSV_NULL_DATA, ',').unwrap();
        assert_eq!(src.len(), 3);
        let int_col = src
            .column("Int")
            .and_then(|c| c.i64())
            .unwrap()
            .iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();
        let float_col = src
            .column("Float")
            .and_then(|c| c.f64())
            .unwrap()
            .iter()
            .collect::<Vec<_>>();
        let str_col = src
            .column("Str")
            .and_then(|c| c.str())
            .unwrap()
            .iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(int_col, &[1, 2, 3]);
        assert_eq!(float_col, &[Some(1.0), None, Some(3.0)]);
        assert_eq!(str_col, &["one", "two", "three"]);
    }

    pub const CSV_NULL_DATA_FST_LINE: &str = "Int,Float,Str\n1,,one\n2,2.0,two\n3,3.0,three\n";

    #[test]
    fn test_parse_csv_data_null_fst_line() {
        let src = super::parse_csv_data(CSV_NULL_DATA_FST_LINE, ',').unwrap();
        assert_eq!(src.len(), 3);
        let int_col = src
            .column("Int")
            .and_then(|c| c.i64())
            .unwrap()
            .iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();
        let float_col = src
            .column("Float")
            .and_then(|c| c.f64())
            .unwrap()
            .iter()
            .collect::<Vec<_>>();
        let str_col = src
            .column("Str")
            .and_then(|c| c.str())
            .unwrap()
            .iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(int_col, &[1, 2, 3]);
        assert_eq!(float_col, &[None, Some(2.0), Some(3.0)]);
        assert_eq!(str_col, &["one", "two", "three"]);
    }
}
