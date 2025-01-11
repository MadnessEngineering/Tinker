use std::collections::HashMap;
use tracing::debug;
use anyhow::Result;

#[derive(Debug)]
pub struct Tab {
    pub id: u32,
    pub url: String,
    pub title: String,
}

impl Tab {
    pub fn new(id: u32, url: String) -> Self {
        Self {
            id,
            url: url.clone(),
            title: url,
        }
    }
}

#[derive(Debug)]
pub struct TabManager {
    tabs: HashMap<u32, Tab>,
    active_tab: Option<u32>,
    next_id: u32,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            active_tab: None,
            next_id: 1,
        }
    }

    pub fn create_tab(&mut self, url: &str) -> Result<u32> {
        let id = self.next_id;
        self.next_id += 1;

        let tab = Tab::new(id, url.to_string());
        self.tabs.insert(id, tab);

        if self.active_tab.is_none() {
            self.active_tab = Some(id);
        }

        Ok(id)
    }

    pub fn close_tab(&mut self, id: u32) -> Result<()> {
        if self.tabs.remove(&id).is_some() {
            if Some(id) == self.active_tab {
                self.active_tab = self.tabs.keys().next().copied();
            }
            Ok(())
        } else {
            Ok(()) // Tab doesn't exist, silently succeed
        }
    }

    pub fn get_tab(&self, id: u32) -> Option<&Tab> {
        self.tabs.get(&id)
    }

    pub fn get_tab_index(&self, id: u32) -> Option<usize> {
        self.tabs.keys()
            .enumerate()
            .find(|(_, &tab_id)| tab_id == id)
            .map(|(index, _)| index)
    }

    pub fn get_active_tab(&self) -> Option<&Tab> {
        self.active_tab.and_then(|id| self.tabs.get(&id))
    }

    pub fn set_active_tab(&mut self, id: u32) -> Result<()> {
        if self.tabs.contains_key(&id) {
            self.active_tab = Some(id);
            Ok(())
        } else {
            Ok(()) // Tab doesn't exist, silently succeed
        }
    }

    pub fn update_tab_url(&mut self, id: u32, url: String) -> Result<()> {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.url = url;
            Ok(())
        } else {
            Ok(()) // Tab doesn't exist, silently succeed
        }
    }

    pub fn update_tab_title(&mut self, id: u32, title: String) -> Result<()> {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.title = title;
            Ok(())
        } else {
            Ok(()) // Tab doesn't exist, silently succeed
        }
    }

    pub fn get_tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn get_all_tabs(&self) -> Vec<&Tab> {
        self.tabs.values().collect()
    }

    pub fn get_tab_mut(&mut self, id: u32) -> Option<&mut Tab> {
        self.tabs.get_mut(&id)
    }

    pub fn is_active_tab(&self, id: u32) -> bool {
        Some(id) == self.active_tab
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_creation() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com").unwrap();
        assert_eq!(manager.get_tab_count(), 1);
        
        let tab = manager.get_tab(id).unwrap();
        assert_eq!(tab.url, "https://example.com");
    }

    #[test]
    fn test_tab_switching() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab("https://example1.com").unwrap();
        let id2 = manager.create_tab("https://example2.com").unwrap();
        
        assert!(manager.set_active_tab(id1).is_ok());
        assert_eq!(manager.get_active_tab().unwrap().id, id1);
        
        assert!(manager.set_active_tab(id2).is_ok());
        assert_eq!(manager.get_active_tab().unwrap().id, id2);
    }

    #[test]
    fn test_tab_closing() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab("https://example1.com").unwrap();
        let id2 = manager.create_tab("https://example2.com").unwrap();
        
        assert!(manager.close_tab(id1).is_ok());
        assert_eq!(manager.get_tab_count(), 1);
        assert_eq!(manager.get_active_tab().unwrap().id, id2);
    }

    #[test]
    fn test_tab_title_update() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com").unwrap();
        
        assert!(manager.update_tab_title(id, "New Title".to_string()).is_ok());
        assert_eq!(manager.get_tab(id).unwrap().title, "New Title");
    }

    #[test]
    fn test_tab_url_update() {
        let mut manager = TabManager::new();
        let id = manager.create_tab("https://example.com").unwrap();
        
        assert!(manager.update_tab_url(id, "https://new-url.com".to_string()).is_ok());
        assert_eq!(manager.get_tab(id).unwrap().url, "https://new-url.com");
    }

    #[test]
    fn test_get_all_tabs() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab("https://example1.com").unwrap();
        let id2 = manager.create_tab("https://example2.com").unwrap();
        
        let tabs = manager.get_all_tabs();
        assert_eq!(tabs.len(), 2);
        assert!(tabs.iter().any(|t| t.id == id1));
        assert!(tabs.iter().any(|t| t.id == id2));
    }
} 
