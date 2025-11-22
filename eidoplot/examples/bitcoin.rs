use eidoplot::ir;

mod common;

fn main() {
    // FIXME: support parsing datetime in CSV
    let btc_csv = common::example_res("BTC-USD.csv");
    let csv_data = std::fs::read_to_string(&btc_csv).unwrap();
    let data_source = eidoplot_utils::CsvParser::new().parse(&csv_data).unwrap();

    let price_series = ir::series::Line::new(
        ir::DataCol::SrcRef("Date".to_string()),
        ir::DataCol::SrcRef("Close".to_string()),
    )
    .with_name("Closing Price".to_string())
    .into();

    let volume_series = ir::series::Line::new(
        ir::DataCol::SrcRef("Date".to_string()),
        ir::DataCol::SrcRef("Volume".to_string()),
    )
    .with_name("Volume".to_string())
    .with_y_axis(ir::axis::Ref::Id("volume".to_string()))
    .into();

    let date_axis = ir::Axis::new().with_ticks(Default::default());
    let price_axis = ir::Axis::new()
        .with_title("Price [USD]".to_string().into())
        .with_ticks(Default::default());
    let volume_axis = ir::Axis::new()
        .with_title("Volume [USD]".to_string().into())
        .with_ticks(Default::default())
        .with_id("volume".to_string().into())
        .with_opposite_side();
    let plot = ir::Plot::new(vec![price_series, volume_series])
        .with_x_axis(date_axis)
        .with_y_axis(price_axis)
        .with_y_axis(volume_axis)
        .with_legend(ir::plot::LegendPos::InTopLeft.into());
    let fig = ir::Figure::new(plot.into()).with_title("Bitcoin historical data".to_string().into());

    common::save_figure(&fig, &data_source, "bitcoin");
}
