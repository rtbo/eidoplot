//! Module for parsing and exporting CSV data.
use std::collections::HashMap;

use super::{TableSource, VecColumn};
use crate::data::SampleRef;
#[cfg(feature = "time")]
use crate::time::DateTime;

/// CSV parsing error
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Inconsistent column count
    ColCount {
        /// Line number where the error occurred
        line: usize,
    },
    /// Inconsistent column type
    ColType {
        /// Line number where the error occurred
        line: usize,
    },
    /// A unknown column title was referenced
    UnknownCol {
        /// Column title
        title: String,
    },
    /// A unknown column index was referenced
    UnknownColIdx {
        /// Column index
        idx: usize,
    },
    /// Column contains only null values
    AllNull {
        /// Column index
        col: usize,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ColCount { line } => {
                write!(f, "Inconsistent column count at line {line}")
            }
            ParseError::ColType { line } => {
                write!(f, "Inconsistent column type at line {line}")
            }
            ParseError::UnknownCol { title } => {
                write!(f, "Unknown column title {title}")
            }
            ParseError::UnknownColIdx { idx } => {
                write!(f, "Unknown column index {idx}")
            }
            ParseError::AllNull { col } => write!(f, "Only null values in column {col}"),
        }
    }
}

impl std::error::Error for ParseError {}

#[allow(missing_copy_implementations)]
/// CSV parsing spec for a specific column
#[derive(Debug, Clone)]
pub enum ColSpec {
    /// Let the parser guess the column type
    /// (the default if no spec is provided)
    Auto,
    /// Column of floating point numbers
    F64,
    /// Column of integer numbers
    I64,
    /// Column of strings
    Str,
    #[cfg(feature = "time")]
    /// Column of date/time values with automatic format detection
    TimeAuto,
    #[cfg(feature = "time")]
    /// Column of date/time values with custom format
    TimeCustom {
        /// Date/time format string
        fmt: String,
    },
}

#[derive(Debug, Clone)]
enum ColId {
    Tit(String),
    Idx(usize),
}

