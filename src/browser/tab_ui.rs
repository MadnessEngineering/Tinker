use wry::{WebView, WebViewBuilder};
use tao::window::Window;
use crate::event::BrowserEvent;

pub struct TabBar {
    container: WebView,
    height: u32,
}

impl TabBar {
    pub fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        let height = 40; // Height of the tab bar in pixels
        
        // Create a WebView for the tab bar with custom HTML/CSS
        let container = WebViewBuilder::new(window)
            .with_html(include_str!("../templates/tab_bar.html"))?
            .with_initialization_script(include_str!("../templates/tab_bar.js"))
            .with_ipc_handler(move |msg| {
                // Handle IPC messages from the tab bar UI
                if let Ok(event) = serde_json::from_str::<BrowserEvent>(&msg) {
                    match event {
                        BrowserEvent::TabCreated { id } => {
                            // Handle new tab creation
                        }
                        BrowserEvent::TabClosed { id } => {
                            // Handle tab closing
                        }
                        BrowserEvent::TabSwitched { id } => {
                            // Handle tab switching
                        }
                        _ => {}
                    }
                }
            })
            .build()?;

        Ok(TabBar {
            container,
            height,
        })
    }

    pub fn update_tab(&self, id: usize, title: &str, url: &str) {
        let script = format!(
            "updateTab({}, '{}', '{}');",
            id,
            title.replace("'", "\\'"),
            url.replace("'", "\\'")
        );
        let _ = self.container.evaluate_script(&script);
    }

    pub fn set_active_tab(&self, id: usize) {
        let script = format!("setActiveTab({});", id);
        let _ = self.container.evaluate_script(&script);
    }

    pub fn add_tab(&self, id: usize, title: &str, url: &str) {
        let script = format!(
            "addTab({}, '{}', '{}');",
            id,
            title.replace("'", "\\'"),
            url.replace("'", "\\'")
        );
        let _ = self.container.evaluate_script(&script);
    }

    pub fn remove_tab(&self, id: usize) {
        let script = format!("removeTab({});", id);
        let _ = self.container.evaluate_script(&script);
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
} 
