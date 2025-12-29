use std::path;

use eidoplot::{data, ir};
use polars::prelude::*;

mod common;

fn iris_csv_path() -> path::PathBuf {
    let iris_csv = path::Path::new(file!());
    let parent = iris_csv.parent().unwrap();
    parent.join("Iris.csv")
}

fn main() {
    let iris_csv = iris_csv_path();

    let df = CsvReadOptions::default()
        .try_into_reader_with_file_path(Some(iris_csv))
        .unwrap()
        .finish()
        .unwrap();

    let setosa = df
        .clone()
        .lazy()
        .filter(col("Species").eq(lit("Iris-setosa")))
        .collect()
        .unwrap();
    let versicolor = df
        .clone()
        .lazy()
        .filter(col("Species").eq(lit("Iris-versicolor")))
        .collect()
        .unwrap();
    let virginica = df
        .lazy()
        .filter(col("Species").eq(lit("Iris-virginica")))
        .collect()
        .unwrap();

    let mut source = data::NamedColumns::new();

    source.add_column(
        "setosa_sepal_length",
        setosa
            .column("SepalLengthCm")
            .unwrap()
            .as_materialized_series(),
    );
    source.add_column(
        "setosa_petal_length",
        setosa
            .column("PetalLengthCm")
            .unwrap()
            .as_materialized_series(),
    );

    source.add_column(
        "versicolor_sepal_length",
        versicolor
            .column("SepalLengthCm")
            .unwrap()
            .as_materialized_series(),
    );
    source.add_column(
        "versicolor_petal_length",
        versicolor
            .column("PetalLengthCm")
            .unwrap()
            .as_materialized_series(),
    );

    source.add_column(
        "virginica_sepal_length",
        virginica
            .column("SepalLengthCm")
            .unwrap()
            .as_materialized_series(),
    );
    source.add_column(
        "virginica_petal_length",
        virginica
            .column("PetalLengthCm")
            .unwrap()
            .as_materialized_series(),
    );

    let title: ir::figure::Title = "Iris dataset".into();

    let x_axis = ir::Axis::new()
        .with_title("Sepal Length [cm]".into())
        .with_ticks(Default::default());
    let y_axis = ir::Axis::new()
        .with_title("Petal Length [cm]".into())
        .with_ticks(Default::default());

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

    common::save_figure(&fig, &source, None, "polars-iris");
}
