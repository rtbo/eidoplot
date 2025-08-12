use eidoplot::drawing::{self, FigureExt};
use eidoplot::{data, ir, style};
use eidoplot_pxl::PxlSurface;
use eidoplot_svg::SvgSurface;

use std::sync::Arc;
use std::{env, f64::consts::PI};

fn main() {
    let x: Vec<f64> = (0..=360).map(|t| t as f64 * PI / 180.0).collect();
    let y = x.iter().map(|x| x.sin()).collect();

    let data_source = data::VecMapSource::new()
        .with_col("x".into(), x)
        .with_col("y".into(), y);

    let title = ir::Text::new(
        "Sine wave",
        style::Font::new("serif".into(), 24.0)
            .with_style(style::font::Style::Italic)
            .with_weight(style::font::Weight::BOLD),
    );

    let x_axis = ir::Axis::new(ir::axis::Scale::default())
        .with_label("x".into())
        .with_ticks(ir::axis::ticks::Locator::PiMultiple { bins: 8 }.into());
    let y_axis = ir::Axis::new(ir::axis::Scale::default()).with_label("y".into());

    let series = ir::Series {
        name: Some("y=sin(x)".into()),
        plot: ir::SeriesPlot::Xy(ir::series::Xy {
            line: style::Line {
                color: style::color::BLUE,
                width: 3.0,
                pattern: style::LinePattern::Solid,
            },
            data: data::Xy::Src("x".to_string(), "y".to_string()),
        }),
    };

    let plot = ir::Plot {
        title: None,
        x_axis,
        y_axis,
        series: vec![series],
        legend: Some(ir::Legend::default().with_font(
            style::Font::new("Noto Sans Math".into(), 16.0).with_style(style::font::Style::Italic),
        )),
        ..ir::Plot::default()
    };

    let fig = ir::Figure::new(ir::figure::Plots::Plot(plot)).with_title(Some(title));

    save_figure(&fig, &data_source);
}

fn save_figure<D>(fig: &ir::Figure, data_source: &D) 
where D: data::Source 
{
    let fontdb = eidoplot::bundled_font_db();

    let mut written = false;
    for arg in env::args() {
        if arg == "png" {
            write_png(&fig, data_source, fontdb.clone());
            written = true;
        } else if arg == "svg" {
            write_svg(&fig, data_source);
            written = true;
        }
    }
    if !written {
        write_png(&fig, data_source, fontdb);
    }
}

fn write_svg<D>(fig: &ir::Figure, data_source: &D)
where D: data::Source {
    let mut svg = SvgSurface::new(800, 600);
    fig.draw(&mut svg, data_source, drawing::Options::default()).unwrap();
    svg.save("plot.svg").unwrap();
}

fn write_png<D>(fig: &ir::Figure, data_source: &D, fontdb: Arc<fontdb::Database>)
where D: data::Source {
    let mut pxl = PxlSurface::new(1600, 1200);
    fig.draw(
        &mut pxl,
        data_source,
        drawing::Options {
            fontdb: Some(fontdb.clone()),
        },
    )
    .unwrap();
    pxl.save("plot.png", Some(fontdb)).unwrap();
}
