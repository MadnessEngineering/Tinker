use anyhow::{Result, anyhow};
use wry::{WebView, WebViewBuilder};
use raw_window_handle::HasWindowHandle;
use windows::Win32::{
    UI::{
        WindowsAndMessaging as win32_window,
        Controls as win32_controls,
        Shell as win32_shell,
        HiDpi as win32_dpi,
        Input::KeyboardAndMouse as win32_input,
    },
    Foundation::{HWND, LPARAM, WPARAM, LRESULT},
    Graphics::Gdi::{HBRUSH, HDC},
};
use tracing::{debug, error};
use crate::platform::PlatformWebView;

pub struct WindowsWebView {
    pub webview: WebView,
    hwnd: HWND,
    dpi_aware: bool,
}

impl WindowsWebView {
    pub fn new(window: &impl HasWindowHandle) -> Result<Self> {
        // Enable DPI awareness
        unsafe {
            win32_dpi::SetProcessDpiAwareness(win32_dpi::PROCESS_PER_MONITOR_DPI_AWARE)?;
        }

        // Create WebView with Windows-specific configuration
        let webview = WebViewBuilder::new(window)
            .with_transparent(true)
            .with_theme(Some(wry::Theme::Dark))
            .build()
            .map_err(|e| anyhow!("Failed to create Windows WebView: {}", e))?;

        // Get native window handle
        let hwnd = unsafe {
            HWND(window.raw_window_handle().get_raw_handle() as isize)
        };

        Ok(Self { 
            webview,
            hwnd,
            dpi_aware: true,
        })
    }

    pub fn set_dpi_aware(&mut self, aware: bool) {
        self.dpi_aware = aware;
        if let Err(e) = unsafe {
            win32_dpi::SetProcessDpiAwareness(
                if aware {
                    win32_dpi::PROCESS_PER_MONITOR_DPI_AWARE
                } else {
                    win32_dpi::PROCESS_DPI_UNAWARE
                }
            )
        } {
            error!("Failed to set DPI awareness: {:?}", e);
        }
    }

    pub fn handle_window_message(&self, msg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match msg {
            win32_window::WM_DPICHANGED => {
                if self.dpi_aware {
                    // Handle DPI change
                    let dpi = win32_dpi::GetDpiForWindow(self.hwnd);
                    debug!("DPI changed to {}", dpi);
                    // Update window and WebView scaling
                    self.update_dpi_scaling(dpi);
                }
                Some(LRESULT(0))
            }
            _ => None
        }
    }

    fn update_dpi_scaling(&self, dpi: u32) {
        let scale_factor = dpi as f64 / 96.0; // 96 is the default DPI
        if let Err(e) = self.webview.evaluate_script(&format!(
            "document.body.style.zoom = '{}';", 
            scale_factor
        )) {
            error!("Failed to update WebView scaling: {}", e);
        }
    }
}

impl PlatformWebView for WindowsWebView {
    fn new(window: &impl HasWindowHandle) -> Result<WebView> {
        Self::new(window).map(|wv| wv.webview)
    }

    fn set_bounds(&self, bounds: wry::Rect) {
        if self.dpi_aware {
            let dpi = unsafe { win32_dpi::GetDpiForWindow(self.hwnd) };
            let scale = dpi as f64 / 96.0;
            let scaled_bounds = wry::Rect {
                x: (bounds.x as f64 * scale) as i32,
                y: (bounds.y as f64 * scale) as i32,
                width: (bounds.width as f64 * scale) as u32,
                height: (bounds.height as f64 * scale) as u32,
            };
            self.webview.set_bounds(scaled_bounds);
        } else {
            self.webview.set_bounds(bounds);
        }
    }

    fn load_url(&self, url: &str) {
        self.webview.load_url(url);
    }

    fn evaluate_script(&self, script: &str) -> Result<()> {
        self.webview.evaluate_script(script)
            .map_err(|e| anyhow!("Failed to evaluate script: {}", e))
    }
} 