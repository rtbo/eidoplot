use core::f32;
use std::f64::consts::PI;
use std::time::Instant;

use eidoplot::Drawing;
use eidoplot::style::theme;
use iced::widget::{column, text};
use iced::{Element, Subscription, Task, window};

use eidoplot::{data, drawing, ir, style, utils};
use iced_oplot::figure::{self, figure};

#[derive(Debug, Clone)]
enum Message {
    Frame(Instant),
}

#[derive(Debug)]
struct FpsCounter {
    t0: Instant,
    frames: u32,
    update_every_secs: f32,
}

impl FpsCounter {
    fn new(update_every_secs: f32) -> Self {
        Self {
            t0: Instant::now(),
            frames: 0,
            update_every_secs,
        }
    }

    fn tick(&mut self) -> Option<f32> {
        self.frames += 1;
        let elapsed = self.t0.elapsed().as_secs_f32();
        if elapsed >= self.update_every_secs {
            let fps = self.frames as f32 / elapsed;
            self.t0 = Instant::now();
            self.frames = 0;
            Some(fps)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct Scope {
    t0: Instant,
    fps: f32,
    fps_counter: FpsCounter,
    phase: f64,
    fig: drawing::Figure,
    x: Vec<f64>,
    y: Vec<f64>,
}

impl Default for Scope {
    fn default() -> Self {
        // initial buffer of 512 samples
        let x: Vec<f64> = utils::linspace(0.0, 2.0 * PI, 512);
        let y: Vec<f64> = x.iter().map(|x| x.sin()).collect();
        let mut data_src = data::NamedColumns::new();
        data_src.add_column("x", &x);
        data_src.add_column("y", &y);
        let fig = build_figure().prepare(&data_src, None).unwrap();
        Self {
            t0: Instant::now(),
            fps: f32::NAN,
            fps_counter: FpsCounter::new(0.5),
            phase: 0.0,
            fig,
            x,
            y,
        }
    }
}

impl Scope {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Frame(now) => {
                let dt = now.duration_since(self.t0).as_secs_f64();
                self.t0 = now;
                if let Some(fps) = self.fps_counter.tick() {
                    self.fps = fps;
                }

                // scroll signal by a small phase increment based on time
                self.phase += (dt * 2.0 * PI * 0.5) as f64; // 0.5 Hz
                for (x, y) in self.x.iter().zip(self.y.iter_mut()) {
                    *y = (x + self.phase).sin();
                }

                let mut data_src = data::NamedColumns::new();
                data_src.add_column("x", &self.x);
                data_src.add_column("y", &self.y);
                self.fig.update_series_data(&data_src).unwrap();

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let fps_text = if self.fps.is_finite() {
            text(format!("FPS: {:.1}", self.fps))
        } else {
            text("FPS: ...")
        }
        .size(16);

        let fig = figure(&self.fig)
            .scale(1.5)
            .style(|theme: &iced::Theme| {
                let is_dark = theme.palette().background.relative_luminance() < 0.5;

                let theme = if is_dark {
                    eidoplot::style::catppuccin::macchiato()
                } else {
                    eidoplot::style::theme::standard_light()
                };
                figure::Style { theme: Some(theme) }
            });

        column![fps_text, fig].into()
    }
}

fn build_figure() -> ir::Figure {
    let x_axis = ir::Axis::new()
        .with_scale(ir::axis::Range::MinMax(0.0, 2.0 * PI).into())
        .with_title("x".to_string().into())
        .with_ticks(ir::axis::ticks::Locator::PiMultiple { bins: 5 }.into());
    let y_axis = ir::Axis::new()
        .with_title("y".to_string().into())
        .with_ticks(Default::default())
        .with_grid(Default::default());

    let series = ir::Series::Line(
        ir::series::Line::new(
            ir::DataCol::SrcRef("x".to_string()),
            ir::DataCol::SrcRef("y".to_string()),
        )
        .with_name("y=sin(x)".to_string())
        .with_line(style::series::Line::default().with_width(2.0)),
    );

    let plot = ir::Plot::new(vec![series])
        .with_x_axis(x_axis)
        .with_y_axis(y_axis)
        .with_fill(theme::Col::Background.into())
        .with_legend(ir::plot::LegendPos::InTopRight.into());

    ir::Figure::new(plot.into())
        .with_title("Real-time plot".to_string().into())
        .with_fill(None)
}

fn main() -> iced::Result {
    iced::application(Scope::default, Scope::update, Scope::view)
        .title("Iced-Oplot: Real-time")
        .subscription(|_| Subscription::batch([window::frames().map(Message::Frame)]))
        .antialiasing(true)
        .run()
}
