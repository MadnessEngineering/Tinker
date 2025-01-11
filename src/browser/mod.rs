//! Browser engine implementation

pub mod tabs;
pub mod tab_ui;

use std::sync::{Arc, Mutex};
use anyhow::Result;
use tracing::{debug, error, info};
use wry::WebView;
use tao::window::{Window, WindowBuilder};
use tao::event_loop::{EventLoop, EventLoopWindowTarget};

use crate::event::EventSystem;
use self::tabs::TabManager;
use self::tab_ui::{TabBar, TabCommand};

pub struct BrowserEngine {
    window: Window,
    tab_manager: TabManager,
    events: Option<Arc<Mutex<EventSystem>>>,
    event_loop: EventLoop<()>,
}

impl BrowserEngine {
    pub fn new(
        headless: bool,
        events: Option<Arc<Mutex<EventSystem>>>,
        initial_url: Option<String>,
    ) -> Result<Self> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title("Tinker")
            .with_decorations(!headless)
            .with_transparent(true)
            .build(&event_loop)?;

        let tab_manager = TabManager::new();

        Ok(Self {
            window,
            tab_manager,
            events,
            event_loop,
        })
    }

    // ... rest of implementation ...
}
