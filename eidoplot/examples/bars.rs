use eidoplot::{
    data, ir,
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

    let stocks_2023 = data::VecColumn::from(vec![50, 30, 20, 35, 5]);
    let stocks_2024 = data::VecColumn::from(vec![60, 40, 25, 45, 10]);
    let stocks_2025 = data::VecColumn::from(vec![55, 50, 20, 40, 10]);

    let mut source = data::NamedColumns::new();
    source.add_column("fruits", &fruits);
    source.add_column("stocks_2023", &stocks_2023);
    source.add_column("stocks_2024", &stocks_2024);
    source.add_column("stocks_2025", &stocks_2025);

    let title = "Categorical bars".into();

    let x_axis = ir::Axis::new(ir::axis::Scale::default()).with_title("Fruits".into());
    let y_axis = ir::Axis::new(ir::axis::Scale::default()).with_title("Stock".into());

    let stocks_2023 = ir::Series {
        name: Some("Stocks 2023".into()),
        plot: ir::SeriesPlot::Bars(ir::series::Bars {
            fill: Default::default(),
            line: None,
            position: ir::series::BarPosition {
                offset: 0.2,
                width: 0.2,
            },
            x_data: ir::series::DataCol::SrcRef("fruits".to_string()),
            y_data: ir::series::DataCol::SrcRef("stocks_2023".to_string()),
        }),
    };
    let stocks_2024 = ir::Series {
        name: Some("Stocks 2024".into()),
        plot: ir::SeriesPlot::Bars(ir::series::Bars {
            fill: Default::default(),
            line: None,
            position: ir::series::BarPosition {
                offset: 0.4,
                width: 0.2,
            },
            x_data: ir::series::DataCol::SrcRef("fruits".to_string()),
            y_data: ir::series::DataCol::SrcRef("stocks_2024".to_string()),
        }),
    };
    let stocks_2025 = ir::Series {
        name: Some("Stocks 2025".into()),
        plot: ir::SeriesPlot::Bars(ir::series::Bars {
            fill: Default::default(),
            line: None,
            position: ir::series::BarPosition {
                offset: 0.6,
                width: 0.2,
            },
            x_data: ir::series::DataCol::SrcRef("fruits".to_string()),
            y_data: ir::series::DataCol::SrcRef("stocks_2025".to_string()),
        }),
    };

    let plot = ir::Plot {
        title: None,
        x_axis,
        y_axis,
        series: vec![stocks_2023, stocks_2024, stocks_2025],
        ..ir::Plot::default()
    };

    let fig = ir::Figure::new(ir::figure::Plots::Plot(plot)).with_title(Some(title));

    common::save_figure(&fig, &source);
}
