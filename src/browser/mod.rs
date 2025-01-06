//! Browser engine implementation

use wry::{WebView, WebViewBuilder, Result as WryResult};
use tracing::{info, error};

pub struct BrowserEngine {
    webview: WebView,
    title: String,
}

impl BrowserEngine {
    pub fn forge(headless: bool) -> WryResult<Self> {
        info!("Forging new browser engine...");
        
        let webview = WebViewBuilder::new()
            .with_title("Tinker Workshop")
            .with_url("about:blank")?
            .with_visible(!headless)
            .build()?;

        info!("Browser engine forged successfully");
        
        Ok(Self {
            webview,
            title: String::from("Tinker Workshop"),
        })
    }

    pub fn navigate(&self, url: &str) -> WryResult<()> {
        info!("Navigating to: {}", url);
        self.webview.load_url(url)
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        if let Err(e) = self.webview.set_title(title) {
            error!("Failed to set window title: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_creation() {
        let browser = BrowserEngine::forge(true);
        assert!(browser.is_ok());
    }

    #[test]
    fn test_browser_navigation() {
        let browser = BrowserEngine::forge(true).unwrap();
        let result = browser.navigate("https://example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_title_setting() {
        let mut browser = BrowserEngine::forge(true).unwrap();
        browser.set_title("Test Title");
        assert_eq!(browser.title, "Test Title");
    }
} 
