use iced::widget::{button, text_input, Row};
use iced::{Element, Length};

use crate::browser::Message;

#[derive(Debug, Default)]
pub struct NavigationBar {
    url_input: text_input::State,
    back_button: button::State,
    forward_button: button::State,
    refresh_button: button::State,
}

impl NavigationBar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn view(&self) -> Element<Message> {
        Row::new()
            .spacing(5)
            .push(
                button(&mut self.back_button, "←")
                    .on_press(Message::TabMessage(None, TabMessage::Back)),
            )
            .push(
                button(&mut self.forward_button, "→")
                    .on_press(Message::TabMessage(None, TabMessage::Forward)),
            )
            .push(
                button(&mut self.refresh_button, "⟳")
                    .on_press(Message::TabMessage(None, TabMessage::Reload)),
            )
            .push(
                text_input(&mut self.url_input, "Enter URL...")
                    .on_submit(|url| Message::Navigate(url))
                    .padding(5)
                    .width(Length::Fill),
            )
            .into()
    }
} 