//! Browser engine implementation

pub mod tabs;
pub mod tab_ui;

use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use tracing::debug;
use tao::window::Window;
use tao::event_loop::EventLoop;
use raw_window_handle::HasRawWindowHandle;

use crate::platform::{
    Platform,
    PlatformWebView,
};

#[cfg(target_os = "windows")]
use crate::platform::{
    WindowsWebView,
    WindowsConfig,
};

#[cfg(target_os = "macos")]
use crate::platform::{
    MacosWebView,
    MacosConfig,
};

pub struct Browser {
    event_loop: EventLoop<()>,
    window: Window,
    webview: Option<Box<dyn PlatformWebView>>,
    initial_url: Option<String>,
}

impl Browser {
    pub fn new(title: impl Into<String>) -> Result<Self> {
        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop)?;

        match Platform::current() {
            #[cfg(target_os = "windows")]
            Platform::Windows => {
                let config = WindowsConfig {
                    title: title.into(),
                    width: 800,
                    height: 600,
                    decorations: true,
                    dpi_aware: true,
                };
                Ok(Self {
                    event_loop,
                    window,
                    webview: None,
                    initial_url: None,
                })
            }
            #[cfg(target_os = "macos")]
            Platform::MacOS => {
                let config = MacosConfig {
                    title: title.into(),
                    width: 800,
                    height: 600,
                    decorations: true,
                };
                Ok(Self {
                    event_loop,
                    window,
                    webview: None,
                    initial_url: None,
                })
            }
            _ => Err(anyhow!("Unsupported platform")),
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.initial_url = Some(url.into());
        self
    }

    pub fn run(mut self) -> Result<()> {
        if let Some(url) = &self.initial_url {
            debug!("Loading initial URL: {}", url);
            #[cfg(target_os = "windows")]
            {
                let config = WindowsConfig {
                    title: "Tinker".to_string(),
                    width: 800,
                    height: 600,
                    decorations: true,
                    dpi_aware: true,
                };
                let webview = WindowsWebView::new(&self.window, config)?;
                webview.navigate(url)?;
                self.webview = Some(Box::new(webview));
            }
            #[cfg(target_os = "macos")]
            {
                let config = MacosConfig {
                    title: "Tinker".to_string(),
                    width: 800,
                    height: 600,
                    decorations: true,
                };
                let webview = MacosWebView::new(&self.window, config)?;
                webview.navigate(url)?;
                self.webview = Some(Box::new(webview));
            }
        }

        self.event_loop.run(move |event, _, control_flow| {
            use tao::event::{Event, WindowEvent};
            use tao::event_loop::ControlFlow;

            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            }
        });

        Ok(())
    }
}
