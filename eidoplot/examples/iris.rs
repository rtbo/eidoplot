use std::path;

use eidoplot::data;
use eidoplot::data::Source;
use eidoplot::ir;
use eidoplot::style;

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
    let mask = column.str()?.iter().map(|v| v == Some(category)).collect();
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
        .iter()
        .zip(mask.iter())
        .filter_map(|(v, &m)| if m { Some(v) } else { None })
        .map(|v| v.unwrap_or(f64::NAN))
        .collect();
    Some(vec.into())
}

fn main() {
    let iris_csv = iris_csv_path();
    let csv_data = std::fs::read_to_string(&iris_csv).unwrap();

    let table = eidoplot_utils::parse_csv_data(&csv_data, ',').unwrap();

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

    let title = "Iris dataset".into();

    let x_axis = ir::Axis::new(ir::axis::Scale::default()).with_label("Sepal Length [cm]".into());
    let y_axis = ir::Axis::new(ir::axis::Scale::default()).with_label("Petal Length [cm]".into());

    let setosa = ir::Series {
        name: Some("Setosa".into()),
        plot: ir::SeriesPlot::Scatter(ir::series::Scatter {
            marker: style::Marker {
                shape: Default::default(),
                size: Default::default(),
                fill: Some(style::series::Color::Auto.into()),
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
                fill: Some(style::series::Color::Auto.into()),
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
                fill: Some(style::series::Color::Auto.into()),
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
