use plotive::{data, des};

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

    let x_axis = des::Axis::new()
        .with_title("Fruits".into())
        .with_ticks(Default::default());
    let y_axis = des::Axis::new()
        .with_title("Stocks".into())
        .with_ticks(Default::default());

    // let bars_group: ir::Series = ir::series::BarsGroup::new(
    //     ir::data_src_ref("fruits"),
    //     vec![
    //         ir::series::BarSeries::new(
    //             Some("Stocks 2023".into()),
    //             ir::data_src_ref("stocks_2023"),
    //         ),
    //         ir::series::BarSeries::new(
    //             Some("Stocks 2024".into()),
    //             ir::data_src_ref("stocks_2024"),
    //         ),
    //         ir::series::BarSeries::new(
    //             Some("Stocks 2025".into()),
    //             ir::data_src_ref("stocks_2025"),
    //         ),
    //     ],
    // )
    // .into();

    // let series = vec![bars_group];

    let stocks_2023 = des::Series::Bars(
        des::series::Bars::new(des::data_src_ref("fruits"), des::data_src_ref("stocks_2023"))
            .with_name("Stocks 2023")
            .with_position(des::series::BarsPosition {
                offset: 0.2,
                width: 0.2,
            }),
    );
    let stocks_2024 = des::Series::Bars(
        des::series::Bars::new(des::data_src_ref("fruits"), des::data_src_ref("stocks_2024"))
            .with_name("Stocks 2024")
            .with_position(des::series::BarsPosition {
                offset: 0.4,
                width: 0.2,
            }),
    );
    let stocks_2025 = des::Series::Bars(
        des::series::Bars::new(des::data_src_ref("fruits"), des::data_src_ref("stocks_2025"))
            .with_name("Stocks 2025")
            .with_position(des::series::BarsPosition {
                offset: 0.6,
                width: 0.2,
            }),
    );

    let series = vec![stocks_2023, stocks_2024, stocks_2025];

    let plot = des::Plot::new(series)
        .with_x_axis(x_axis)
        .with_y_axis(y_axis)
        .with_legend(Default::default());

    let fig = des::Figure::new(plot.into()).with_title("Categorial bars".into());

    common::save_figure(&fig, &source, None, "bars");
}
