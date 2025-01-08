use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt;
use tracing::debug;
use wry::WebView;

pub struct Tab {
    pub id: usize,
    pub url: String,
    pub title: String,
    pub webview: Option<Arc<Mutex<WebView>>>,
}

impl fmt::Debug for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tab")
            .field("id", &self.id)
            .field("url", &self.url)
            .field("title", &self.title)
            .field("webview", &if self.webview.is_some() { "Some(WebView)" } else { "None" })
            .finish()
    }
}

#[derive(Default)]
pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
    next_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        TabManager {
            tabs: Vec::new(),
            active_tab: None,
            next_id: 0,
        }
    }

    pub fn create_tab(&mut self, url: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let tab = Tab {
            id,
            url,
            title: String::from("New Tab"),
            webview: None,
        };

        let is_first_tab = self.tabs.is_empty();
        self.tabs.push(tab);
        if is_first_tab {
            self.active_tab = Some(0);
        } else {
            self.active_tab = Some(self.tabs.len() - 1);
        }
        debug!("Created new tab with id: {}", id);
        id
    }

    pub fn set_tab_webview(&mut self, id: usize, webview: Arc<Mutex<WebView>>) -> bool {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.webview = Some(webview);
            true
        } else {
            false
        }
    }

    pub fn get_tab_webview(&self, id: usize) -> Option<Arc<Mutex<WebView>>> {
        self.tabs.iter()
            .find(|t| t.id == id)
            .and_then(|tab| tab.webview.clone())
    }

    pub fn close_tab(&mut self, id: usize) -> bool {
        // Find the index of the tab with the given id
        if let Some(index) = self.tabs.iter().position(|tab| tab.id == id) {
            // Cannot close the last tab
            if self.tabs.len() == 1 {
                debug!("Cannot close the last tab");
                return false;
            }

            // If we're closing the active tab, switch to another one first
            if let Some(active_index) = self.active_tab {
                if active_index == index {
                    // Switch to the previous tab if available, otherwise the next one
                    let new_active = if index > 0 { index - 1 } else { index + 1 };
                    self.active_tab = Some(new_active);
                } else if active_index > index {
                    // Update active tab index if it's after the removed tab
                    self.active_tab = Some(active_index - 1);
                }
            }

            // Remove the tab
            self.tabs.remove(index);
            debug!("Closed tab: {}", id);
            true
        } else {
            debug!("Tab {} does not exist", id);
            false
        }
    }

    pub fn switch_to_tab(&mut self, id: usize) -> bool {
        if let Some(index) = self.tabs.iter().position(|tab| tab.id == id) {
            self.active_tab = Some(index);
            debug!("Switched to tab: {}", id);
            true
        } else {
            debug!("Tab {} does not exist", id);
            false
        }
    }

    pub fn get_active_tab(&self) -> Option<&Tab> {
        self.active_tab.map(|index| &self.tabs[index])
    }

    pub fn get_active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab.map(|index| &mut self.tabs[index])
    }

    pub fn get_all_tabs(&self) -> Vec<&Tab> {
        self.tabs.iter().collect()
    }

    pub fn update_tab_title(&mut self, id: usize, title: String) -> bool {
        if id < self.tabs.len() {
            self.tabs[id].title = title;
            debug!("Updated title for tab {}: {}", id, self.tabs[id].title);
            true
        } else {
            false
        }
    }

    pub fn update_tab_url(&mut self, id: usize, url: String) -> bool {
        if id < self.tabs.len() {
            self.tabs[id].url = url;
            debug!("Updated URL for tab {}: {}", id, self.tabs[id].url);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_creation() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com".to_string());
        assert_eq!(manager.tabs.len(), 1);
        assert_eq!(manager.active_tab, Some(0));
    }

    #[test]
    fn test_tab_switching() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab("https://example1.com".to_string());
        let id2 = manager.create_tab("https://example2.com".to_string());
        
        assert!(manager.switch_to_tab(id1));
        assert_eq!(manager.active_tab, Some(0));
        
        assert!(manager.switch_to_tab(id2));
        assert_eq!(manager.active_tab, Some(1));
    }

    #[test]
    fn test_tab_closing() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab("https://example1.com".to_string());
        let id2 = manager.create_tab("https://example2.com".to_string());
        
        assert_eq!(manager.tabs.len(), 2);
        
        // Close first tab should succeed
        assert!(manager.close_tab(id1));
        assert!(manager.tabs.iter().position(|tab| tab.id == id1).is_none());
        assert_eq!(manager.tabs.len(), 1);
        
        // Cannot close the last tab
        assert!(!manager.close_tab(id2));
        assert!(manager.tabs.iter().position(|tab| tab.id == id2).is_some());
        assert_eq!(manager.tabs.len(), 1);

        // Cannot close non-existent tab
        assert!(!manager.close_tab(999));
    }

    #[test]
    fn test_tab_title_update() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com".to_string());
        
        assert!(manager.update_tab_title(id, "New Title".to_string()));
        if let Some(tab) = manager.get_active_tab() {
            assert_eq!(tab.title, "New Title");
        }
        
        // Test updating non-existent tab
        assert!(!manager.update_tab_title(999, "Invalid".to_string()));
    }

    #[test]
    fn test_tab_url_update() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com".to_string());
        
        assert!(manager.update_tab_url(id, "https://new-example.com".to_string()));
        if let Some(tab) = manager.get_active_tab() {
            assert_eq!(tab.url, "https://new-example.com");
        }
        
        // Test updating non-existent tab
        assert!(!manager.update_tab_url(999, "https://invalid.com".to_string()));
    }
} 
