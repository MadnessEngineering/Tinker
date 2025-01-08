use wry::{WebView, WebViewBuilder};
use tao::window::Window;
use tracing::debug;
use std::sync::{Arc, Mutex};
use crate::templates::{TAB_BAR_HTML, TAB_BAR_JS};

pub enum TabCommand {
    Create { url: String },
    Close { id: usize },
    Switch { id: usize },
}

#[derive(Clone)]
pub struct TabBar {
    container: Arc<Mutex<WebView>>,
    height: u32,
}

impl TabBar {
    pub fn new<F>(window: &Window, command_handler: F) -> Result<Self, Box<dyn std::error::Error>>
    where
        F: Fn(TabCommand) + Send + 'static,
    {
        let height = 40; // Height of the tab bar in pixels
        
        // Create a WebView for the tab bar with custom HTML/CSS
        let webview = WebViewBuilder::new(window)
            .with_html(TAB_BAR_HTML)?
            .with_initialization_script(TAB_BAR_JS)
            .with_ipc_handler(move |value| {
                let msg = if let Some(s) = value.as_str() {
                    s
                } else {
                    debug!("Invalid IPC message format: {:?}", value);
                    return;
                };

                match msg.split_once(':') {
                    Some(("TabCreated", url)) => {
                        let url = url.trim()
                            .strip_prefix('"')
                            .and_then(|s| s.strip_suffix('"'))
                            .unwrap_or("about:blank")
                            .to_string();
                        debug!("Creating new tab with URL: {}", url);
                        command_handler(TabCommand::Create { url });
                    }
                    Some(("TabClosed", id)) => {
                        if let Ok(id) = id.trim().parse::<usize>() {
                            debug!("Closing tab with ID: {}", id);
                            command_handler(TabCommand::Close { id });
                        }
                    }
                    Some(("TabSwitched", id)) => {
                        if let Ok(id) = id.trim().parse::<usize>() {
                            debug!("Switching to tab with ID: {}", id);
                            command_handler(TabCommand::Switch { id });
                        }
                    }
                    _ => {
                        debug!("Unknown IPC message: {:?}", value);
                    }
                }
            })
            .build()?;

        Ok(TabBar {
            container: Arc::new(Mutex::new(webview)),
            height,
        })
    }

    pub fn add_tab(&self, id: usize, title: &str, url: &str) {
        let script = format!(
            "addTab({}, '{}', '{}');",
            id,
            title.replace('\'', "\\'"),
            url.replace('\'', "\\'")
        );
        if let Ok(container) = self.container.lock() {
            let _ = container.evaluate_script(&script);
        }
    }

    pub fn remove_tab(&self, id: usize) {
        let script = format!("removeTab({});", id);
        if let Ok(container) = self.container.lock() {
            let _ = container.evaluate_script(&script);
        }
    }

    pub fn set_active_tab(&self, id: usize) {
        let script = format!("setActiveTab({});", id);
        if let Ok(container) = self.container.lock() {
            let _ = container.evaluate_script(&script);
        }
    }
} 
