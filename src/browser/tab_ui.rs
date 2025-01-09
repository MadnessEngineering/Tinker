use tao::window::Window;
use wry::{WebView, WebViewBuilder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TabCommand {
    Create { url: String },
    Close { id: usize },
    Switch { id: usize },
}

#[derive(Clone)]
pub struct TabBar {
    height: i32,
    tabs: Arc<Mutex<WebView>>,
}

impl TabBar {
    pub fn new(window: &Window, on_command: impl Fn(TabCommand) + Send + 'static) -> Result<Self, Box<dyn std::error::Error>> {
        let tabs = WebViewBuilder::new(window)
            .with_initialization_script(include_str!("../templates/tab_bar.js"))
            .with_html(include_str!("../templates/tab_bar.html"))?
            .with_ipc_handler(move |msg: String| {
                if let Ok(cmd) = serde_json::from_str::<TabCommand>(&msg) {
                    on_command(cmd);
                }
            })
            .build()?;

        let tabs = Arc::new(Mutex::new(tabs));
        Ok(TabBar {
            height: 30,
            tabs,
        })
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn update_tab_url(&self, id: usize, url: &str) {
        if let Ok(tabs) = &self.tabs.lock() {
            let js = format!(
                r#"
                const tab = document.querySelector(`[data-tab-id="{}"]`);
                if (tab) {{
                    tab.setAttribute('data-url', '{}');
                }}
                "#,
                id, url
            );
            let _ = tabs.evaluate_script(&js);
        }
    }

    pub fn update_tab_title(&self, id: usize, title: &str) {
        if let Ok(tabs) = &self.tabs.lock() {
            let js = format!(
                r#"
                const tab = document.querySelector(`[data-tab-id="{}"]`);
                if (tab) {{
                    tab.querySelector('.tab-title').textContent = '{}';
                }}
                "#,
                id, title
            );
            let _ = tabs.evaluate_script(&js);
        }
    }

    pub fn add_tab(&self, id: usize, title: &str, url: &str) {
        if let Ok(tabs) = &self.tabs.lock() {
            let js = format!(
                r#"
                addTab({}, '{}', '{}');
                "#,
                id, title, url
            );
            let _ = tabs.evaluate_script(&js);
        }
    }

    pub fn remove_tab(&self, id: usize) {
        if let Ok(tabs) = &self.tabs.lock() {
            let js = format!(
                r#"
                removeTab({});
                "#,
                id
            );
            let _ = tabs.evaluate_script(&js);
        }
    }

    pub fn set_active_tab(&self, id: usize) {
        if let Ok(tabs) = &self.tabs.lock() {
            let js = format!(
                r#"
                setActiveTab({});
                "#,
                id
            );
            let _ = tabs.evaluate_script(&js);
        }
    }
} 
