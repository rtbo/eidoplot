//! Module that provides the "show" functionality using the `iced` GUI library
//! Requires the `iced` feature to be enabled

use eidoplot::ir::PlotIdx;
use eidoplot::style::CustomStyle;
use eidoplot::{drawing, geom};
use iced::Length;
use iced::widget::column;

use crate::figure::figure;
use crate::toolbar;

/// Trait to show figures in a window
pub trait Show {
    /// Show the figure in a GUI window.
    /// This function will block the calling thread until the window is closed.
    fn show(self, style: Option<CustomStyle>) -> iced::Result;
}

#[derive(Debug, Clone)]
enum Message {
    Toolbar(toolbar::Message),

    FigureMousePress(geom::Point),
    FigureMouseMove(geom::Point),
    FigureMouseRelease(geom::Point),

    Event(iced::event::Event),
}

#[derive(Debug, Clone, Default)]
enum Interaction {
    #[default]
    None,
    ZoomEnabled,
    ZoomDragging {
        plot: PlotIdx,
        start: geom::Point,
        end: geom::Point,
    },
}

struct ShowWindow {
    figure: drawing::Figure,
    style: Option<CustomStyle>,
    tb_state: toolbar::State,
    interaction: Interaction,
}

impl ShowWindow {
    fn new(figure: drawing::Figure, style: Option<CustomStyle>) -> Self {
        Self {
            figure,
            style,
            tb_state: Default::default(),
            interaction: Interaction::None,
        }
    }

    fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::Toolbar(msg) => match msg {
                toolbar::Message::Home => {
                    self.tb_state.at_home = true;
                    self.interaction = Interaction::None;
                }
                toolbar::Message::ZoomIn | toolbar::Message::ZoomOut => {
                    self.tb_state.at_home = false;
                    self.interaction = Interaction::ZoomEnabled;
                }
            },
            Message::FigureMouseMove(point) => {
                let hit = self.figure.hit_test(point);

                let status = hit
                    .as_ref()
                    .map(|h| format!("X = {} | Y = {}", h.x_coords, h.y_coords));
                self.tb_state.status = status;

                match (&mut self.interaction, &hit) {
                    (Interaction::ZoomDragging { plot, end, .. }, Some(hit)) => {
                        if *plot == hit.idx {
                            *end = point;
                        }
                    }
                    _ => {}
                }
            }
            Message::FigureMousePress(point) => {
                let hit = self.figure.hit_test_idx(point);
                match (&self.interaction, hit) {
                    (Interaction::ZoomEnabled, Some(plot)) => {
                        self.interaction = Interaction::ZoomDragging {
                            plot,
                            start: point,
                            end: point,
                        };
                    }
                    _ => {}
                }
            }
            Message::FigureMouseRelease(point) => {
                match &self.interaction {
                    Interaction::ZoomDragging { plot, .. } => {
                        let hit = self.figure.hit_test_idx(point);
                        if let Some(hit_plot) = hit {
                            if *plot == hit_plot {
                                // validate zoom here
                            }
                        }
                        self.interaction = Interaction::ZoomEnabled;
                    }
                    _ => {
                        self.interaction = Interaction::None;
                    }
                }
            }
            Message::Event(iced::event::Event::Mouse(ev)) => match ev {
                iced::mouse::Event::CursorLeft => {
                    self.tb_state.status = None;
                }
                _ => {}
            },
            Message::Event(iced::event::Event::Keyboard(ev)) => {
                use iced::keyboard::{self, key};
                match ev {
                    keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(key::Named::Escape),
                        ..
                    } => {
                        self.interaction = Interaction::None;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let fig = figure(&self.figure)
            .width(Length::Fill)
            .height(Length::Fill)
            .on_mouse_move(Message::FigureMouseMove)
            .on_mouse_press(Message::FigureMousePress)
            .on_mouse_release(Message::FigureMouseRelease);

        let fig = if let Some(style) = &self.style {
            fig.style(|_| style.clone())
        } else {
            fig
        };

        let fig = match &self.interaction {
            Interaction::ZoomDragging { start, end, .. } => fig.zoom_rect(*start, *end),
            _ => fig,
        };

        let toolbar = toolbar::view(&self.tb_state).map(Message::Toolbar);

        column![fig, toolbar].into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen().map(Message::Event)
    }
}

impl Show for drawing::Figure {
    fn show(self, style: Option<CustomStyle>) -> iced::Result {
        iced::application(
            move || {
                let fig = self.clone();
                let style = style.clone();
                (ShowWindow::new(fig, style), iced::Task::none())
            },
            ShowWindow::update,
            ShowWindow::view,
        )
        // subscribe to key events
        .subscription(ShowWindow::subscription)
        .run()
    }
}
