use std::sync::Arc;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};
use tracing::{debug, error, info};
use wry::{WebView, WebViewBuilder};

use crate::utils::Result;

#[derive(Debug, thiserror::Error)]
pub enum WebViewError {
    #[error("Failed to create WebView on Windows: {0}")]
    WindowsError(String),

    #[error("Failed to create WebView on macOS: {0}")]
    MacOSError(String),

    #[error("Failed to create WebView on Linux: {0}")]
    LinuxError(String),

    #[error("Failed to create WebView: {0}")]
    InitError(#[from] wry::Error),

    #[error("Failed to create window: {0}")]
    WindowError(String),
}

pub struct Browser {
    webview: WebView,
}

impl Browser {
    pub fn new() -> Result<(Self, EventLoop<()>)> {
        info!("Creating new browser instance");
        
        // Create the event loop
        let event_loop = EventLoop::new();
        debug!("Created event loop");

        // Create the main window
        let window = WindowBuilder::new()
            .with_title("Tinker Browser")
            .with_inner_size(tao::dpi::LogicalSize::new(1024.0, 768.0))
            .with_min_inner_size(tao::dpi::LogicalSize::new(400.0, 300.0))
            .build(&event_loop)?;

        debug!("Created main window");

        // Create the web view
        let webview = WebViewBuilder::new(&window)
            .with_url("https://example.com")?
            .with_initialization_script(include_str!("../../assets/js/init.js"))
            .with_ipc_handler(|req| {
                debug!("IPC request: {}", req);
                // Handle IPC messages here
            })
            .build()?;

        info!("Created web view");

        Ok((Self { webview }, event_loop))
    }

    pub fn handle_event(&self, event: Event<()>, control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    info!("Window close requested");
                    *control_flow = ControlFlow::Exit
                }
                WindowEvent::Resized(new_size) => {
                    debug!("Window resized to {:?}", new_size);
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                // Application update code here
            }
            _ => (),
        }
    }
} 