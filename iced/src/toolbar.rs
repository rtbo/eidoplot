use iced::{
    Alignment, Length, widget::{button, row, space}
};

#[derive(Debug, Clone)]
pub enum Message {
    Home,
    ZoomIn,
    ZoomOut,
}

#[derive(Debug, Clone)]
pub struct State {
    pub at_home: bool,
    pub status: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            at_home: true,
            status: None,
        }
    }
}

pub fn view(state: &State) -> iced::Element<'_, Message> {
    let home_button = button("Home").on_press_maybe((!state.at_home).then_some(Message::Home));
    let zoom_in_button = button("Zoom In").on_press(Message::ZoomIn);
    let zoom_out_button =
        button("Zoom Out").on_press_maybe((!state.at_home).then_some(Message::ZoomOut));

    let status_txt = state.status.as_deref().unwrap_or("");
    let status_txt = iced::widget::text(status_txt)
        .height(Length::Fill)
        .align_y(Alignment::Center);

    row![
        home_button,
        zoom_in_button,
        zoom_out_button,
        space::horizontal(),
        status_txt
    ]
    .width(Length::Fill)
    .height(Length::Shrink)
    .spacing(10)
    .padding(5)
    .into()
}
