//! Browser engine implementation

pub mod tabs;
pub mod tab_ui;
pub mod native_ui;
pub mod keyboard;

use std::sync::{Arc, Mutex};
use anyhow::Result;
use tao::window::{Window, WindowBuilder};
use wry::WebView;

use crate::platform::windows::{WindowsConfig, WindowsManager};
use tinker::event::EventSystem;

pub struct BrowserEngine {
    event_loop: tao::event_loop::EventLoop<()>,
    window: Window,
    webview: Option<WebView>,
    events: Option<Arc<Mutex<EventSystem>>>,
}

impl BrowserEngine {
    pub fn new(_headless: bool, events: Option<Arc<Mutex<EventSystem>>>, _initial_url: Option<String>) -> Result<Self> {
        let event_loop = tao::event_loop::EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Tinker")
            .build(&event_loop)?;

        #[cfg(target_os = "windows")]
        {
            let config = WindowsConfig {
                title: "Tinker".to_string(),
                width: 1024,
                height: 768,
                decorations: true,
                dpi_aware: true,
            };
            let _manager = WindowsManager::new(config)?;
        }

        Ok(Self {
            event_loop,
            window,
            webview: None,
            events,
        })
    }

    pub fn run(self) -> ! {
        self.event_loop.run(move |event, _, control_flow| {
            use tao::event::{Event, WindowEvent};
            use tao::event_loop::ControlFlow;

            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        })
    }
}
