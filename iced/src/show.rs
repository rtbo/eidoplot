//! Module that provides the "show" functionality using the `iced` GUI library
//! Requires the `iced` feature to be enabled

use eidoplot::drawing;
use eidoplot::style::CustomStyle;
use iced::Length;
use iced::widget::column;

use crate::figure::figure;
use crate::toolbar::{self, Toolbar};

/// Trait to show figures in a window
pub trait Show {
    /// Show the figure in a GUI window.
    /// This function will block the calling thread until the window is closed.
    fn show(self, style: Option<CustomStyle>) -> iced::Result;
}

#[derive(Debug, Clone)]
enum Message {
    Toolbar(toolbar::Message),
    PlotHover(Option<drawing::PlotHit>),
}

struct ShowWindow {
    figure: drawing::Figure,
    style: Option<CustomStyle>,
    toolbar: Toolbar,
}

impl ShowWindow {
    fn new(figure: drawing::Figure, style: Option<CustomStyle>) -> Self {
        Self {
            figure,
            style,
            toolbar: Toolbar::default(),
        }
    }

    fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::Toolbar(msg) => {
                match msg {
                    toolbar::Message::Home => {
                        // TODO: Reset the figure view to home
                        self.toolbar.set_at_home(true);
                    }
                    toolbar::Message::ZoomIn | toolbar::Message::ZoomOut => {
                        // TODO: Apply zoom to the figure view
                        self.toolbar.set_at_home(false);
                    }
                }
            }
            Message::PlotHover(hit) => {
                if let Some(hit) = hit {
                    let status = format!("X = {} | Y = {}", &hit.x_coords, &hit.y_coords);
                    self.toolbar.set_status(Some(status));
                } else {
                    self.toolbar.set_status(None);
                }
            }
        }
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let fig = figure(&self.figure)
            .width(Length::Fill)
            .height(Length::Fill)
            .on_hover_hit(Message::PlotHover);

        let fig = if let Some(style) = &self.style {
            fig.style(|_| style.clone())
        } else {
            fig
        };

        let toolbar = self.toolbar.view().map(Message::Toolbar);

        column![fig, toolbar].into()
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
        .run()
    }
}
