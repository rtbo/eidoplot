use iced::{
    Alignment, Length, widget::{button, row, space}
};

#[derive(Debug, Clone)]
pub enum Message {
    Home,
    Zoom,
}

#[derive(Debug, Clone)]
pub struct State {
    pub at_home: bool,
    pub zooming: bool,
    pub status: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            at_home: true,
            zooming: false,
            status: None,
        }
    }
}

pub fn view(state: &State) -> iced::Element<'_, Message> {
    let home_button = button("Home").on_press_maybe((!state.at_home).then_some(Message::Home));
    let zoom_in_button = button("Zoom").on_press(Message::Zoom);
    let zoom_in_button = if state.zooming {
        zoom_in_button.style(button::secondary)
    } else {
        zoom_in_button.style(button::primary)
    };

    let status_txt = state.status.as_deref().unwrap_or("");
    let status_txt = iced::widget::text(status_txt)
        .height(Length::Fill)
        .align_y(Alignment::Center);

    row![
        home_button,
        zoom_in_button,
        space::horizontal(),
        status_txt
    ]
    .width(Length::Fill)
    .height(Length::Shrink)
    .spacing(10)
    .padding(5)
    .into()
}
