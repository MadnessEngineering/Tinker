use tao::window::Window;
use std::sync::mpsc::Sender;
use wry::{WebView, WebViewBuilder};
use std::sync::{Arc, Mutex};
use tracing::{debug, error};

/// Height of the top chrome (address bar + tab strip), in pixels. The content
/// view is positioned directly below it.
pub const CHROME_HEIGHT: u32 = 76;

#[derive(Clone)]
pub struct TabBar {
    webview: Arc<Mutex<WebView>>,
    cmd_tx: Sender<TabCommand>,
}

impl TabBar {
    pub fn new(window: &Window, cmd_tx: Sender<TabCommand>) -> Result<Self, String> {
        debug!("Creating new TabBar");
        
        // new_as_child so with_bounds/set_bounds take effect (bounds are ignored
        // for webviews created with the plain `new`, which fill the whole window).
        let webview = WebViewBuilder::new_as_child(window)
            .with_bounds(wry::Rect {
                x: 0_i32,
                y: 0_i32,
                width: window.inner_size().width,
                height: CHROME_HEIGHT,
            })
            .with_visible(true)
            .with_transparent(false)
            .with_ipc_handler({
                let cmd_tx = cmd_tx.clone();
                move |msg: String| {
                    match serde_json::from_str::<TabCommand>(&msg) {
                        Ok(cmd) => {
                            if let Err(e) = cmd_tx.send(cmd) {
                                error!("Failed to forward tab command: {}", e);
                            }
                        }
                        Err(e) => error!("Failed to parse tab command '{}': {}", msg, e),
                    }
                }
            })
            .with_initialization_script(include_str!("../templates/tab_bar.js"))
            .with_html(include_str!("../templates/tab_bar.html"))
            .map_err(|e| format!("Failed to create tab bar WebView: {}", e))?
            .build()
            .map_err(|e| format!("Failed to build tab bar WebView: {}", e))?;

        debug!("TabBar WebView created successfully");
        
        Ok(TabBar {
            webview: Arc::new(Mutex::new(webview)),
            cmd_tx,
        })
    }

    pub fn add_tab(&self, id: u32, title: &str, url: &str) {
        if let Ok(webview) = self.webview.lock() {
            // JS signature is createTab(id, url, title).
            let _ = webview.evaluate_script(&format!(
                "createTab({}, '{}', '{}');",
                id,
                url.replace('\'', "\\'"),
                title.replace('\'', "\\'")
            ));
        }
    }

    pub fn remove_tab(&self, id: u32) {
        if let Ok(webview) = self.webview.lock() {
            let _ = webview.evaluate_script(&format!("removeTab({});", id));
        }
    }

    pub fn set_active_tab(&self, id: u32) {
        if let Ok(webview) = self.webview.lock() {
            let _ = webview.evaluate_script(&format!("setActiveTab({});", id));
        }
    }

    pub fn update_bounds(&self, window: &Window) {
        if let Ok(view) = self.webview.lock() {
            view.set_bounds(wry::Rect {
                x: 0_i32,
                y: 0_i32,
                width: window.inner_size().width,
                height: CHROME_HEIGHT,
            });
            view.set_visible(true);
        }
    }

    /// Update the address-bar URL field (unless the user is editing it).
    pub fn update_address_bar(&self, url: &str) {
        if let Ok(view) = self.webview.lock() {
            let _ = view.evaluate_script(&format!(
                "updateAddressBar('{}');", url.replace('\'', "\\'")
            ));
        }
    }

    pub fn update_tab_url(&self, id: usize, url: &str) {
        if let Ok(view) = self.webview.lock() {
            // JS signature is updateTabUrl(id, url).
            if let Err(e) = view.evaluate_script(&format!(
                "updateTabUrl({}, '{}');", id, url.replace('\'', "\\'")
            )) {
                error!("Failed to update tab URL: {}", e);
            }
        }
    }

    pub fn update_tab_title(&self, id: usize, title: &str) {
        if let Ok(view) = self.webview.lock() {
            // JS signature is updateTabTitle(id, title).
            if let Err(e) = view.evaluate_script(&format!(
                "updateTabTitle({}, '{}');", id, title.replace('\'', "\\'")
            )) {
                error!("Failed to update tab title: {}", e);
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TabCommand {
    Create { url: String },
    Close { id: usize },
    Switch { id: usize },
    UpdateUrl { id: usize, url: String },
    UpdateTitle { id: usize, title: String },
    Navigate { url: String },
    Back,
    Forward,
    Reload,
} 
