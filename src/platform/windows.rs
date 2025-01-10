use anyhow::Result;
use wry::{WebView, WebViewBuilder};
use raw_window_handle::HasWindowHandle;
use super::PlatformWebView;

pub struct WindowsWebView {
    webview: WebView,
}

impl WindowsWebView {
    pub fn new(window: &impl HasWindowHandle) -> Result<Self> {
        let webview = WebViewBuilder::new(window)
            .with_transparent(true)
            .build()?;
        
        Ok(Self { webview })
    }
}

impl PlatformWebView for WindowsWebView {
    fn new(window: &impl HasWindowHandle) -> Result<WebView> {
        Ok(WebViewBuilder::new(window)
            .with_transparent(true)
            .build()?)
    }

    fn set_bounds(&self, bounds: wry::Rect) {
        self.webview.set_bounds(bounds);
    }

    fn load_url(&self, url: &str) {
        self.webview.load_url(url);
    }

    fn evaluate_script(&self, script: &str) -> Result<()> {
        self.webview.evaluate_script(script)?;
        Ok(())
    }
} 