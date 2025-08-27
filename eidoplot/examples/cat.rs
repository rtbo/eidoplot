use eidoplot::{
    data, ir,
    style,
};

mod common;

fn main() {
    let fruits = data::VecColumn::from(vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
        "date".to_string(),
        "lime".to_string(),
    ]);

    let stock = data::VecColumn::from(vec![50, 30, 20, 35, 5]);

    let mut source = data::NamedColumns::new();
    source.add_column("fruits", &fruits);
    source.add_column("stock", &stock);

    let title = "Categorical scatter".into();

    let x_axis = ir::Axis::new(ir::axis::Scale::default()).with_title("Fruits".into());
    let y_axis = ir::Axis::new(ir::axis::Scale::default()).with_title("Stock".into());

    let stock = ir::Series {
        name: Some("Stock".into()),
        plot: ir::SeriesPlot::Scatter(ir::series::Scatter {
            marker: style::Marker {
                shape: Default::default(),
                size: Default::default(),
                fill: Some(style::series::Color::Auto.into()),
                stroke: None,
            },
            x_data: ir::series::DataCol::SrcRef("fruits".to_string()),
            y_data: ir::series::DataCol::SrcRef("stock".to_string()),
        }),
    };

    let plot = ir::Plot {
        title: None,
        x_axis,
        y_axis,
        series: vec![stock],
        legend: None,
        ..ir::Plot::default()
    };

    let fig = ir::Figure::new(ir::figure::Plots::Plot(plot)).with_title(Some(title));

    common::save_figure(&fig, &source);
}
