use anyhow::{Result, anyhow};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use wry::{WebView, WebViewBuilder};
use windows::Win32::Foundation::HWND;
use tracing::debug;
use super::config::WindowsConfig;
use crate::platform::PlatformWebView;
use tao::window::Window;

pub struct WindowsWebView {
    webview: WebView,
    hwnd: HWND,
    config: WindowsConfig,
}

impl WindowsWebView {
    pub fn new<T: HasRawWindowHandle>(window: &T, config: WindowsConfig) -> Result<Self> {
        let handle = window.raw_window_handle();
        let hwnd = match handle {
            RawWindowHandle::Win32(handle) => HWND(handle.hwnd as isize),
            _ => return Err(anyhow!("Expected Win32 window handle")),
        };

        // Create a temporary window to host the WebView
        let event_loop = tao::event_loop::EventLoop::new();
        let window = Window::new(&event_loop)?;

        let webview = WebViewBuilder::new(&window)
            .with_transparent(true)
            .build()?;

        Ok(Self {
            webview,
            hwnd,
            config,
        })
    }
}

impl PlatformWebView for WindowsWebView {
    fn new<T: HasRawWindowHandle>(window: &T) -> Result<Self> {
        let config = WindowsConfig::default();
        Self::new(window, config)
    }

    fn navigate(&self, url: &str) -> Result<()> {
        self.webview.load_url(url);
        Ok(())
    }

    fn evaluate_script(&self, script: &str) -> Result<()> {
        self.webview.evaluate_script(script);
        Ok(())
    }

    fn resize(&self, width: i32, height: i32) {
        debug!("Resizing webview to {}x{}", width, height);
        // No need to explicitly resize the WebView as it automatically
        // follows the window size
    }
} 