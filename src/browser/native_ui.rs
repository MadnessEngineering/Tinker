use tao::window::{Window, WindowBuilder};
use tao::dpi::LogicalSize;
use std::sync::{Arc, Mutex};

pub struct NativeTabBar {
    height: u32,
    tabs: Arc<Mutex<Vec<TabInfo>>>,
}

struct TabInfo {
    id: usize,
    title: String,
    active: bool,
}

impl NativeTabBar {
    pub fn new(parent: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(NativeTabBar {
            height: 40,
            tabs: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn add_tab(&mut self, id: usize, title: &str) {
        if let Ok(mut tabs) = self.tabs.lock() {
            tabs.push(TabInfo {
                id,
                title: title.to_string(),
                active: false,
            });
        }
    }

    pub fn remove_tab(&mut self, id: usize) {
        if let Ok(mut tabs) = self.tabs.lock() {
            if let Some(pos) = tabs.iter().position(|tab| tab.id == id) {
                tabs.remove(pos);
            }
        }
    }

    pub fn set_active_tab(&mut self, id: usize) {
        if let Ok(mut tabs) = self.tabs.lock() {
            for tab in tabs.iter_mut() {
                tab.active = tab.id == id;
            }
        }
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
} 
