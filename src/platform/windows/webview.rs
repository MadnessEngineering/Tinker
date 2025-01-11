use anyhow::Result;
use wry::WebView;
use windows::Win32::Foundation::HWND;
use super::config::WindowsConfig;

pub struct WindowsWebView {
    webview: WebView,
    hwnd: HWND,
    config: WindowsConfig,
}

impl WindowsWebView {
    pub fn new(webview: WebView, hwnd: HWND, config: WindowsConfig) -> Self {
        Self {
            webview,
            hwnd,
            config,
        }
    }

    pub fn evaluate_script(&self, script: &str) -> Result<()> {
        self.webview.evaluate_script(script)?;
        Ok(())
    }
} 