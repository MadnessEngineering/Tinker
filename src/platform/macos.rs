use tao::{
    window::{Window, WindowBuilder, Theme},
    dpi::LogicalSize,
    platform::macos::{WindowBuilderExtMacOS, WindowExtMacOS},
};

use super::{PlatformWindow, PlatformWebView, WindowTheme, PlatformError};
use wry::WebViewBuilder;

pub struct MacOSWindow {
    window: Window,
}

impl MacOSWindow {
    pub fn new() -> Result<Self, PlatformError> {
        let window = WindowBuilder::new()
            .with_title("Tinker")
            .with_inner_size(LogicalSize::new(1024, 768))
            .with_visible(false)
            .with_titlebar_transparent(true)
            .build()
            .map_err(|e| PlatformError::WindowCreationFailed(e.to_string()))?;

        Ok(Self { window })
    }
}

impl PlatformWindow for MacOSWindow {
    fn set_theme(&self, theme: WindowTheme) {
        match theme {
            WindowTheme::Light => self.window.set_theme(Some(Theme::Light)),
            WindowTheme::Dark => self.window.set_theme(Some(Theme::Dark)),
            WindowTheme::System => self.window.set_theme(None),
        }
    }

    fn show(&self) {
        self.window.set_visible(true);
    }

    fn hide(&self) {
        self.window.set_visible(false);
    }

    fn as_window(&self) -> &Window {
        &self.window
    }
}

pub struct MacOSWebView {
    webview: wry::WebView,
}

impl MacOSWebView {
    pub fn new(window: &Window) -> Result<Self, PlatformError> {
        let webview = WebViewBuilder::new(window)
            .with_transparent(true)
            .with_initialization_script(include_str!("../../assets/js/init.js"))
            .build()
            .map_err(|e| PlatformError::WebViewCreation(e.to_string()))?;

        Ok(Self { webview })
    }
}

impl PlatformWebView for MacOSWebView {
    fn create_webview(&self, url: &str) -> Result<WebView, PlatformError> {
        let window = self.webview.window().ok_or_else(|| {
            PlatformError::WebViewCreation("Failed to get window from WebView".to_string())
        })?;

        WebViewBuilder::new(window)
            .with_url(url)
            .build()
            .map_err(|e| PlatformError::WebViewCreation(e.to_string()))
    }

    fn set_visibility(&self, visible: bool) {
        if let Some(window) = self.webview.window() {
            window.set_visible(visible);
        }
    }

    fn update_bounds(&self, x: i32, y: i32, width: u32, height: u32) {
        if let Some(window) = self.webview.window() {
            window.set_outer_position(LogicalPosition::new(x, y));
            window.set_inner_size(LogicalSize::new(width, height));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tao::event_loop::EventLoop;

    #[test]
    #[ignore = "Window creation must be on main thread"]
    fn test_macos_window_creation() {
        let window = MacOSWindow::new();
        assert!(window.is_ok());
    }

    #[test]
    #[ignore = "Window creation must be on main thread"]
    fn test_macos_webview_creation() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .build(&event_loop)
            .expect("Failed to create window");
        
        let webview = MacOSWebView::new(&window);
        assert!(webview.is_ok());
    }
} 
