//! Browser engine implementation

use std::sync::Arc;
use anyhow::Result;
use tracing::debug;

#[cfg(feature = "webview")]
pub mod tabs;
#[cfg(feature = "webview")]
pub mod tab_ui;
#[cfg(feature = "native-ui")]
pub mod native_ui;
#[cfg(feature = "native-ui")]
pub mod keyboard;
pub mod event;

#[cfg(feature = "webview")]
use tao::window::{Window, WindowBuilder};
#[cfg(feature = "webview")]
use wry::{WebView, WebViewBuilder};

use crate::platform::PlatformManager;
#[cfg(all(target_os = "windows", feature = "native-ui"))]
use crate::platform::windows::{WindowsConfig, WindowsManager};

pub struct BrowserEngine {
    #[cfg(feature = "webview")]
    event_loop: tao::event_loop::EventLoop<()>,
    #[cfg(feature = "webview")]
    window: Window,
    #[cfg(feature = "webview")]
    webview: Option<WebView>,
    events: Option<Arc<std::sync::Mutex<event::EventSystem>>>,
    #[cfg(feature = "native-ui")]
    toolbar_height: u32,
}

impl BrowserEngine {
    pub fn new(headless: bool, events: Option<Arc<std::sync::Mutex<event::EventSystem>>>, initial_url: Option<String>) -> Result<Self> {
        #[cfg(feature = "webview")]
        let event_loop = tao::event_loop::EventLoop::new();
        
        #[cfg(feature = "webview")]
        let window = WindowBuilder::new()
            .with_title("Tinker")
            .with_visible(!headless)
            .with_inner_size(tao::dpi::LogicalSize::new(1024.0, 768.0))
            .with_min_inner_size(tao::dpi::LogicalSize::new(400.0, 300.0))
            .build(&event_loop)?;

        #[cfg(all(target_os = "windows", feature = "native-ui"))]
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

        #[cfg(feature = "native-ui")]
        let toolbar_height = if cfg!(target_os = "windows") {
            40 // Windows default toolbar height
        } else {
            36 // Default toolbar height for other platforms
        };

        #[cfg(not(feature = "native-ui"))]
        let toolbar_height = 0;

        #[cfg(feature = "webview")]
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

        #[cfg(not(feature = "webview"))]
        let webview = None;

        // Notify event system of navigation if URL provided
        if let (Some(url), Some(events)) = (initial_url, &events) {
            if let Ok(mut events) = events.lock() {
                let _ = events.publish(event::BrowserEvent::Navigation { url });
            }
        }

        Ok(Self {
            #[cfg(feature = "webview")]
            event_loop,
            #[cfg(feature = "webview")]
            window,
            #[cfg(feature = "webview")]
            webview,
            events,
            #[cfg(feature = "native-ui")]
            toolbar_height,
        })
    }

    #[cfg(feature = "webview")]
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

    #[cfg(not(feature = "webview"))]
    pub fn run(self) -> Result<()> {
        debug!("Running in headless mode");
        Ok(())
    }
}
