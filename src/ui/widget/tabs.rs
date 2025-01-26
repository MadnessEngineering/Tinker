use std::collections::HashMap;

use iced::widget::{button, row, Row};
use iced::{Element, Length};

use crate::browser::{Message, Tab, TabId};

#[derive(Debug, Default)]
pub struct TabBar {
    new_tab_button: button::State,
}

impl TabBar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn view(&self, tabs: &HashMap<TabId, Tab>, active_tab: Option<TabId>) -> Element<Message> {
        let mut row = Row::new().spacing(2);

        // Add tab buttons
        for (id, tab) in tabs {
            let label = format!("{} ✕", tab.title());
            let mut btn = button(label)
                .on_press(Message::ActivateTab(*id))
                .width(Length::Shrink);

            if Some(*id) == active_tab {
                btn = btn.style(theme::Button::Primary);
            }

            row = row.push(btn);
        }

        // Add new tab button
        row = row.push(
            button(&mut self.new_tab_button, "+")
                .on_press(Message::NewTab)
                .width(Length::Shrink),
        );

        row.into()
    }
} 