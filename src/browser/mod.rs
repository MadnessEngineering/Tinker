//! Browser engine implementation

use wry::{WebView, WebViewBuilder, Result as WryResult};
use tracing::{info, error, debug};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use wry::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

mod keyboard;
use keyboard::{KeyCommand, handle_keyboard_input, KeyCode, ModifiersState};

struct DummyWindow;

unsafe impl HasRawWindowHandle for DummyWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        #[cfg(target_os = "macos")]
        {
            use wry::raw_window_handle::AppKitWindowHandle;
            let mut handle = AppKitWindowHandle::empty();
            handle.ns_window = std::ptr::null_mut();
            handle.ns_view = std::ptr::null_mut();
            RawWindowHandle::AppKit(handle)
        }
        #[cfg(not(target_os = "macos"))]
        unimplemented!("Platform not supported")
    }
}

pub struct BrowserEngine {
    window: Option<WebView>,
    title: String,
    history: Vec<String>,
    current_index: usize,
    tabs: HashMap<usize, Arc<Mutex<Tab>>>,
    active_tab: usize,
    next_tab_id: usize,
    keyboard_tx: Option<Sender<(KeyCode, ModifiersState)>>,
    keyboard_rx: Option<Receiver<(KeyCode, ModifiersState)>>,
}

struct Tab {
    webview: Option<WebView>,
    title: String,
    url: String,
}

impl BrowserEngine {
    pub fn forge(headless: bool) -> WryResult<Self> {
        info!("Forging new browser engine...");
        
        let initial_tab = Arc::new(Mutex::new(Tab {
            webview: None,
            title: String::from("New Tab"),
            url: String::from("about:blank"),
        }));

        let mut tabs = HashMap::new();
        tabs.insert(0, initial_tab);

        let (tx, rx) = channel();

        let mut engine = Self {
            window: None,
            title: String::from("Tinker Workshop"),
            history: vec![String::from("about:blank")],
            current_index: 0,
            tabs,
            active_tab: 0,
            next_tab_id: 1,
            keyboard_tx: Some(tx),
            keyboard_rx: Some(rx),
        };

        if !headless {
            engine.initialize_webviews()?;
        }
        
        Ok(engine)
    }

    fn initialize_webviews(&mut self) -> WryResult<()> {
        let dummy = DummyWindow;
        let window = WebViewBuilder::new(&dummy)
            .with_url("about:blank")?
            .build()?;

        self.window = Some(window);

        if let Some(tab) = self.tabs.get_mut(&0) {
            let mut tab = tab.lock().unwrap();
            tab.webview = Some(
                WebViewBuilder::new(&dummy)
                    .with_url("about:blank")?
                    .build()?
            );
        }

        Ok(())
    }

