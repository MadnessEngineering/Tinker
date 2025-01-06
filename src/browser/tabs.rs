use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct Tab {
    pub id: usize,
    pub url: String,
    pub title: String,
}

pub struct TabManager {
    tabs: HashMap<usize, Tab>,
    active_tab: usize,
    next_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        let mut manager = TabManager {
            tabs: HashMap::new(),
            active_tab: 0,
            next_id: 0,
        };
        // Create initial tab
        manager.create_tab("about:blank".to_string());
        manager
    }

    pub fn create_tab(&mut self, url: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let tab = Tab {
            id,
            url,
            title: String::from("New Tab"),
        };

        self.tabs.insert(id, tab);
        self.active_tab = id;
        debug!("Created new tab with id: {}", id);
        id
    }

    pub fn close_tab(&mut self, id: usize) -> bool {
        if self.tabs.len() <= 1 {
            debug!("Cannot close last tab");
            return false;
        }

        if let Some(_) = self.tabs.remove(&id) {
            if self.active_tab == id {
                // Switch to another tab
                if let Some(&first_id) = self.tabs.keys().next() {
                    self.active_tab = first_id;
                }
            }
            debug!("Closed tab: {}", id);
            true
        } else {
            false
        }
    }

    pub fn switch_to_tab(&mut self, id: usize) -> bool {
        if self.tabs.contains_key(&id) {
            self.active_tab = id;
            debug!("Switched to tab: {}", id);
            true
        } else {
            false
        }
    }

    pub fn get_active_tab(&self) -> Option<&Tab> {
        self.tabs.get(&self.active_tab)
    }

    pub fn get_active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(&self.active_tab)
    }

    pub fn get_all_tabs(&self) -> Vec<&Tab> {
        self.tabs.values().collect()
    }

    pub fn update_tab_title(&mut self, id: usize, title: String) -> bool {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.title = title;
            debug!("Updated title for tab {}: {}", id, tab.title);
            true
        } else {
            false
        }
    }

    pub fn update_tab_url(&mut self, id: usize, url: String) -> bool {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.url = url;
            debug!("Updated URL for tab {}: {}", id, tab.url);
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
        assert_eq!(manager.tabs.len(), 2); // Including initial tab
        assert_eq!(manager.active_tab, id);
    }

    #[test]
    fn test_tab_switching() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab("https://example1.com".to_string());
        let id2 = manager.create_tab("https://example2.com".to_string());
        
        assert!(manager.switch_to_tab(id1));
        assert_eq!(manager.active_tab, id1);
        
        assert!(manager.switch_to_tab(id2));
        assert_eq!(manager.active_tab, id2);
    }

    #[test]
    fn test_tab_closing() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab("https://example1.com".to_string());
        let id2 = manager.create_tab("https://example2.com".to_string());
        
        assert!(manager.close_tab(id1));
        assert!(!manager.tabs.contains_key(&id1));
        
        // Cannot close last tab
        assert!(!manager.close_tab(id2));
        assert!(manager.tabs.contains_key(&id2));
    }
} 
