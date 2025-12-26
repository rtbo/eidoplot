use std::path;

use eidoplot::data::Source;
use eidoplot::{data, eplt};

mod common;

fn iris_csv_path() -> path::PathBuf {
    let iris_csv = path::Path::new(file!());
    let parent = iris_csv.parent().unwrap();
    parent.join("Iris.csv")
}

/// Returns a boolean mask where the column matches the given category
/// Returns None if the column is not string-like
fn category_mask<C>(column: &C, category: &str) -> Option<Vec<bool>>
where
    C: data::Column + ?Sized,
{
    let mask = column.str()?.str_iter().map(|v| v == Some(category)).collect();
    Some(mask)
}

/// Filters a numeric column by a boolean mask
/// Returns None if the column is not numeric and panics if the lengths do not match
fn filter_numeric_by_mask<C>(num_col: &C, mask: &[bool]) -> Option<data::VecColumn>
where
    C: data::Column + ?Sized,
{
    assert_eq!(num_col.len(), mask.len());

    let vec: Vec<f64> = num_col
        .f64()?
        .f64_iter()
        .zip(mask.iter())
        .filter_map(|(v, &m)| if m { Some(v) } else { None })
        .map(|v| v.unwrap_or(f64::NAN))
        .collect();
    Some(vec.into())
}

fn main() {
    let iris_csv = iris_csv_path();
    let csv_data = std::fs::read_to_string(&iris_csv).unwrap();

    let table = data::CsvParser::new().parse(&csv_data).unwrap();

    let species = table.column("Species").unwrap();
    let sepal_length = table.column("SepalLengthCm").unwrap();
    let petal_length = table.column("PetalLengthCm").unwrap();

    let setosa_mask = category_mask(species, "Iris-setosa").unwrap();
    let versicolor_mask = category_mask(species, "Iris-versicolor").unwrap();
    let virginica_mask = category_mask(species, "Iris-virginica").unwrap();

    let setosa_sepal_length = filter_numeric_by_mask(sepal_length, &setosa_mask).unwrap();
    let setosa_petal_length = filter_numeric_by_mask(petal_length, &setosa_mask).unwrap();

    let versicolor_sepal_length = filter_numeric_by_mask(sepal_length, &versicolor_mask).unwrap();
    let versicolor_petal_length = filter_numeric_by_mask(petal_length, &versicolor_mask).unwrap();

    let virginica_sepal_length = filter_numeric_by_mask(sepal_length, &virginica_mask).unwrap();
    let virginica_petal_length = filter_numeric_by_mask(petal_length, &virginica_mask).unwrap();

    let mut source = data::NamedColumns::new();

    source.add_column(
        "setosa_sepal_length",
        &setosa_sepal_length as &dyn data::Column,
    );
    source.add_column(
        "setosa_petal_length",
        &setosa_petal_length as &dyn data::Column,
    );

    source.add_column(
        "versicolor_sepal_length",
        &versicolor_sepal_length as &dyn data::Column,
    );
    source.add_column(
        "versicolor_petal_length",
        &versicolor_petal_length as &dyn data::Column,
    );

    source.add_column(
        "virginica_sepal_length",
        &virginica_sepal_length as &dyn data::Column,
    );
    source.add_column(
        "virginica_petal_length",
        &virginica_petal_length as &dyn data::Column,
    );

    let file_name = file!();
    let eplt_file = path::Path::new(file_name)
        .parent()
        .unwrap()
        .join("iris.eplt");
    let eplt = std::fs::read_to_string(&eplt_file).unwrap();

    let figs = eplt::parse_diag(&eplt, Some(&eplt_file)).unwrap();
    common::save_figure(&figs[0], &source, "iris_eplt");
}
