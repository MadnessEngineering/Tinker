//! Browser engine implementation

pub mod tabs;
pub mod tab_ui;
pub mod native_ui;
pub mod keyboard;
pub mod event;

use std::sync::{Arc, Mutex};
use anyhow::Result;
use tao::window::{Window, WindowBuilder};
use wry::WebView;
use wry::WebViewBuilder;

use crate::platform::windows::{WindowsConfig, WindowsManager};
use self::event::EventSystem;

pub struct BrowserEngine {
    event_loop: tao::event_loop::EventLoop<()>,
    window: Window,
    webview: Option<WebView>,
    events: Option<Arc<Mutex<EventSystem>>>,
    toolbar_height: u32,
}

impl BrowserEngine {
    pub fn new(headless: bool, events: Option<Arc<Mutex<EventSystem>>>, initial_url: Option<String>) -> Result<Self> {
        let event_loop = tao::event_loop::EventLoop::new();
        
        // Set default window size and position
        let window = WindowBuilder::new()
            .with_title("Tinker")
            .with_visible(!headless)
            .with_inner_size(tao::dpi::LogicalSize::new(1024.0, 768.0))
            .with_min_inner_size(tao::dpi::LogicalSize::new(400.0, 300.0))
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

        // Calculate toolbar height based on DPI
        let toolbar_height = if cfg!(target_os = "windows") {
            40 // Windows default toolbar height
        } else {
            36 // Default toolbar height for other platforms
        };

        // Initialize WebView with initial URL
        let webview = if let Some(url) = initial_url.clone() {
            let webview = WebViewBuilder::new(&window)
                .with_url(&url)
                .with_transparent(true)
                .with_initialization_script(include_str!("../resources/init.js"))
                .build()?;
            Some(webview)
        } else {
            None
        };

        // Notify event system of navigation if URL provided
        if let (Some(url), Some(events)) = (initial_url, &events) {
            if let Ok(mut events) = events.lock() {
                let _ = events.publish(event::BrowserEvent::Navigation { url });
            }
        }

        Ok(Self {
            event_loop,
            window,
            webview,
            events,
            toolbar_height,
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
