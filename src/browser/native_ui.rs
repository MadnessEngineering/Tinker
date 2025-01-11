use tao::window::Window;
use std::sync::Mutex;
use anyhow::Result;
use tao::event_loop::EventLoop;
use tao::window::{Window, WindowBuilder};
use tao::platform::windows::EventLoopBuilderExtWindows;

#[derive(Debug)]
pub struct NativeTab {
    pub id: usize,
    pub title: String,
    pub active: bool,
}

pub struct NativeTabBar {
    tabs: Mutex<Vec<NativeTab>>,
    height: u32,
}

impl NativeTabBar {
    pub fn new(_window: &Window) -> Result<Self> {
        Ok(Self {
            tabs: Mutex::new(Vec::new()),
            height: 40,
        })
    }

    pub fn add_tab(&self, id: usize, title: &str) {
        let mut tabs = self.tabs.lock().unwrap();
        tabs.push(NativeTab {
            id,
            title: title.to_string(),
            active: false,
        });
    }

    pub fn remove_tab(&self, id: usize) {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(pos) = tabs.iter().position(|tab| tab.id == id) {
            tabs.remove(pos);
        }
    }

    pub fn set_active_tab(&self, id: usize) {
        let mut tabs = self.tabs.lock().unwrap();
        for tab in tabs.iter_mut() {
            tab.active = tab.id == id;
        }
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tao::event_loop::EventLoopBuilder;
    use tao::platform::windows::EventLoopBuilderExtWindows;

    fn create_test_window() -> (EventLoop<()>, Window) {
        let event_loop = EventLoopBuilder::new()
            .with_any_thread(true)
            .build();
        let window = WindowBuilder::new()
            .with_title("Test Window")
            .build(&event_loop)
            .unwrap();
        (event_loop, window)
    }

    #[test]
    fn test_native_tab_bar_creation() {
        let (_event_loop, window) = create_test_window();
        let tab_bar = NativeTabBar::new(&window).unwrap();
        assert_eq!(tab_bar.get_height(), 30);
    }

    #[test]
    fn test_tab_management() {
        let (_event_loop, window) = create_test_window();
        let mut tab_bar = NativeTabBar::new(&window).unwrap();
        tab_bar.add_tab(0, "Test Tab");
        tab_bar.set_active_tab(0);
        tab_bar.remove_tab(0);
    }

    #[test]
    fn test_multiple_tabs() {
        let (_event_loop, window) = create_test_window();
        let mut tab_bar = NativeTabBar::new(&window).unwrap();
        tab_bar.add_tab(0, "Tab 1");
        tab_bar.add_tab(1, "Tab 2");
        tab_bar.add_tab(2, "Tab 3");
        tab_bar.set_active_tab(1);
        tab_bar.remove_tab(0);
        tab_bar.remove_tab(2);
    }
} 
