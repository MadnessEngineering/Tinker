use std::error::Error;
use wry::{WebView, WebViewBuilder};
use raw_window_handle::HasRawWindowHandle;
use windows::Win32::Foundation::HWND;
use super::window::Window;

pub struct WebViewWrapper<T: HasRawWindowHandle> {
    webview: WebView,
    _window: T,
}

impl<T: HasRawWindowHandle> WebViewWrapper<T> {
    pub fn new(window: T) -> Result<Self, Box<dyn Error>> {
        let hwnd = unsafe {
            let handle = window.raw_window_handle();
            match handle {
                raw_window_handle::RawWindowHandle::Win32(handle) => {
                    HWND(handle.hwnd as isize)
                }
                _ => return Err("Unsupported window handle type".into()),
            }
        };

        let webview = WebViewBuilder::new(hwnd)
            .with_transparent(true)
            .build()?;

        Ok(Self {
            webview,
            _window: window,
        })
    }

    pub fn load_url(&self, url: &str) -> Result<(), Box<dyn Error>> {
        self.webview.load_url(url)?;
        Ok(())
    }

    pub fn evaluate_script(&self, script: &str) -> Result<(), Box<dyn Error>> {
        self.webview.evaluate_script(script)?;
        Ok(())
    }

    pub fn resize(&self, width: i32, height: i32) {
        self.webview.resize().unwrap();
    }

    pub fn set_title(&self, title: &str) {
        if let Some(window) = self._window.window() {
            window.set_title(title);
        }
    }

    pub fn set_visible(&self, visible: bool) {
        if let Some(window) = self._window.window() {
            window.set_visible(visible);
        }
    }

    pub fn set_resizable(&self, resizable: bool) {
        if let Some(window) = self._window.window() {
            window.set_resizable(resizable);
        }
    }

    pub fn set_maximized(&self, maximized: bool) {
        if let Some(window) = self._window.window() {
            window.set_maximized(maximized);
        }
    }

    pub fn set_minimized(&self, minimized: bool) {
        if let Some(window) = self._window.window() {
            window.set_minimized(minimized);
        }
    }

    pub fn set_decorations(&self, decorations: bool) {
        if let Some(window) = self._window.window() {
            window.set_decorations(decorations);
        }
    }

    pub fn set_always_on_top(&self, always_on_top: bool) {
        if let Some(window) = self._window.window() {
            window.set_always_on_top(always_on_top);
        }
    }

    pub fn set_content_protection(&self, enabled: bool) {
        if let Some(window) = self._window.window() {
            window.set_content_protection(enabled);
        }
    }
} 