use plotive::{data, des};

mod common;

fn main() {
    // FIXME: support parsing datetime in CSV
    let btc_csv = common::example_res("BTC-USD.csv");
    let csv_data = std::fs::read_to_string(&btc_csv).unwrap();
    let data_source = data::CsvParser::new().parse(&csv_data).unwrap();

    let price_series = des::series::Line::new(des::data_src_ref("Date"), des::data_src_ref("Close"))
        .with_name("Closing Price")
        .into();

    let volume_series = des::series::Line::new(des::data_src_ref("Date"), des::data_src_ref("Volume"))
        .with_name("Volume")
        .with_y_axis(des::axis::ref_id("volume"))
        .into();

    let date_axis = des::Axis::new().with_ticks(Default::default());
    // setting Y-ranges to have ticks at same level
    // this will line-up the grid lines
    let price_axis = des::Axis::new()
        .with_title("Price [USD]".into())
        .with_scale(des::axis::Range::MinMax(0.0, 8e4).into())
        .with_ticks(Default::default())
        .with_grid(Default::default());
    let volume_axis = des::Axis::new()
        .with_title("Volume [USD]".into())
        .with_scale(des::axis::Range::MinMax(0.0, 4e11).into())
        .with_ticks(Default::default())
        .with_id("volume")
        .with_opposite_side();
    let plot = des::Plot::new(vec![price_series, volume_series])
        .with_x_axis(date_axis)
        .with_y_axis(price_axis)
        .with_y_axis(volume_axis)
        .with_legend(des::plot::LegendPos::InTopLeft.into());
    let fig = des::Figure::new(plot.into()).with_title("Bitcoin historical data".into());

    common::save_figure(&fig, &data_source, None, "bitcoin");
}
