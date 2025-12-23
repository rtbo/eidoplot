//! Module that provides the "show" functionality using the `iced` GUI library
//! Requires the `iced` feature to be enabled

use eidoplot::drawing;
use eidoplot::style::CustomStyle;
use iced::Length;

use crate::figure::figure;

/// Trait to show figures in a window
pub trait Show {
    /// Show the figure in a GUI window.
    /// This function will block the calling thread until the window is closed.
    fn show(self, style: Option<CustomStyle>) -> iced::Result;
}

enum Message {}

struct FigureWindow {
    figure: drawing::Figure,
    style: Option<CustomStyle>,
}

impl FigureWindow {
    fn new(figure: drawing::Figure, style: Option<CustomStyle>) -> Self {
        Self { figure, style }
    }

    fn update(&mut self, _msg: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let fig = figure(&self.figure)
            .width(Length::Fill)
            .height(Length::Fill);
        if let Some(style) = &self.style {
            fig.style(|_| style.clone())
        } else {
            fig
        }
        .into()
    }
}

impl Show for drawing::Figure {
    fn show(self, style: Option<CustomStyle>) -> iced::Result {
        iced::application(
            move || {
                let fig = self.clone();
                let style = style.clone();
                (FigureWindow::new(fig, style), iced::Task::none())
            },
            FigureWindow::update,
            FigureWindow::view,
        )
        .run()
    }
}
