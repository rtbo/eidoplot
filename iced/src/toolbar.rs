use iced::{
    Length,
    widget::{button, row},
};

#[derive(Debug, Clone)]
pub enum Message {
    Home,
    ZoomIn,
    ZoomOut,
}

pub struct Toolbar {
    at_home: bool,
}

impl Toolbar {
    pub fn new(at_home: bool) -> Self {
        Self {
            at_home,
        }
    }

    pub fn set_at_home(&mut self, at_home: bool) {
        self.at_home = at_home;
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let home_button = button("Home")
            .on_press_maybe((!self.at_home).then_some(Message::Home));
        let zoom_in_button = button("Zoom In")
            .on_press(Message::ZoomIn);
        let zoom_out_button = button("Zoom Out")
            .on_press_maybe((!self.at_home).then_some(Message::ZoomOut));

        row![home_button, zoom_in_button, zoom_out_button,]
            .width(Length::Fill)
            .spacing(10)
            .padding(5)
            .into()
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new(true)
    }
}
