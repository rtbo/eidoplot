//! Module that provides the "show" functionality using the `iced` GUI library
//! Requires the `iced` feature to be enabled

use eidoplot::{drawing, style::Theme};
use iced::Length;

use crate::figure::{self, figure};

/// Trait to show figures in a window
pub trait Show {
    /// Show the figure in a GUI window.
    /// This function will block the calling thread until the window is closed.
    fn show(self, theme: Option<Theme>) -> iced::Result;
}

enum Message {}

struct FigureWindow {
    figure: drawing::Figure,
    theme: Option<Theme>,
}

impl FigureWindow {
    fn new(figure: drawing::Figure, theme: Option<Theme>) -> Self {
        Self { figure, theme }
    }

    fn update(&mut self, _msg: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        figure(&self.figure)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| figure::Style { theme: self.theme.clone() })
            .into()
    }
}

impl Show for drawing::Figure {
    fn show(self, theme: Option<Theme>) -> iced::Result {
        iced::application(
            move || {
                let fig = self.clone();
                let theme = theme.clone();
                (FigureWindow::new(fig, theme), iced::Task::none())
            },
            FigureWindow::update,
            FigureWindow::view,
        )
        .run()
    }
}
