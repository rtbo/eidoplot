use super::{TableSource, VecColumn};
use crate::time::DateTime;

#[derive(Debug, Clone)]
pub enum CsvParseError {
    ColCount { line: usize },
    ColType { line: usize },
    UnknownCol { title: String },
    UnknownColIdx { idx: usize },
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
            CsvParseError::UnknownCol { title } => {
                write!(f, "Unknown column title {title}")
            }
            CsvParseError::UnknownColIdx { idx } => {
                write!(f, "Unknown column index {idx}")
            }
            CsvParseError::AllNull { col } => write!(f, "Only null values in column {col}"),
        }
    }
}

impl std::error::Error for CsvParseError {}

/// CSV parsing spec for a specific column
#[derive(Debug, Clone)]
pub enum CsvColSpec {
    Auto,
    F64,
    I64,
    Str,
    TimeAuto,
    TimeCustom { fmt: String },
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
    // data and parse format
    Time(Vec<Option<DateTime>>, Option<String>),
}

impl CsvColumn {
    fn len(&self) -> usize {
        match self {
            CsvColumn::F64(vec) => vec.len(),
            CsvColumn::I64(vec) => vec.len(),
            CsvColumn::Str(vec) => vec.len(),
            CsvColumn::Time(vec, _) => vec.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CsvParser {
    sep: char,
    col_specs: Vec<(ColId, CsvColSpec)>,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            sep: ',',
            col_specs: Vec::new(),
        }
    }

    pub fn with_sep(mut self, sep: char) -> Self {
        self.sep = sep;
        self
    }

    pub fn with_col_spec(mut self, title: &str, spec: CsvColSpec) -> Self {
        self.col_specs.push((ColId::Tit(title.to_string()), spec));
        self
    }

    pub fn with_col_spec_idx(mut self, idx: usize, spec: CsvColSpec) -> Self {
        self.col_specs.push((ColId::Idx(idx), spec));
        self
    }

    pub fn parse(self, data: &str) -> Result<TableSource, CsvParseError> {
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
                col_specs.push((ColId::Idx(idx), CsvColSpec::Auto));
            }
        }

