//! Browser engine implementation

use wry::{WebView, WebViewBuilder};

pub struct Browser {
    webview: WebView,
    event_handlers: Vec<Box<dyn Fn(&BrowserEvent)>>,
}

impl Browser {
    pub fn new() -> Result<Self, wry::Error> {
        let webview = WebViewBuilder::new()
            .with_title("TestBrowser")
            .with_url("about:blank")?
            .build()?;

        Ok(Self {
            webview,
            event_handlers: Vec::new(),
        })
    }

    pub fn navigate(&mut self, url: &str) -> Result<(), wry::Error> {
        self.webview.load_url(url)
    }
} 
