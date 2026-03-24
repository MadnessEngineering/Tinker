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
    pub history: Vec<String>,
    pub history_index: usize,
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
    tabs: HashMap<usize, Tab>,
    active_tab: Option<usize>,
    next_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        TabManager {
            tabs: HashMap::new(),
            active_tab: None,
            next_id: 0,
        }
    }

    pub fn create_tab(&mut self, url: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let tab = Tab {
            id,
            url: url.clone(),
            title: String::new(),
            webview: None,
            history: vec![url],
            history_index: 0,
        };

        self.tabs.insert(id, tab);
        self.active_tab = Some(id);
        id
    }

    pub fn get_tab_info(&self, id: usize) -> Option<&Tab> {
        self.tabs.get(&id)
    }

    pub fn get_tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn get_active_tab(&self) -> Option<&Tab> {
        self.active_tab.and_then(|id| self.tabs.get(&id))
    }

    pub fn get_active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab.and_then(move |id| self.tabs.get_mut(&id))
    }

    pub fn get_tab(&self, id: usize) -> Option<&Tab> {
        self.tabs.get(&id)
    }

    pub fn get_all_tabs(&self) -> Vec<&Tab> {
        self.tabs.values().collect()
    }

    pub fn get_tab_webview(&self, id: usize) -> Option<Arc<Mutex<WebView>>> {
        self.tabs.get(&id).and_then(|tab| tab.webview.clone())
    }

    pub fn set_tab_webview(&mut self, id: usize, webview: Arc<Mutex<WebView>>) -> bool {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.webview = Some(webview);
            true
        } else {
            false
        }
    }

    pub fn close_tab(&mut self, id: usize) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        if self.tabs.remove(&id).is_some() {
            if Some(id) == self.active_tab {
                self.active_tab = self.tabs.keys().next().copied();
            }
            true
        } else {
            false
        }
    }

    pub fn switch_to_tab(&mut self, id: usize) -> bool {
        if self.tabs.contains_key(&id) {
            self.active_tab = Some(id);
            true
        } else {
            false
        }
    }

    /// Push a URL onto a tab's history, discarding any forward history.
    pub fn push_history(&mut self, id: usize, url: String) -> bool {
        if let Some(tab) = self.tabs.get_mut(&id) {
            // Drop any forward history when navigating to a new URL
            tab.history.truncate(tab.history_index + 1);
            tab.history.push(url.clone());
            tab.history_index = tab.history.len() - 1;
            tab.url = url;
            true
        } else {
            false
        }
    }

    /// Navigate back one step; returns the URL moved to, or None if already at start.
    pub fn go_back(&mut self, id: usize) -> Option<String> {
        if let Some(tab) = self.tabs.get_mut(&id) {
            if tab.history_index > 0 {
                tab.history_index -= 1;
                let url = tab.history[tab.history_index].clone();
                tab.url = url.clone();
                Some(url)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Navigate forward one step; returns the URL moved to, or None if at end.
    pub fn go_forward(&mut self, id: usize) -> Option<String> {
        if let Some(tab) = self.tabs.get_mut(&id) {
            if tab.history_index + 1 < tab.history.len() {
                tab.history_index += 1;
                let url = tab.history[tab.history_index].clone();
                tab.url = url.clone();
                Some(url)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn can_go_back(&self, id: usize) -> bool {
        self.tabs.get(&id).map(|t| t.history_index > 0).unwrap_or(false)
    }

    pub fn can_go_forward(&self, id: usize) -> bool {
        self.tabs.get(&id)
            .map(|t| t.history_index + 1 < t.history.len())
            .unwrap_or(false)
    }

    pub fn update_tab_title(&mut self, id: usize, title: String) -> bool {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.title = title;
            true
        } else {
            false
        }
    }

    pub fn update_tab_url(&mut self, id: usize, url: String) -> bool {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.url = url;
            true
        } else {
            false
        }
    }

    pub fn get_tab_mut(&mut self, id: usize) -> Option<&mut Tab> {
        self.tabs.get_mut(&id)
    }

    pub fn is_active_tab(&self, id: usize) -> bool {
        Some(id) == self.active_tab
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_creation() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com".to_string());
        assert_eq!(manager.get_tab_count(), 1);
        
        let tab = manager.get_tab_info(id).unwrap();
        assert_eq!(tab.url, "https://example.com");
    }

    #[test]
    fn test_tab_switching() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab("https://example1.com".to_string());
        let id2 = manager.create_tab("https://example2.com".to_string());
        
        assert!(manager.switch_to_tab(id1));
        assert_eq!(manager.get_active_tab().unwrap().id, id1);
        
        assert!(manager.switch_to_tab(id2));
        assert_eq!(manager.get_active_tab().unwrap().id, id2);
    }

    #[test]
    fn test_tab_closing() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab("https://example1.com".to_string());
        let id2 = manager.create_tab("https://example2.com".to_string());
        
        assert!(manager.close_tab(id1));
        assert_eq!(manager.get_tab_count(), 1);
        assert_eq!(manager.get_active_tab().unwrap().id, id2);
    }

    #[test]
    fn test_tab_title_update() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com".to_string());
        
        assert!(manager.update_tab_title(id, "New Title".to_string()));
        assert_eq!(manager.get_tab_info(id).unwrap().title, "New Title");
    }

    #[test]
    fn test_tab_url_update() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com".to_string());
        
        assert!(manager.update_tab_url(id, "https://new-url.com".to_string()));
        assert_eq!(manager.get_tab_info(id).unwrap().url, "https://new-url.com");
    }

    #[test]
    fn test_history_push_and_back() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com".to_string());

        manager.push_history(id, "https://example.com/page1".to_string());
        manager.push_history(id, "https://example.com/page2".to_string());

        assert!(manager.can_go_back(id));
        assert!(!manager.can_go_forward(id));

        let url = manager.go_back(id).unwrap();
        assert_eq!(url, "https://example.com/page1");
        assert_eq!(manager.get_tab_info(id).unwrap().url, "https://example.com/page1");

        let url = manager.go_back(id).unwrap();
        assert_eq!(url, "https://example.com");
        assert!(!manager.can_go_back(id));

        // At the start — go_back returns None
        assert!(manager.go_back(id).is_none());
    }

    #[test]
    fn test_history_forward() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com".to_string());
        manager.push_history(id, "https://example.com/page1".to_string());
        manager.push_history(id, "https://example.com/page2".to_string());

        manager.go_back(id);
        manager.go_back(id);

        assert!(manager.can_go_forward(id));
        let url = manager.go_forward(id).unwrap();
        assert_eq!(url, "https://example.com/page1");

        let url = manager.go_forward(id).unwrap();
        assert_eq!(url, "https://example.com/page2");
        assert!(!manager.can_go_forward(id));
        assert!(manager.go_forward(id).is_none());
    }

    #[test]
    fn test_history_truncates_on_new_navigate() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com".to_string());
        manager.push_history(id, "https://example.com/a".to_string());
        manager.push_history(id, "https://example.com/b".to_string());

        // Go back to /a, then navigate to /c — /b should be dropped
        manager.go_back(id);
        manager.push_history(id, "https://example.com/c".to_string());

        assert!(!manager.can_go_forward(id));
        let url = manager.go_back(id).unwrap();
        assert_eq!(url, "https://example.com/a");
    }

    #[test]
    fn test_get_all_tabs() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab("https://example1.com".to_string());
        let id2 = manager.create_tab("https://example2.com".to_string());
        
        let tabs = manager.get_all_tabs();
        assert_eq!(tabs.len(), 2);
        assert!(tabs.iter().any(|tab| tab.id == id1));
        assert!(tabs.iter().any(|tab| tab.id == id2));
    }
} 
