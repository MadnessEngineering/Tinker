use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct Tab {
    pub id: usize,
    pub url: String,
    pub title: String,
}

#[derive(Default)]
pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
    next_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        let mut manager = TabManager {
            tabs: Vec::new(),
            active_tab: None,
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

        self.tabs.push(tab);
        self.active_tab = Some(id);
        debug!("Created new tab with id: {}", id);
        id
    }

    pub fn close_tab(&mut self, id: usize) -> bool {
        // Cannot close a non-existent tab
        if id >= self.tabs.len() {
            debug!("Tab {} does not exist", id);
            return false;
        }

        // Cannot close the last tab
        if self.tabs.len() <= 1 {
            debug!("Cannot close last tab");
            return false;
        }

        // If we're closing the active tab, switch to another one first
        if self.active_tab == Some(id) {
            if let Some(other_id) = self.tabs.iter().position(|tab| tab.id != id) {
                self.active_tab = Some(other_id);
            }
        }

        // Remove the tab
        self.tabs.remove(id);
        debug!("Closed tab: {}", id);
        true
    }

    pub fn switch_to_tab(&mut self, id: usize) -> bool {
        if id < self.tabs.len() {
            self.active_tab = Some(id);
            debug!("Switched to tab: {}", id);
            true
        } else {
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
        assert_eq!(manager.tabs.len(), 2); // Including initial tab
        assert_eq!(manager.active_tab, Some(id));
    }

    #[test]
    fn test_tab_switching() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab("https://example1.com".to_string());
        let id2 = manager.create_tab("https://example2.com".to_string());
        
        assert!(manager.switch_to_tab(id1));
        assert_eq!(manager.active_tab, Some(id1));
        
        assert!(manager.switch_to_tab(id2));
        assert_eq!(manager.active_tab, Some(id2));
    }

    #[test]
    fn test_tab_closing() {
        let mut manager = TabManager::new();
        let initial_tab = manager.get_active_tab().unwrap().id;
        let id1 = manager.create_tab("https://example1.com".to_string());
        let id2 = manager.create_tab("https://example2.com".to_string());
        
        assert_eq!(manager.tabs.len(), 3); // Initial + 2 new tabs
        
        // Close first new tab should succeed
        assert!(manager.close_tab(id1));
        assert!(manager.tabs.iter().position(|tab| tab.id == id1).is_none());
        assert_eq!(manager.tabs.len(), 2);
        
        // Close second new tab should succeed
        assert!(manager.close_tab(id2));
        assert!(manager.tabs.iter().position(|tab| tab.id == id2).is_none());
        assert_eq!(manager.tabs.len(), 1);
        
        // Cannot close the initial tab (last remaining tab)
        assert!(!manager.close_tab(initial_tab));
        assert!(manager.tabs.iter().position(|tab| tab.id == initial_tab).is_some());
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
