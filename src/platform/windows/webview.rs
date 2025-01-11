use anyhow::Result;
use wry::WebView;
use windows::Win32::Foundation::HWND;
use raw_window_handle::HasRawWindowHandle;
use super::config::WindowsConfig;
use crate::platform::PlatformWebView;

pub struct WindowsWebView {
    webview: WebView,
    hwnd: HWND,
    config: WindowsConfig,
    visible: bool,
}

impl WindowsWebView {
    pub fn new(webview: WebView, hwnd: HWND, config: WindowsConfig) -> Self {
        Self {
            webview,
            hwnd,
            config,
            visible: true,
        }
    }
}

impl PlatformWebView for WindowsWebView {
    fn new<T>(window: &T) -> Result<Self> 
    where T: HasRawWindowHandle 
    {
        let hwnd = HWND(window.raw_window_handle().raw_handle() as isize);
        let config = WindowsConfig::default();
        let webview = WebView::new(window)?;
        
        Ok(Self {
            webview,
            hwnd,
            config,
            visible: true,
        })
    }

    fn navigate(&self, url: &str) -> Result<()> {
        self.webview.load_url(url)?;
        Ok(())
    }

    fn evaluate_script(&self, script: &str) -> Result<()> {
        self.webview.evaluate_script(script)?;
        Ok(())
    }

    fn resize(&self, width: i32, height: i32) {
        if let Some(view) = self.webview.window() {
            view.set_inner_size(tao::dpi::LogicalSize::new(width as f64, height as f64));
        }
    }

    fn set_theme(&self, theme: &str) -> Result<()> {
        // Windows-specific theme implementation
        use windows::Win32::UI::WindowsAndMessaging::{SetWindowLongW, GWL_STYLE};
        let style = match theme {
            "dark" => self.config.get_dark_theme_style(),
            "light" => self.config.get_light_theme_style(),
            _ => self.config.get_default_style(),
        };
        
        unsafe {
            SetWindowLongW(self.hwnd, GWL_STYLE, style as i32);
        }
        Ok(())
    }

    fn set_title(&self, title: &str) -> Result<()> {
        if let Some(window) = self.webview.window() {
            window.set_title(title);
        }
        Ok(())
    }

    fn set_visible(&self, visible: bool) -> Result<()> {
        if let Some(window) = self.webview.window() {
            window.set_visible(visible);
        }
        Ok(())
    }

    fn get_position(&self) -> Result<(i32, i32)> {
        if let Some(window) = self.webview.window() {
            let pos = window.outer_position()?;
            Ok((pos.x, pos.y))
        } else {
            Ok((0, 0))
        }
    }

    fn set_position(&self, x: i32, y: i32) -> Result<()> {
        if let Some(window) = self.webview.window() {
            window.set_outer_position(tao::dpi::LogicalPosition::new(x, y));
        }
        Ok(())
    }

    fn handle_platform_message(&self, message: &str) -> Result<()> {
        // Windows-specific message handling
        match message {
            "maximize" => {
                if let Some(window) = self.webview.window() {
                    window.set_maximized(true);
                }
            }
            "minimize" => {
                if let Some(window) = self.webview.window() {
                    window.set_minimized(true);
                }
            }
            "restore" => {
                if let Some(window) = self.webview.window() {
                    window.set_maximized(false);
                    window.set_minimized(false);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn get_parent_window(&self) -> Option<&dyn HasRawWindowHandle> {
        self.webview.window().map(|w| w as &dyn HasRawWindowHandle)
    }
} 