    pub fn navigate(&mut self, url: &str) -> WryResult<()> {
        info!("Navigating to: {}", url);
        self.history.truncate(self.current_index + 1);
        self.history.push(url.to_string());
        self.current_index = self.history.len() - 1;
        
        if let Some(tab) = self.tabs.get(&self.active_tab) {
            let tab = tab.lock().unwrap();
            if let Some(webview) = &tab.webview {
                webview.load_url(url);
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn back(&mut self) -> WryResult<()> {
        if self.current_index > 0 {
            self.current_index -= 1;
            let url = self.history[self.current_index].clone();
            debug!("Navigating back to: {}", url);
            self.navigate(&url)
        } else {
            debug!("No previous page in history");
            Ok(())
        }
    }

    pub fn forward(&mut self) -> WryResult<()> {
        if self.current_index < self.history.len() - 1 {
            self.current_index += 1;
            let url = self.history[self.current_index].clone();
            debug!("Navigating forward to: {}", url);
            self.navigate(&url)
        } else {
            debug!("No next page in history");
            Ok(())
        }
    }

    pub fn refresh(&self) -> WryResult<()> {
        debug!("Refreshing current page");
        if let Some(tab) = self.tabs.get(&self.active_tab) {
            let tab = tab.lock().unwrap();
            if let Some(webview) = &tab.webview {
                return webview.evaluate_script("window.location.reload()");
            }
        }
        Ok(())
    }

    pub fn new_tab(&mut self, url: Option<String>) -> WryResult<usize> {
        let url = url.unwrap_or_else(|| String::from("about:blank"));
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let tab = Arc::new(Mutex::new(Tab {
            webview: None,
            title: String::from("New Tab"),
            url: url.clone(),
        }));

        self.tabs.insert(tab_id, tab);
        self.initialize_tab_webview(tab_id, true)?;
        info!("Created new tab with ID: {}", tab_id);
        Ok(tab_id)
    }

    pub fn close_tab(&mut self, tab_id: usize) -> WryResult<()> {
        if self.tabs.len() > 1 {
            self.tabs.remove(&tab_id);
            info!("Closed tab: {}", tab_id);
            
            if self.active_tab == tab_id {
                self.active_tab = *self.tabs.keys().max().unwrap_or(&0);
            }
            Ok(())
        } else {
            error!("Cannot close the last tab");
            Ok(())
        }
    }

    pub fn switch_tab(&mut self, tab_id: usize) -> WryResult<()> {
        if self.tabs.contains_key(&tab_id) {
            self.active_tab = tab_id;
            info!("Switched to tab: {}", tab_id);
            Ok(())
        } else {
            error!("Tab {} does not exist", tab_id);
            Ok(())
        }
    }

    pub fn cleanup(&mut self) {
        debug!("Cleaning up browser engine resources");
        
        // Take ownership of all resources before dropping
        let window = self.window.take();
        let keyboard_tx = self.keyboard_tx.take();
        let keyboard_rx = self.keyboard_rx.take();
        let tabs = std::mem::take(&mut self.tabs);
        
        // Drop resources in a controlled order
        for (_, tab) in tabs {
            if let Ok(mut tab) = tab.lock() {
                tab.webview.take();
            }
        }
        
        drop(window);
        drop(keyboard_tx);
        drop(keyboard_rx);
    }

    pub fn run(&mut self) {
        if cfg!(test) {
            // In tests, we don't need to run the event loop
            return;
        }
        
        loop {
            // Collect all pending keyboard events
            let mut events = Vec::new();
            if let Some(rx) = &self.keyboard_rx {
                while let Ok(event) = rx.try_recv() {
                    events.push(event);
                }
            }

            // Process collected events
            for (key, modifiers) in events {
                if let Err(e) = self.handle_keyboard_event(key, modifiers) {
                    error!("Failed to handle keyboard event: {:?}", e);
                }
            }

            // Handle window operations
            if let Some(window) = &self.window {
                window.evaluate_script("").unwrap_or(());
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn initialize_tab_webview(&self, tab_id: usize, _headless: bool) -> WryResult<()> {
        if let Some(tab) = self.tabs.get(&tab_id) {
            let mut tab = tab.lock().unwrap();
            if tab.webview.is_none() {
                let dummy = DummyWindow;
                tab.webview = Some(
                    WebViewBuilder::new(&dummy)
                        .with_url(&tab.url)?
                        .build()?
                );
            }
        }
        Ok(())
    }

    pub fn handle_keyboard_event(&mut self, key: KeyCode, modifiers: ModifiersState) -> WryResult<()> {
        if let Some(command) = handle_keyboard_input(key, modifiers) {
            debug!("Handling keyboard command: {:?}", command);
            match command {
                KeyCommand::Back => self.back()?,
                KeyCommand::Forward => self.forward()?,
                KeyCommand::Refresh => self.refresh()?,
                KeyCommand::NewTab => {
                    self.new_tab(None)?;
                }
                KeyCommand::CloseTab => {
                    self.close_tab(self.active_tab)?;
                }
                KeyCommand::SwitchTab(index) => {
                    if index < self.tabs.len() {
                        self.switch_tab(index)?;
                    }
                }
                KeyCommand::FocusAddressBar => {
                    // TODO: Implement address bar focus
                    debug!("Address bar focus not implemented yet");
                }
                KeyCommand::StopLoading => {
                    if let Some(tab) = self.tabs.get(&self.active_tab) {
                        let tab = tab.lock().unwrap();
                        if let Some(webview) = &tab.webview {
                            webview.evaluate_script("window.stop()")?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Drop for BrowserEngine {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            let _subscriber = FmtSubscriber::builder()
                .with_max_level(Level::DEBUG)
                .try_init();
        });
    }

    #[test]
    fn test_browser_creation() {
        setup();
        let browser = BrowserEngine::forge(true);
        assert!(browser.is_ok());
    }

    #[test]
    fn test_navigation() {
        setup();
        let mut browser = BrowserEngine::forge(true).unwrap();
        assert!(browser.navigate("https://example.com").is_ok());
    }

    #[test]
    fn test_tab_management() {
        setup();
        let mut browser = BrowserEngine::forge(true).unwrap();
        let initial_tab_count = browser.tabs.len();
        assert!(browser.new_tab(None).is_ok());
        assert_eq!(browser.tabs.len(), initial_tab_count + 1);
    }

    #[test]
    fn test_keyboard_navigation() {
        setup();
        let mut browser = BrowserEngine::forge(true).unwrap();
        
        // Navigate to a page
        browser.navigate("https://example.com").unwrap();
        
        // Test back command with Alt+Left
        browser.handle_keyboard_event(
            KeyCode::ArrowLeft,
            ModifiersState { alt: true, ctrl: false, shift: false, meta: false }
        ).unwrap();
        
        // Test forward command with Alt+Right
        browser.handle_keyboard_event(
            KeyCode::ArrowRight,
            ModifiersState { alt: true, ctrl: false, shift: false, meta: false }
        ).unwrap();
    }

    #[test]
    fn test_keyboard_tabs() {
        setup();
        let mut browser = BrowserEngine::forge(true).unwrap();
        let initial_tab_count = browser.tabs.len();
        
        // Create new tab with Ctrl+T
        browser.handle_keyboard_event(
            KeyCode::KeyT,
            ModifiersState { alt: false, ctrl: true, shift: false, meta: false }
        ).unwrap();
        assert_eq!(browser.tabs.len(), initial_tab_count + 1);
        
        // Switch to first tab with Ctrl+1
        browser.handle_keyboard_event(
            KeyCode::Digit1,
            ModifiersState { alt: false, ctrl: true, shift: false, meta: false }
        ).unwrap();
        assert_eq!(browser.active_tab, 0);
        
        // Close current tab with Ctrl+W
        browser.handle_keyboard_event(
            KeyCode::KeyW,
            ModifiersState { alt: false, ctrl: true, shift: false, meta: false }
        ).unwrap();
        assert_eq!(browser.tabs.len(), initial_tab_count);
    }

    #[test]
    fn test_keyboard_refresh() {
        setup();
        let mut browser = BrowserEngine::forge(true).unwrap();
        
        // Test refresh with Ctrl+R
        assert!(browser.handle_keyboard_event(
            KeyCode::KeyR,
            ModifiersState { alt: false, ctrl: true, shift: false, meta: false }
        ).is_ok());
    }
} 
