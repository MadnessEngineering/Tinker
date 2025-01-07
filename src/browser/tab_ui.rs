use wry::{WebView, WebViewBuilder};
use tao::window::Window;
use crate::event::BrowserEvent;
use std::sync::mpsc::Sender;

pub enum TabCommand {
    Create { url: String },
    Close { id: usize },
    Switch { id: usize },
}

pub struct TabBar {
    container: WebView,
    height: u32,
}

impl TabBar {
    pub fn new(window: &Window, command_tx: Sender<TabCommand>) -> Result<Self, Box<dyn std::error::Error>> {
        let height = 40; // Height of the tab bar in pixels
        
        // Create a WebView for the tab bar with custom HTML/CSS
        let container = WebViewBuilder::new(window)
            .with_html(include_str!("../templates/tab_bar.html"))?
            .with_initialization_script(include_str!("../templates/tab_bar.js"))
            .with_ipc_handler(move |msg: String| {
                // Handle IPC messages from the tab bar UI
                if let Ok(event) = serde_json::from_str::<BrowserEvent>(&msg) {
                    match event {
                        BrowserEvent::TabCreated { .. } => {
                            let _ = command_tx.send(TabCommand::Create {
                                url: "about:blank".to_string()
                            });
                        }
                        BrowserEvent::TabClosed { id } => {
                            let _ = command_tx.send(TabCommand::Close { id });
                        }
                        BrowserEvent::TabSwitched { id } => {
                            let _ = command_tx.send(TabCommand::Switch { id });
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
