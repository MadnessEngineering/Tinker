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
    pub fn new() -> Self {
        // Initialize tab bar WebView with drag and drop support
        let webview = WebViewBuilder::new()
            .with_html(include_str!("../templates/tab_bar.html"))
            .with_initialization_script(include_str!("../templates/tab_bar.js"))
            .build()
            .expect("Failed to create tab bar WebView");

        Self {
            webview: Arc::new(webview),
        }
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
    use wry::WebViewBuilder;

    fn create_test_tab_bar() -> TabBar {
        TabBar::new()
    }

    #[test]
    fn test_tab_bar_creation() {
        let tab_bar = create_test_tab_bar();
        assert!(tab_bar.webview.evaluate_script("window.tabBar !== undefined").is_ok());
    }

    #[test]
    fn test_tab_addition() {
        let tab_bar = create_test_tab_bar();
        assert!(tab_bar.add_tab("test_tab").is_ok());
    }

    #[test]
    fn test_url_update() {
        let tab_bar = create_test_tab_bar();
        assert!(tab_bar.update_url("https://example.com").is_ok());
        
        // Test URL escaping
        assert!(tab_bar.update_url("https://example.com/path's/end").is_ok());
    }

    #[test]
    fn test_navigation_state() {
        let tab_bar = create_test_tab_bar();
        assert!(tab_bar.update_navigation_state(true, false).is_ok());
        assert!(tab_bar.update_navigation_state(false, true).is_ok());
        assert!(tab_bar.update_navigation_state(false, false).is_ok());
    }

    #[test]
    fn test_loading_state() {
        let tab_bar = create_test_tab_bar();
        assert!(tab_bar.update_loading_state(true).is_ok());
        assert!(tab_bar.update_loading_state(false).is_ok());
    }

    #[test]
    fn test_tab_order_update() {
        let tab_bar = create_test_tab_bar();
        let tab_ids = vec![
            "tab_0".to_string(),
            "tab_1".to_string(),
            "tab_2".to_string(),
        ];
        assert!(tab_bar.update_tab_order(&tab_ids).is_ok());
    }

    #[test]
    fn test_drag_operations() {
        let tab_bar = create_test_tab_bar();
        
        // Test drag start
        assert!(tab_bar.start_tab_drag("test_tab").is_ok());
        
        // Test drag end
        assert!(tab_bar.end_tab_drag().is_ok());
    }

    #[test]
    fn test_tab_command_serialization() {
        use serde_json::json;

        // Test create command
        let create_json = json!({
            "type": "create",
            "url": "https://example.com"
        });
        let create_cmd: TabCommand = serde_json::from_value(create_json).unwrap();
        assert!(matches!(create_cmd, TabCommand::Create { url } if url == "https://example.com"));

        // Test drag commands
        let drag_start_json = json!({
            "type": "drag_start",
            "id": "tab_0"
        });
        let drag_start_cmd: TabCommand = serde_json::from_value(drag_start_json).unwrap();
        assert!(matches!(drag_start_cmd, TabCommand::DragStart { id } if id == "tab_0"));

        let drag_end_json = json!({
            "type": "drag_end",
            "id": "tab_0",
            "target_id": "tab_1"
        });
        let drag_end_cmd: TabCommand = serde_json::from_value(drag_end_json).unwrap();
        assert!(matches!(drag_end_cmd, TabCommand::DragEnd { id, target_id } 
            if id == "tab_0" && target_id == "tab_1"));
    }
} 
