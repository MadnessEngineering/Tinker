use tao::window::Window;
use std::sync::mpsc::Sender;
use wry::{WebView, WebViewBuilder};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TabBar {
    webview: Arc<Mutex<WebView>>,
    cmd_tx: Sender<TabCommand>,
}

impl TabBar {
    pub fn new(window: &Arc<Window>, cmd_tx: Sender<TabCommand>) -> Result<Self, Box<dyn std::error::Error>> {
        let tab_height: u32 = 40;
        let cmd_tx_clone = cmd_tx.clone();
        let webview = WebViewBuilder::new(window)
            .with_bounds(wry::Rect {
                x: 0_i32,
                y: 0_i32,
                width: window.inner_size().width,
                height: tab_height,
            })
            .with_initialization_script(include_str!("../templates/tab_bar.js"))
            .with_html(include_str!("../templates/tab_bar.html"))?
            .with_ipc_handler(move |msg: String| {
                if let Ok(cmd) = serde_json::from_str::<TabCommand>(&msg) {
                    let _ = cmd_tx_clone.send(cmd);
                }
            })
            .build()?;

        Ok(Self {
            webview: Arc::new(Mutex::new(webview)),
            cmd_tx,
        })
    }

    pub fn add_tab(&self, id: u32, title: &str, url: &str) {
        if let Ok(webview) = self.webview.lock() {
            let _ = webview.evaluate_script(&format!(
                "addTab({}, '{}', '{}');",
                id,
                title.replace('\'', "\\'"),
                url.replace('\'', "\\'")
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
        if let Ok(webview) = self.webview.lock() {
            webview.set_bounds(wry::Rect {
                x: 0_i32,
                y: 0_i32,
                width: window.inner_size().width,
                height: 40,
            });
        }
    }

    pub fn update_tab_url(&self, id: usize, url: &str) {
        if let Ok(webview) = self.webview.lock() {
            let js = format!(
                r#"
                const tab = document.querySelector(`[data-tab-id="{}"]`);
                if (tab) {{
                    tab.setAttribute('data-url', '{}');
                }}
                "#,
                id,
                url.replace('\'', "\\'")
            );
            let _ = webview.evaluate_script(&js);
        }
    }

    pub fn update_tab_title(&self, id: usize, title: &str) {
        if let Ok(webview) = self.webview.lock() {
            let js = format!(
                r#"
                const tab = document.querySelector(`[data-tab-id="{}"]`);
                if (tab) {{
                    tab.querySelector('.tab-title').textContent = '{}';
                }}
                "#,
                id,
                title.replace('\'', "\\'")
            );
            let _ = webview.evaluate_script(&js);
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
} 
