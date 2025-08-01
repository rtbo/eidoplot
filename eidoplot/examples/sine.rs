use eidoplot::prelude::*;
use eidoplot_svg::SvgSurface;

use std::f64::consts::PI;

fn main() {
    let fig = Figure {
        size: FigSize::default(),
        title: Some("Sine wave".into()),
        fill: Some(css::ANTIQUEWHITE.into()),
        plots: Plots::Plot(Plot {
            title: None,
            fill: Some(css::ALICEBLUE.into()),
            desc: PlotDesc::Curves(Curves {
                x_axis: Axis {
                    name: "x".into(),
                    range: axis::Range::Auto,
                    scale: axis::Scale::Linear,
                    ticks: Some(TickLocator::PiMultiple { num: 1.0, den: 2.0 }),
                    ticks_min: None,
                },
                y_axis: Axis {
                    name: "y".into(),
                    range: axis::Range::Auto,
                    scale: axis::Scale::Linear,
                    ticks: Some(TickLocator::Auto),
                    ticks_min: None,
                },
                series: vec![XySeries {
                    name: "y=sin(x)".into(),
                    line_style: style::Line {
                        color: css::FUCHSIA,
                        width: 2.0,
                        pattern: LinePattern::Solid,
                    },
                    points: (0..=360)
                        .map(|x| (x as f64 / 180.0 * PI, (x as f64 / 180.0 * PI).sin()))
                        .collect(),
                }],
            }),
        }),
    };

    let mut svg = SvgSurface::new(1200, 900);
    fig.draw(&mut svg).unwrap();
    svg.save("sine.svg").unwrap();
}
