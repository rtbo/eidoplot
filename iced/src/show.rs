//! Module that provides the "show" functionality using the `iced` GUI library
//! Requires the `iced` feature to be enabled

use eidoplot::drawing;
use iced::Length;

use crate::figure::figure;

/// Trait to show figures in a window
pub trait Show {
    /// Show the figure in a GUI window.
    /// This function will block the calling thread until the window is closed.
    fn show(self) -> iced::Result;
}

enum Message {}

struct FigureWindow {
    figure: drawing::Figure,
}

impl FigureWindow {
    fn new(figure: drawing::Figure) -> Self {
        Self { figure }
    }

    fn update(&mut self, _msg: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        figure(&self.figure)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Show for drawing::Figure {
    fn show(self) -> iced::Result {
        iced::application(
            move || {
                let fig = self.clone();
                (FigureWindow::new(fig), iced::Task::none())
            },
            FigureWindow::update,
            FigureWindow::view,
        )
        .run()
    }
}