impl ColId {
    fn idx(&self) -> Option<usize> {
        match self {
            ColId::Idx(idx) => Some(*idx),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
enum CsvColumn {
    F64(Vec<f64>),
    I64(Vec<Option<i64>>),
    Str(Vec<Option<String>>),

    #[cfg(feature = "time")]
    Time(Vec<Option<DateTime>>, Option<String>), // data and parse format
}

impl CsvColumn {
    fn len(&self) -> usize {
        match self {
            CsvColumn::F64(vec) => vec.len(),
            CsvColumn::I64(vec) => vec.len(),
            CsvColumn::Str(vec) => vec.len(),
            #[cfg(feature = "time")]
            CsvColumn::Time(vec, _) => vec.len(),
        }
    }
}

/// A CSV file parser
#[derive(Debug, Clone)]
pub struct Parser {
    sep: char,
    col_specs: Vec<(ColId, ColSpec)>,
}

impl Parser {
    /// Creates a new CSV parser with default settings.
    pub fn new() -> Self {
        Parser {
            sep: ',',
            col_specs: Vec::new(),
        }
    }

    /// Set the separator character
    pub fn with_sep(mut self, sep: char) -> Self {
        self.sep = sep;
        self
    }

    /// Add a column specification by title
    pub fn with_col_spec(mut self, title: &str, spec: ColSpec) -> Self {
        self.col_specs.push((ColId::Tit(title.to_string()), spec));
        self
    }

    /// Add a column specification by index
    pub fn with_col_spec_idx(mut self, idx: usize, spec: ColSpec) -> Self {
        self.col_specs.push((ColId::Idx(idx), spec));
        self
    }

    /// Parse the given CSV data
    pub fn parse(self, data: &str) -> Result<TableSource, ParseError> {
        let sep = self.sep;
        let mut col_specs = self.col_specs;

        let mut lines = data.lines();
        let Some(head_line) = lines.next() else {
            return Ok(TableSource::new());
        };
        let header: Vec<&str> = head_line.split(sep).map(|s| s.trim()).collect();

        for (idx, head) in header.iter().enumerate() {
            if let Some(spec) = col_specs.iter_mut().find(|(id, _)| match id {
                ColId::Tit(title) => title == head,
                ColId::Idx(i) => *i == idx,
            }) {
                spec.0 = ColId::Idx(idx);
            } else {
                col_specs.push((ColId::Idx(idx), ColSpec::Auto));
            }
        }

        // check column specs consistencty
        for col_spec in &col_specs {
            match &col_spec.0 {
                ColId::Tit(title) => {
                    return Err(ParseError::UnknownCol {
                        title: title.clone(),
                    });
                }
                ColId::Idx(idx) if idx >= &header.len() => {
                    return Err(ParseError::UnknownColIdx { idx: *idx });
                }
                _ => (),
            }
        }

        // collect col specs in a sorted vec
        col_specs.sort_unstable_by(|a, b| a.0.idx().unwrap().cmp(&b.0.idx().unwrap()));

        // build empty columns
        // None columns will have their type determined at first non-null value
        // Once a type is determined, all following rows must have the same type
        let mut columns: Vec<Option<CsvColumn>> = col_specs
            .into_iter()
            .map(|spec| match spec.1 {
                ColSpec::F64 => Some(CsvColumn::F64(Vec::new())),
                ColSpec::I64 => Some(CsvColumn::I64(Vec::new())),
                ColSpec::Str => Some(CsvColumn::Str(Vec::new())),
                #[cfg(feature = "time")]
                ColSpec::TimeAuto => Some(CsvColumn::Time(Vec::new(), None)),
                #[cfg(feature = "time")]
                ColSpec::TimeCustom { fmt } => Some(CsvColumn::Time(Vec::new(), Some(fmt))),
                ColSpec::Auto => None,
            })
            .collect();

        let mut row_count = 0;
        for line in lines {
            for (cidx, data) in line.split(sep).map(|s| s.trim()).enumerate() {
                if cidx >= columns.len() {
                    return Err(ParseError::ColCount { line: 2 });
                }
                let col = &mut columns[cidx];
                if col.is_none() && !data.is_empty() {
                    *col = Some(guess_column_type(data, row_count));
                } else if let Some(col) = col {
                    parse_column_data(data, col)?;
                }
            }
            row_count += 1;
        }

        let mut src = TableSource::new();
        for (ci, csv_col) in columns.into_iter().enumerate() {
            let col = match csv_col {
                Some(CsvColumn::F64(vec)) => Some(VecColumn::F64(vec)),
                Some(CsvColumn::I64(vec)) => Some(VecColumn::I64(vec)),
                Some(CsvColumn::Str(vec)) => Some(VecColumn::Str(vec)),
                #[cfg(feature = "time")]
                Some(CsvColumn::Time(vec, ..)) => Some(VecColumn::Time(vec)),
                None => None,
            };
            let col = col.ok_or(ParseError::AllNull { col: ci })?;
            src.add_column(&header[ci], col);
        }
        Ok(src)
    }
}

fn guess_column_type(data: &str, num_nulls: usize) -> CsvColumn {
    #[cfg(feature = "time")]
    if let Ok(dt) = DateTime::fmt_parse(data, "%Y-%m-%d %H:%M:%S%.f") {
        let mut vec: Vec<Option<DateTime>> = vec![None; num_nulls];
        vec.push(Some(dt));
        return CsvColumn::Time(vec, Some("%Y-%m-%d %H:%M:%S%.f".to_string()));
    }

    #[cfg(feature = "time")]
    if let Ok(dt) = DateTime::fmt_parse(data, "%Y-%m-%d %H:%M:%S") {
        let mut vec: Vec<Option<DateTime>> = vec![None; num_nulls];
        vec.push(Some(dt));
        return CsvColumn::Time(vec, Some("%Y-%m-%d %H:%M:%S".to_string()));
    }

    #[cfg(feature = "time")]
    if let Ok(dt) = DateTime::fmt_parse(data, "%Y-%m-%d") {
        let mut vec: Vec<Option<DateTime>> = vec![None; num_nulls];
        vec.push(Some(dt));
        return CsvColumn::Time(vec, Some("%Y-%m-%d".to_string()));
    }

    // intentionally not trying i64 first to avoid misdetection of float columns with integer values
    if let Ok(d) = data.parse::<f64>() {
        let mut vec: Vec<f64> = vec![f64::NAN; num_nulls];
        vec.push(d);
        CsvColumn::F64(vec)
    } else {
        let mut vec: Vec<Option<String>> = vec![None; num_nulls];
        vec.push(Some(data.to_string()));
        CsvColumn::Str(vec)
    }
}

fn parse_column_data(data: &str, col: &mut CsvColumn) -> Result<(), ParseError> {
    match col {
        CsvColumn::F64(vec) => {
            if data.is_empty() {
                vec.push(f64::NAN);
            } else if let Ok(d) = data.parse::<f64>() {
                vec.push(d);
            } else {
                return Err(ParseError::ColType { line: 2 });
            }
        }
        CsvColumn::I64(vec) => {
            if data.is_empty() {
                vec.push(None);
            } else if let Ok(d) = data.parse::<i64>() {
                vec.push(Some(d));
            } else {
                return Err(ParseError::ColType {
                    line: col.len() + 2,
                });
            }
        }
        CsvColumn::Str(vec) => {
            if data.is_empty() {
                vec.push(None);
            } else {
                vec.push(Some(data.to_string()));
            }
        }
        #[cfg(feature = "time")]
        CsvColumn::Time(vec, fmt) => {
            if data.is_empty() {
                vec.push(None);
            } else {
                match fmt {
                    None => {
                        // guess format
                        if let Ok(dt) = DateTime::fmt_parse(data, "%Y-%m-%d %H:%M:%S%.f") {
                            vec.push(Some(dt));
                            *fmt = Some("%Y-%m-%d %H:%M:%S%.f".to_string());
                        } else if let Ok(dt) = DateTime::fmt_parse(data, "%Y-%m-%d %H:%M:%S") {
                            vec.push(Some(dt));
                            *fmt = Some("%Y-%m-%d %H:%M:%S".to_string());
                        } else if let Ok(dt) = DateTime::fmt_parse(data, "%Y-%m-%d") {
                            vec.push(Some(dt));
                            *fmt = Some("%Y-%m-%d".to_string());
                        } else {
                            return Err(ParseError::ColType {
                                line: col.len() + 2,
                            });
                        }
                    }
                    Some(fmt) => {
                        if let Ok(dt) = DateTime::fmt_parse(data, fmt) {
                            vec.push(Some(dt));
                        } else {
                            return Err(ParseError::ColType {
                                line: col.len() + 2,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// An error that can occur during CSV export
#[derive(Debug)]
pub enum ExportError {
    /// I/O error
    Io(std::io::Error),
    /// Row count is not consistent across columns
    InconsistentRowCount,
    /// Column type not consistent with specified format
    /// See [`ExportFormat`]
    InconsistentColumnType,
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Io(e) => write!(f, "IO error: {}", e),
            ExportError::InconsistentRowCount => write!(f, "Inconsistent row count"),
            ExportError::InconsistentColumnType => write!(f, "Inconsistent column type"),
        }
    }
}

impl From<std::io::Error> for ExportError {
    fn from(err: std::io::Error) -> Self {
        ExportError::Io(err)
    }
}

impl std::error::Error for ExportError {}

/// A CSV export column value format
#[derive(Default)]
pub enum ExportFormat {
    /// Automatic format based on sample type (the default)
    #[default]
    Auto,
    /// Floating point number with fixed number of decimals
    Decimals(usize),
    /// Floating point number in scientific notation with fixed number of decimals
    Scientific(usize),
    #[cfg(feature = "time")]
    /// Date/time with custom format
    DateTime(String),
    #[cfg(feature = "time")]
    /// Time delta with custom format
    TimeDelta(String),
    /// Custom formatting function
    Custom(Box<dyn Fn(SampleRef) -> String + Send + Sync>),
}

impl std::fmt::Debug for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Auto => write!(f, "CsvExportFormat::Auto"),
            ExportFormat::Decimals(d) => write!(f, "CsvExportFormat::Decimals({})", d),
            ExportFormat::Scientific(d) => {
                write!(f, "CsvExportFormat::Scientific({})", d)
            }
            #[cfg(feature = "time")]
            ExportFormat::DateTime(fmt) => {
                write!(f, "CsvExportFormat::DateTime({})", fmt)
            }
            #[cfg(feature = "time")]
            ExportFormat::TimeDelta(fmt) => {
                write!(f, "CsvExportFormat::TimeDelta({})", fmt)
            }
            ExportFormat::Custom(_) => write!(f, "CsvExportFormat::Custom(..)"),
        }
    }
}

impl ExportFormat {
    fn format_value(
        &self,
        value: SampleRef,
        decimal_point: char,
    ) -> Result<String, ExportError> {
        match (self, value) {
            (_, SampleRef::Null) => Ok(String::new()),
            (ExportFormat::Auto, SampleRef::Cat(s)) => Ok(s.to_string()),
            (ExportFormat::Auto, SampleRef::Num(v)) => {
                Ok(handle_dec_point(format!("{}", v), decimal_point))
            }
            (ExportFormat::Decimals(d), SampleRef::Num(v)) => {
                Ok(handle_dec_point(format!("{:.*}", *d, v), decimal_point))
            }
            (ExportFormat::Scientific(d), SampleRef::Num(v)) => {
                Ok(handle_dec_point(format!("{:.*e}", *d, v), decimal_point))
            }
            #[cfg(feature = "time")]
            (ExportFormat::Auto, SampleRef::Time(dt)) => Ok(dt.to_string()),
            #[cfg(feature = "time")]
            (ExportFormat::Auto, SampleRef::TimeDelta(dt)) => Ok(dt.to_string()),
            #[cfg(feature = "time")]
            (ExportFormat::DateTime(fmt), SampleRef::Time(dt)) => Ok(dt.fmt_to_string(fmt)),
            #[cfg(feature = "time")]
            (ExportFormat::TimeDelta(fmt), SampleRef::TimeDelta(td)) => {
                Ok(td.fmt_to_string(fmt))
            }
            (ExportFormat::Custom(f), v) => Ok(f(v)),
            _ => Err(ExportError::InconsistentColumnType),
        }
    }
}

fn handle_dec_point(s: String, decimal_point: char) -> String {
    if decimal_point != '.' {
        s.replace('.', &decimal_point.to_string())
    } else {
        s
    }
}

/// CSV header row export options
#[derive(Debug, Default)]
pub enum ExportHeaderRow {
    /// No header row
    None,
    #[default]
    /// Use column names as headers
    Names,
    /// Use custom mapped names
    /// The map key is the original column name, and the value is the desired header name
    /// If a column name is not found in the map, the original name is used
    MappedNames(HashMap<String, String>),
}

/// Options for exporting CSV data
#[derive(Debug)]
pub struct ExportOptions {
    /// Header row options
    pub header_row: ExportHeaderRow,
    /// Column delimiter character (default to ',')
    pub delimiter: char,
    /// Decimal point character (default to '.')
    pub decimal_point: char,
    /// Per-column value formats (default to all Auto)
    /// Only include here columns that need custom formatting
    pub value_formats: Option<HashMap<String, ExportFormat>>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            header_row: ExportHeaderRow::default(),
            delimiter: ',',
            decimal_point: '.',
            value_formats: None,
        }
    }
}

/// Export the given data source to CSV format
pub fn export_data_source<W, D>(
    output: &mut W,
    data_source: &D,
    options: ExportOptions,
) -> Result<(), ExportError>
where
    W: std::io::Write,
    D: super::Source + ?Sized,
{
    let names = data_source.names();
    if names.is_empty() {
        return Ok(());
    }

    if matches!(options.header_row, ExportHeaderRow::Names | ExportHeaderRow::MappedNames(_)) {
        for (i, h) in names.iter().enumerate() {
            let h = match &options.header_row {
                ExportHeaderRow::MappedNames(map) => {
                    map.get(*h).map(|s| s.as_str()).unwrap_or(*h)
                }
                _ => *h,
            };
            write!(output, "{}", h)?;
            if i + 1 < names.len() {
                write!(output, "{}", options.delimiter)?;
            }
        }
        writeln!(output)?;
    }

    let mut columns = Vec::with_capacity(names.len());
    let mut data_len = None;
    for n in 0..names.len() {
        let col = data_source.column(names[n]).unwrap();
        match data_len {
            Some(len) => {
                if col.len() != len {
                    return Err(ExportError::InconsistentRowCount);
                }
            }
            None => {
                data_len = Some(col.len());
            }
        }
        columns.push(col.sample_iter());
    }

    let def_fmt = ExportFormat::default();
    let value_formats = names
        .iter()
        .map(|name| {
            options
                .value_formats
                .as_ref()
                .and_then(|vf| vf.get(*name))
                .unwrap_or(&def_fmt)
        })
        .collect::<Vec<_>>();

    for _ in 0..data_len.unwrap() {
        for (cidx, col_iter) in columns.iter_mut().enumerate() {
            let sample = col_iter.next().expect("Inconsistent row count");
            let fmt = value_formats[cidx];
            let value_str = fmt.format_value(sample, options.decimal_point)?;
            write!(output, "{}", value_str)?;
            if cidx + 1 < names.len() {
                write!(output, "{}", options.delimiter)?;
            }
        }
        writeln!(output)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Source;

    pub const CSV_DATA: &str = "Int,Float,Str\n1,1.0,one\n2,2.0,two\n3,3.0,three\n";

    #[test]
    fn test_parse_csv_data() {
        let src = Parser::new().parse(CSV_DATA).unwrap();
        assert_eq!(src.len(), 3);

        let int_col = src
            .column("Int")
            .and_then(|c| c.f64())
            .unwrap()
            .f64_iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        let float_col = src
            .column("Float")
            .and_then(|c| c.f64())
            .unwrap()
            .f64_iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        let str_col = src
            .column("Str")
            .and_then(|c| c.str())
            .unwrap()
            .str_iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(int_col, &[1.0, 2.0, 3.0]);
        assert_eq!(float_col, &[1.0, 2.0, 3.0]);
        assert_eq!(str_col, &["one", "two", "three"]);
    }

    pub const CSV_NULL_DATA: &str = "Int,Float,Str\n1,1.0,one\n2,,two\n3,3.0,three\n";

    #[test]
    fn test_parse_csv_data_null() {
        let src = Parser::new().parse(CSV_NULL_DATA).unwrap();
        assert_eq!(src.len(), 3);

        let int_col = src
            .column("Int")
            .and_then(|c| c.f64())
            .unwrap()
            .f64_iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        let float_col = src
            .column("Float")
            .and_then(|c| c.f64())
            .unwrap()
            .f64_iter()
            .collect::<Vec<_>>();

        let str_col = src
            .column("Str")
            .and_then(|c| c.str())
            .unwrap()
            .str_iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(int_col, &[1.0, 2.0, 3.0]);
        assert_eq!(float_col, &[Some(1.0), None, Some(3.0)]);
        assert_eq!(str_col, &["one", "two", "three"]);
    }

    pub const CSV_NULL_DATA_FST_LINE: &str = "Int,Float,Str\n1,,one\n2,2.0,two\n3,3.0,three\n";

    #[test]
    fn test_parse_csv_data_null_fst_line() {
        let src = Parser::new().parse(CSV_NULL_DATA_FST_LINE).unwrap();
        assert_eq!(src.len(), 3);

        let int_col = src
            .column("Int")
            .and_then(|c| c.f64())
            .unwrap()
            .f64_iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        let float_col = src
            .column("Float")
            .and_then(|c| c.f64())
            .unwrap()
            .f64_iter()
            .collect::<Vec<_>>();

        let str_col = src
            .column("Str")
            .and_then(|c| c.str())
            .unwrap()
            .str_iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(int_col, &[1.0, 2.0, 3.0]);
        assert_eq!(float_col, &[None, Some(2.0), Some(3.0)]);
        assert_eq!(str_col, &["one", "two", "three"]);
    }

    #[cfg(feature = "time")]
    pub const CSV_DATE: &str = "Date,Float\n2025-01-01,1.0\n2025-01-02,2.0\n2025-01-03,3.0\n";

    #[cfg(feature = "time")]
    #[test]
    fn test_parse_csv_date() {
        let src = Parser::new().parse(CSV_DATE).unwrap();
        assert_eq!(src.len(), 3);

        let date_col = src
            .column("Date")
            .and_then(|c| c.time())
            .unwrap()
            .time_iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        let float_col = src
            .column("Float")
            .and_then(|c| c.f64())
            .unwrap()
            .f64_iter()
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(
            date_col,
            &[
                DateTime::from_ymd(2025, 1, 1).unwrap(),
                DateTime::from_ymd(2025, 1, 2).unwrap(),
                DateTime::from_ymd(2025, 1, 3).unwrap()
            ]
        );
        assert_eq!(float_col, &[1.0, 2.0, 3.0]);
    }
}
