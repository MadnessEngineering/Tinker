use iced::{
    executor,
    theme::{self, Theme},
    Application, Command, Element, Settings, Subscription,
};

mod browser;
mod engine;
mod ui;
mod utils;

use browser::Browser;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    Browser::run(Settings {
        window: iced::window::Settings {
            size: (1024, 768),
            position: iced::window::Position::Centered,
            min_size: Some((400, 300)),
            ..Default::default()
        },
        default_text_size: 16.0,
        ..Default::default()
    })
}

// Re-export commonly used types
pub mod prelude {
    pub use super::browser::{Message, Tab, TabId};
    pub use super::engine::{Content, Resource};
    pub use super::ui::{style, widget};
    pub use super::utils::Result;
    
    pub use iced::{
        alignment, theme, Application, Color, Command, Element,
        Length, Point, Rectangle, Size, Subscription, Vector,
    };
}
