//! Browser engine implementation

use wry::{WebView, WebViewBuilder, Result as WryResult};
use tracing::{info, error, debug};
use std::collections::HashMap;

pub struct BrowserEngine {
    webview: WebView,
    title: String,
    history: Vec<String>,
    current_index: usize,
    tabs: HashMap<usize, Tab>,
    active_tab: usize,
    next_tab_id: usize,
}

struct Tab {
    webview: WebView,
    title: String,
    url: String,
}

impl BrowserEngine {
    pub fn forge(headless: bool) -> WryResult<Self> {
        info!("Forging new browser engine...");
        
        let initial_tab = Tab {
            webview: WebViewBuilder::new()
                .with_title("Tinker Workshop")
                .with_url("about:blank")?
                .with_visible(!headless)
                .build()?,
            title: String::from("New Tab"),
            url: String::from("about:blank"),
        };

        let mut tabs = HashMap::new();
        tabs.insert(0, initial_tab);

        info!("Browser engine forged successfully");
        
        Ok(Self {
            webview: WebViewBuilder::new()
                .with_title("Tinker Workshop")
                .with_url("about:blank")?
                .with_visible(!headless)
                .build()?,
            title: String::from("Tinker Workshop"),
            history: vec![String::from("about:blank")],
            current_index: 0,
            tabs,
            active_tab: 0,
            next_tab_id: 1,
        })
    }

    // Navigation Controls
    pub fn navigate(&mut self, url: &str) -> WryResult<()> {
        info!("Navigating to: {}", url);
        self.history.truncate(self.current_index + 1);
        self.history.push(url.to_string());
        self.current_index = self.history.len() - 1;
        self.webview.load_url(url)
    }

    pub fn back(&mut self) -> WryResult<()> {
        if self.current_index > 0 {
            self.current_index -= 1;
            let url = &self.history[self.current_index];
            debug!("Navigating back to: {}", url);
            self.webview.load_url(url)
        } else {
            debug!("No previous page in history");
            Ok(())
        }
    }

    pub fn forward(&mut self) -> WryResult<()> {
        if self.current_index < self.history.len() - 1 {
            self.current_index += 1;
            let url = &self.history[self.current_index];
            debug!("Navigating forward to: {}", url);
            self.webview.load_url(url)
        } else {
            debug!("No next page in history");
            Ok(())
        }
    }

    pub fn refresh(&self) -> WryResult<()> {
        debug!("Refreshing current page");
        self.webview.evaluate_script("window.location.reload()")
    }

    // Tab Management
    pub fn new_tab(&mut self, url: Option<String>) -> WryResult<usize> {
        let url = url.unwrap_or_else(|| String::from("about:blank"));
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let tab = Tab {
            webview: WebViewBuilder::new()
                .with_title("New Tab")
                .with_url(&url)?
                .with_visible(true)
                .build()?,
            title: String::from("New Tab"),
            url,
        };

        self.tabs.insert(tab_id, tab);
        info!("Created new tab with ID: {}", tab_id);
        Ok(tab_id)
    }

    pub fn close_tab(&mut self, tab_id: usize) -> WryResult<()> {
        if self.tabs.len() > 1 {
            self.tabs.remove(&tab_id);
            info!("Closed tab: {}", tab_id);
            
            if self.active_tab == tab_id {
                // Switch to the last tab
                self.active_tab = *self.tabs.keys().max().unwrap_or(&0);
            }
            Ok(())
        } else {
            error!("Cannot close the last tab");
            Ok(())
        }
    }

    pub fn switch_tab(&mut self, tab_id: usize) -> WryResult<()> {
        if self.tabs.contains_key(&tab_id) {
            self.active_tab = tab_id;
            info!("Switched to tab: {}", tab_id);
            Ok(())
        } else {
            error!("Tab {} does not exist", tab_id);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_history() {
        let mut browser = BrowserEngine::forge(true).unwrap();
        
        // Test navigation
        browser.navigate("https://example.com").unwrap();
        browser.navigate("https://test.com").unwrap();
        assert_eq!(browser.history.len(), 3); // including about:blank
        
        // Test back
        browser.back().unwrap();
        assert_eq!(browser.current_index, 1);
        
        // Test forward
        browser.forward().unwrap();
        assert_eq!(browser.current_index, 2);
    }

    #[test]
    fn test_tab_management() {
        let mut browser = BrowserEngine::forge(true).unwrap();
        
        // Test new tab
        let tab_id = browser.new_tab(None).unwrap();
        assert_eq!(tab_id, 1);
        assert_eq!(browser.tabs.len(), 2);
        
        // Test switching tabs
        browser.switch_tab(tab_id).unwrap();
        assert_eq!(browser.active_tab, tab_id);
        
        // Test closing tab
        browser.close_tab(tab_id).unwrap();
        assert_eq!(browser.tabs.len(), 1);
    }
} 
