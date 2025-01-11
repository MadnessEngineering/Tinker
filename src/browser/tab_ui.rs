use tao::window::Window;
use std::sync::mpsc::Sender;
use wry::{WebView, WebViewBuilder};
use std::sync::{Arc, Mutex};
use tracing::{debug, error};
use crate::browser::error::BrowserResult;

/// Manages the browser's tab bar UI
#[derive(Clone)]
pub struct TabBar {
    webview: Arc<WebView>,
}

impl TabBar {
    pub fn new(window: &Window) -> BrowserResult<Self> {
        let webview = WebViewBuilder::new(window)
            .with_html(include_str!("../templates/tab_bar.html"))?
            .with_initialization_script(include_str!("../templates/tab_bar.js"))?
            .build()?;

        Ok(Self {
            webview: Arc::new(webview),
        })
    }

    /// Add a new tab to the UI
    pub fn add_tab(&self, id: &str) -> BrowserResult<()> {
        let script = format!(
            "window.tabBar.addTab('{}', 'New Tab', 'about:blank');",
            id
        );
        self.webview.evaluate_script(&script)
            .map_err(|e| format!("Failed to add tab: {}", e))?;
        Ok(())
    }

    /// Update the URL display
    pub fn update_url(&self, url: &str) -> BrowserResult<()> {
        let script = format!(
            "window.tabBar.updateUrl('{}');",
            url.replace('\'', "\\'")
        );
        self.webview.evaluate_script(&script)
            .map_err(|e| format!("Failed to update URL: {}", e))?;
        Ok(())
    }

    /// Update navigation button states
    pub fn update_navigation_state(&self, can_go_back: bool, can_go_forward: bool) -> BrowserResult<()> {
        let script = format!(
            "window.tabBar.updateNavigationState({}, {});",
            can_go_back, can_go_forward
        );
        self.webview.evaluate_script(&script)
            .map_err(|e| format!("Failed to update navigation state: {}", e))?;
        Ok(())
    }

    /// Update loading state
    pub fn update_loading_state(&self, is_loading: bool) -> BrowserResult<()> {
        let script = format!(
            "window.tabBar.updateLoadingState({});",
            is_loading
        );
        self.webview.evaluate_script(&script)
            .map_err(|e| format!("Failed to update loading state: {}", e))?;
        Ok(())
    }

    /// Update tab order after reordering
    pub fn update_tab_order(&self, tab_ids: &[String]) -> BrowserResult<()> {
        let ids_json = serde_json::to_string(tab_ids)
            .map_err(|e| format!("Failed to serialize tab IDs: {}", e))?;
        
        let script = format!(
            "window.tabBar.updateTabOrder({});",
            ids_json
        );
        self.webview.evaluate_script(&script)
            .map_err(|e| format!("Failed to update tab order: {}", e))?;
        Ok(())
    }

    /// Start tab dragging
    pub fn start_tab_drag(&self, id: &str) -> BrowserResult<()> {
        let script = format!(
            "window.tabBar.startDrag('{}');",
            id
        );
        self.webview.evaluate_script(&script)
            .map_err(|e| format!("Failed to start drag: {}", e))?;
        Ok(())
    }

    /// End tab dragging
    pub fn end_tab_drag(&self) -> BrowserResult<()> {
        let script = "window.tabBar.endDrag();";
        self.webview.evaluate_script(script)
            .map_err(|e| format!("Failed to end drag: {}", e))?;
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TabCommand {
    Create { url: String },
    Close { id: usize },
    Switch { id: usize },
    UpdateUrl { id: usize, url: String },
    UpdateTitle { id: usize, title: String },
    Reload,
    Stop,
    Split,
    DragStart { id: String },
    DragEnd { id: String, target_id: String },
} 

#[cfg(test)]
mod tests {
    use super::*;
    use tao::window::{Window, WindowBuilder};
    use tao::event_loop::EventLoop;

    fn create_test_window() -> (Window, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Test Window")
            .build(&event_loop)
            .expect("Failed to create test window");
        (window, event_loop)
    }

    #[test]
    fn test_tab_bar_creation() {
        let (window, _event_loop) = create_test_window();
        let tab_bar = TabBar::new(&window);
        assert!(tab_bar.is_ok());
    }

    // Add more test functions here...
} 
