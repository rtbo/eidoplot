use std::path;

use eidoplot::data::Source;
use eidoplot::{data, ir};

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

    let title: ir::figure::Title = "Iris dataset".into();

    let x_axis = ir::Axis::new()
        .with_title("Sepal Length [cm]".into())
        .with_ticks(Default::default())
        .with_grid(Default::default());
    let y_axis = ir::Axis::new()
        .with_title("Petal Length [cm]".into())
        .with_ticks(Default::default())
        .with_grid(Default::default());

    let setosa = ir::Series::Scatter(
        ir::series::Scatter::new(
            ir::data_src_ref("setosa_sepal_length"),
            ir::data_src_ref("setosa_petal_length"),
        )
        .with_name("Setosa"),
    );
    let virginica = ir::Series::Scatter(
        ir::series::Scatter::new(
            ir::data_src_ref("virginica_sepal_length"),
            ir::data_src_ref("virginica_petal_length"),
        )
        .with_name("Virginica"),
    );
    let versicolor = ir::Series::Scatter(
        ir::series::Scatter::new(
            ir::data_src_ref("versicolor_sepal_length"),
            ir::data_src_ref("versicolor_petal_length"),
        )
        .with_name("Versicolor"),
    );

    let plot = ir::Plot::new(vec![setosa, versicolor, virginica])
        .with_x_axis(x_axis)
        .with_y_axis(y_axis)
        .with_legend(ir::plot::LegendPos::InBottomRight.into());

    let fig = ir::Figure::new(plot.into()).with_title(title);

    common::save_figure(&fig, &source, "iris");
}