        // check column specs consistencty
        for col_spec in &col_specs {
            match &col_spec.0 {
                ColId::Tit(title) => {
                    return Err(CsvParseError::UnknownCol {
                        title: title.clone(),
                    });
                }
                ColId::Idx(idx) if idx >= &header.len() => {
                    return Err(CsvParseError::UnknownColIdx { idx: *idx });
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
                CsvColSpec::F64 => Some(CsvColumn::F64(Vec::new())),
                CsvColSpec::I64 => Some(CsvColumn::I64(Vec::new())),
                CsvColSpec::Str => Some(CsvColumn::Str(Vec::new())),
                CsvColSpec::TimeAuto => Some(CsvColumn::Time(Vec::new(), None)),
                CsvColSpec::TimeCustom { fmt } => Some(CsvColumn::Time(Vec::new(), Some(fmt))),
                CsvColSpec::Auto => None,
            })
            .collect();

        let mut row_count = 0;
        for line in lines {
            for (cidx, data) in line.split(sep).map(|s| s.trim()).enumerate() {
                if cidx >= columns.len() {
                    return Err(CsvParseError::ColCount { line: 2 });
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
                Some(CsvColumn::Time(vec, ..)) => Some(VecColumn::Time(vec)),
                None => None,
            };
            let col = col.ok_or(CsvParseError::AllNull { col: ci })?;
            src.add_column(&header[ci], col);
        }
        Ok(src)
    }
}

fn guess_column_type(data: &str, num_nulls: usize) -> CsvColumn {
    if let Ok(dt) = DateTime::fmt_parse(data, "%Y-%m-%d %H:%M:%S%.f") {
        let mut vec: Vec<Option<DateTime>> = vec![None; num_nulls];
        vec.push(Some(dt));
        CsvColumn::Time(vec, Some("%Y-%m-%d %H:%M:%S%.f".to_string()))
    } else if let Ok(dt) = DateTime::fmt_parse(data, "%Y-%m-%d %H:%M:%S") {
        let mut vec: Vec<Option<DateTime>> = vec![None; num_nulls];
        vec.push(Some(dt));
        CsvColumn::Time(vec, Some("%Y-%m-%d %H:%M:%S".to_string()))
    } else if let Ok(dt) = DateTime::fmt_parse(data, "%Y-%m-%d") {
        let mut vec: Vec<Option<DateTime>> = vec![None; num_nulls];
        vec.push(Some(dt));
        CsvColumn::Time(vec, Some("%Y-%m-%d".to_string()))
    // } else if let Ok(d) = data.parse::<i64>() {
    //     let mut vec: Vec<Option<i64>> = vec![None; num_nulls];
    //     vec.push(Some(d));
    //     CsvColumn::I64(vec)
    } else if let Ok(d) = data.parse::<f64>() {
        let mut vec: Vec<f64> = vec![f64::NAN; num_nulls];
        vec.push(d);
        CsvColumn::F64(vec)
    } else {
        let mut vec: Vec<Option<String>> = vec![None; num_nulls];
        vec.push(Some(data.to_string()));
        CsvColumn::Str(vec)
    }
}

fn parse_column_data(data: &str, col: &mut CsvColumn) -> Result<(), CsvParseError> {
    match col {
        CsvColumn::F64(vec) => {
            if data.is_empty() {
                vec.push(f64::NAN);
            } else if let Ok(d) = data.parse::<f64>() {
                vec.push(d);
            } else {
                return Err(CsvParseError::ColType { line: 2 });
            }
        }
        CsvColumn::I64(vec) => {
            if data.is_empty() {
                vec.push(None);
            } else if let Ok(d) = data.parse::<i64>() {
                vec.push(Some(d));
            } else {
                return Err(CsvParseError::ColType {
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
                            return Err(CsvParseError::ColType {
                                line: col.len() + 2,
                            });
                        }
                    }
                    Some(fmt) => {
                        if let Ok(dt) = DateTime::fmt_parse(data, fmt) {
                            vec.push(Some(dt));
                        } else {
                            return Err(CsvParseError::ColType {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Source;

    pub const CSV_DATA: &str = "Int,Float,Str\n1,1.0,one\n2,2.0,two\n3,3.0,three\n";

    #[test]
    fn test_parse_csv_data() {
        let src = CsvParser::new().parse(CSV_DATA).unwrap();
        assert_eq!(src.len(), 3);

        let int_col = src
            .column("Int")
            .and_then(|c| c.f64())
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

        assert_eq!(int_col, &[1.0, 2.0, 3.0]);
        assert_eq!(float_col, &[1.0, 2.0, 3.0]);
        assert_eq!(str_col, &["one", "two", "three"]);
    }

    pub const CSV_NULL_DATA: &str = "Int,Float,Str\n1,1.0,one\n2,,two\n3,3.0,three\n";

    #[test]
    fn test_parse_csv_data_null() {
        let src = CsvParser::new().parse(CSV_NULL_DATA).unwrap();
        assert_eq!(src.len(), 3);

        let int_col = src
            .column("Int")
            .and_then(|c| c.f64())
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

        assert_eq!(int_col, &[1.0, 2.0, 3.0]);
        assert_eq!(float_col, &[Some(1.0), None, Some(3.0)]);
        assert_eq!(str_col, &["one", "two", "three"]);
    }

    pub const CSV_NULL_DATA_FST_LINE: &str = "Int,Float,Str\n1,,one\n2,2.0,two\n3,3.0,three\n";

    #[test]
    fn test_parse_csv_data_null_fst_line() {
        let src = CsvParser::new().parse(CSV_NULL_DATA_FST_LINE).unwrap();
        assert_eq!(src.len(), 3);

        let int_col = src
            .column("Int")
            .and_then(|c| c.f64())
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

        assert_eq!(int_col, &[1.0, 2.0, 3.0]);
        assert_eq!(float_col, &[None, Some(2.0), Some(3.0)]);
        assert_eq!(str_col, &["one", "two", "three"]);
    }

    pub const CSV_DATE: &str = "Date,Float\n2025-01-01,1.0\n2025-01-02,2.0\n2025-01-03,3.0\n";

    #[test]
    fn test_parse_csv_date() {
        let src = CsvParser::new().parse(CSV_DATE).unwrap();
        assert_eq!(src.len(), 3);

        let date_col = src
            .column("Date")
            .and_then(|c| c.time())
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
