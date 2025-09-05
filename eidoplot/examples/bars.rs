use eidoplot::{data, ir};

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

    let title = ir::figure::Title::new("Categorical bars".into());

    let x_axis = ir::Axis::new().with_title("Fruits".to_string().into());
    let y_axis = ir::Axis::new().with_title("Stocks".to_string().into());

    // let bars_group: ir::Series = ir::series::BarsGroup::new(
    //     ir::DataCol::SrcRef("fruits".to_string()),
    //     vec![
    //         ir::series::BarSeries::new(
    //             Some("Stocks 2023".into()),
    //             ir::DataCol::SrcRef("stocks_2023".to_string()),
    //         ),
    //         ir::series::BarSeries::new(
    //             Some("Stocks 2024".into()),
    //             ir::DataCol::SrcRef("stocks_2024".to_string()),
    //         ),
    //         ir::series::BarSeries::new(
    //             Some("Stocks 2025".into()),
    //             ir::DataCol::SrcRef("stocks_2025".to_string()),
    //         ),
    //     ],
    // )
    // .into();

    // let series = vec![bars_group];

    let stocks_2023 = ir::Series::Bars(
        ir::series::Bars::new(
            ir::DataCol::SrcRef("fruits".to_string()),
            ir::DataCol::SrcRef("stocks_2023".to_string()),
        )
        .with_name("Stocks 2023".into())
        .with_position(ir::series::BarsPosition {
            offset: 0.2,
            width: 0.2,
        }),
    );
    let stocks_2024 = ir::Series::Bars(
        ir::series::Bars::new(
            ir::DataCol::SrcRef("fruits".to_string()),
            ir::DataCol::SrcRef("stocks_2024".to_string()),
        )
        .with_name("Stocks 2024".into())
        .with_position(ir::series::BarsPosition {
            offset: 0.2,
            width: 0.2,
        }),
    );
    let stocks_2025 = ir::Series::Bars(
        ir::series::Bars::new(
            ir::DataCol::SrcRef("fruits".to_string()),
            ir::DataCol::SrcRef("stocks_2025".to_string()),
        )
        .with_name("Stocks 2025".into())
        .with_position(ir::series::BarsPosition {
            offset: 0.2,
            width: 0.2,
        }),
    );

    let series = vec![stocks_2023, stocks_2024, stocks_2025];

    let plot = ir::Plot::new(series)
        .with_x_axis(x_axis)
        .with_y_axis(y_axis);

    let fig = ir::Figure::new(ir::figure::Plots::Plot(plot)).with_title(title);

    common::save_figure(&fig, &source);
}
