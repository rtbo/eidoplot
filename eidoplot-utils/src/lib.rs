use eidoplot::data::{VecColumn, TableSource};

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
