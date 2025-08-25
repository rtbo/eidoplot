use std::path;

use eidoplot::ir;
use eidoplot::data;
use eidoplot::style;

use eidoplot::style::palette;
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

    let title = "Iris dataset".into();

    let x_axis = ir::Axis::new(ir::axis::Scale::default())
        .with_label("Sepal Length [cm]".into());
    let y_axis = ir::Axis::new(ir::axis::Scale::default())
        .with_label("Petal Length [cm]".into());

    let setosa = ir::Series {
        name: Some("Setosa".into()),
        plot: ir::SeriesPlot::Scatter(ir::series::Scatter {
            marker: style::Marker {
                shape: Default::default(),
                size: Default::default(),
                fill: Some(style::Fill::Solid(style::Color::Palette(palette::Color(0)))),
                stroke: None,
            },
            x_data: ir::series::DataCol::SrcRef("setosa_sepal_length".to_string()),
            y_data: ir::series::DataCol::SrcRef("setosa_petal_length".to_string()),
        }),
    };
    let versicolor = ir::Series {
        name: Some("Versicolor".into()),
        plot: ir::SeriesPlot::Scatter(ir::series::Scatter {
            marker: style::Marker {
                shape: Default::default(),
                size: Default::default(),
                fill: Some(style::Fill::Solid(style::Color::Palette(palette::Color(1)))),
                stroke: None,
            },
            x_data: ir::series::DataCol::SrcRef("versicolor_sepal_length".to_string()),
            y_data: ir::series::DataCol::SrcRef("versicolor_petal_length".to_string()),
        }),
    };
    let virginica = ir::Series {
        name: Some("Virginica".into()),
        plot: ir::SeriesPlot::Scatter(ir::series::Scatter {
            marker: style::Marker {
                shape: Default::default(),
                size: Default::default(),
                fill: Some(style::Fill::Solid(style::Color::Palette(palette::Color(2)))),
                stroke: None,
            },
            x_data: ir::series::DataCol::SrcRef("virginica_sepal_length".to_string()),
            y_data: ir::series::DataCol::SrcRef("virginica_petal_length".to_string()),
        }),
    };

    let plot = ir::Plot {
        title: None,
        x_axis,
        y_axis,
        series: vec![setosa, versicolor, virginica],
        legend: Some(ir::plot::LegendPos::InBottomRight.into()),
        ..ir::Plot::default()
    };

    let fig = ir::Figure::new(ir::figure::Plots::Plot(plot)).with_title(Some(title));

    common::save_figure(&fig, &source);
}
