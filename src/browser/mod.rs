//! Browser engine implementation

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tao::{
    event::{Event, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    dpi::LogicalSize,
};
use wry::{WebView, WebViewBuilder};
use tracing::{debug, info, error};
use crate::browser::error::BrowserResult;
use crate::browser::navigation::{NavigationManager, NavigationState};

#[derive(Debug, thiserror::Error)]
pub enum WebViewError {
    #[error("Failed to create WebView on Windows: {0}")]
    WindowsError(String),

    #[error("Failed to create WebView on macOS: {0}")]
    MacOSError(String),

    #[error("Failed to create WebView on Linux: {0}")]
    LinuxError(String),

    #[error("Failed to create WebView: {0}")]
    InitError(#[from] wry::Error),

    #[error("Failed to create window: {0}")]
    WindowError(String),

    #[error("Failed to create tab bar: {0}")]
    TabBarError(String),

    #[error("Failed to lock resource: {0}")]
    LockError(String),

    #[error("Tab operation failed: {0}")]
    TabError(String),

    #[error("Generic error: {0}")]
    GenericError(String),
}

pub mod error;
pub mod navigation;
pub mod state_manager;
pub mod window_manager;

/// Manages a browser tab's state and content
pub struct Tab {
    id: String,
    webview: WebView,
    navigation: NavigationManager,
}

impl Tab {
    pub fn new(id: String, webview: WebView) -> Self {
        Self {
            id,
            webview,
            navigation: NavigationManager::new(),
        }
    }

    /// Navigate to a URL or search query
    pub fn navigate(&self, input: &str) -> BrowserResult<()> {
        let parsed_url = self.navigation.navigate(input)?;
        self.webview.load_url(parsed_url.url().as_str());
        Ok(())
    }

    /// Navigate back in history
    pub fn go_back(&self) -> BrowserResult<()> {
        if let Some(parsed_url) = self.navigation.go_back()? {
            self.webview.load_url(parsed_url.url().as_str());
        }
        Ok(())
    }

    /// Navigate forward in history
    pub fn go_forward(&self) -> BrowserResult<()> {
        if let Some(parsed_url) = self.navigation.go_forward()? {
            self.webview.load_url(parsed_url.url().as_str());
        }
        Ok(())
    }

    /// Get the current navigation state
    pub fn navigation_state(&self) -> BrowserResult<NavigationState> {
        self.navigation.state()
    }

    /// Handle a page load error
    pub fn handle_load_error(&self, error: &str) -> BrowserResult<()> {
        self.navigation.handle_error(error)
    }

    /// Get the current URL
    pub fn current_url(&self) -> BrowserResult<Option<String>> {
        Ok(self.navigation.current_url()?.map(|url| url.display_url()))
    }

    /// Check if we can navigate back
    pub fn can_go_back(&self) -> BrowserResult<bool> {
        self.navigation.can_go_back()
    }

    /// Check if we can navigate forward
    pub fn can_go_forward(&self) -> BrowserResult<bool> {
        self.navigation.can_go_forward()
    }

    /// Reload the current page
    pub fn reload(&self) -> BrowserResult<()> {
        if let Some(parsed_url) = self.navigation.reload()? {
            self.webview.load_url(parsed_url.url().as_str());
        }
        Ok(())
    }

    /// Stop the current page load
    pub fn stop_loading(&self) -> BrowserResult<()> {
        self.webview.evaluate_script("window.stop();")?;
        self.navigation.stop_loading()
    }

    /// Get the current URL for splitting
    pub fn get_url_for_split(&self) -> BrowserResult<Option<String>> {
        Ok(self.navigation.get_current_for_split()?.map(|url| url.url().to_string()))
    }
}

/// Manages multiple browser tabs
pub struct TabManager {
    tabs: Arc<Mutex<Vec<Tab>>>,
    active_tab: Arc<Mutex<usize>>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(Mutex::new(Vec::new())),
            active_tab: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a new tab with the given WebView
    pub fn create_tab(&self, webview: WebView) -> BrowserResult<String> {
        let mut tabs = self.tabs.lock().map_err(|e| format!("Failed to lock tabs: {}", e))?;
        let id = format!("tab_{}", tabs.len());
        tabs.push(Tab::new(id.clone(), webview));
        Ok(id)
    }

    /// Get the active tab
    pub fn active_tab(&self) -> BrowserResult<Option<Tab>> {
        let tabs = self.tabs.lock().map_err(|e| format!("Failed to lock tabs: {}", e))?;
        let active_tab = self.active_tab.lock().map_err(|e| format!("Failed to lock active tab: {}", e))?;
        Ok(tabs.get(*active_tab).cloned())
    }

    /// Set the active tab by index
    pub fn set_active_tab(&self, index: usize) -> BrowserResult<()> {
        let tabs = self.tabs.lock().map_err(|e| format!("Failed to lock tabs: {}", e))?;
        if index < tabs.len() {
            let mut active_tab = self.active_tab.lock().map_err(|e| format!("Failed to lock active tab: {}", e))?;
            *active_tab = index;
            Ok(())
        } else {
            Err("Invalid tab index".into())
        }
    }

    /// Navigate the active tab to a URL
    pub fn navigate(&self, input: &str) -> BrowserResult<()> {
        if let Some(tab) = self.active_tab()? {
            tab.navigate(input)?;
        }
        Ok(())
    }

    /// Navigate the active tab back in history
    pub fn go_back(&self) -> BrowserResult<()> {
        if let Some(tab) = self.active_tab()? {
            tab.go_back()?;
        }
        Ok(())
    }

    /// Navigate the active tab forward in history
    pub fn go_forward(&self) -> BrowserResult<()> {
        if let Some(tab) = self.active_tab()? {
            tab.go_forward()?;
        }
        Ok(())
    }

    /// Get the current URL of the active tab
    pub fn current_url(&self) -> BrowserResult<Option<String>> {
        if let Some(tab) = self.active_tab()? {
            tab.current_url()
        } else {
            Ok(None)
        }
    }

    /// Check if the active tab can go back
    pub fn can_go_back(&self) -> BrowserResult<bool> {
        Ok(self.active_tab()?.map(|tab| tab.can_go_back().unwrap_or(false)).unwrap_or(false))
    }

    /// Check if the active tab can go forward
    pub fn can_go_forward(&self) -> BrowserResult<bool> {
        Ok(self.active_tab()?.map(|tab| tab.can_go_forward().unwrap_or(false)).unwrap_or(false))
    }

    /// Reload the active tab
    pub fn reload(&self) -> BrowserResult<()> {
        if let Some(tab) = self.active_tab()? {
            tab.reload()?;
        }
        Ok(())
    }

    /// Stop loading the active tab
    pub fn stop_loading(&self) -> BrowserResult<()> {
        if let Some(tab) = self.active_tab()? {
            tab.stop_loading()?;
        }
        Ok(())
    }

    /// Split the current tab
    pub fn split_tab(&self) -> BrowserResult<()> {
        // Get the current URL before creating new tab
        let current_url = if let Some(tab) = self.active_tab()? {
            tab.get_url_for_split()?
        } else {
            None
        };

        // Create new tab with the same URL
        let mut tabs = self.tabs.lock().map_err(|e| format!("Failed to lock tabs: {}", e))?;
        let id = format!("tab_{}", tabs.len());
        
        // Create WebView with the same properties as the original
        let webview = WebViewBuilder::new()
            .with_transparent(true)
            .with_visible(true)
            .build()
            .map_err(|e| format!("Failed to create WebView: {}", e))?;

        let new_tab = Tab::new(id.clone(), webview);
        
        // Navigate to the same URL if one exists
        if let Some(url) = current_url {
            new_tab.navigate(&url)?;
        }

        tabs.push(new_tab);
        Ok(())
    }
}

mod event_viewer;
mod tab_ui;
mod replay;
pub mod keyboard;

use self::{
    event_viewer::EventViewer,
    tab_ui::TabBar,
    replay::{EventRecorder, EventPlayer},
};

use crate::event::{BrowserEvent, EventSystem, BrowserCommand};

pub struct BrowserEngine {
    window: Window,
    tab_manager: Arc<TabManager>,
    event_viewer: Arc<EventViewer>,
    tab_bar: Arc<TabBar>,
}

impl BrowserEngine {
    pub fn new(window: Window) -> BrowserResult<Self> {
        let tab_manager = Arc::new(TabManager::new());
        let event_viewer = Arc::new(EventViewer::new());
        let tab_bar = Arc::new(TabBar::new());

        Ok(Self {
            window,
            tab_manager,
            event_viewer,
            tab_bar,
        })
    }

    /// Create a new tab with the given URL
    pub fn create_tab(&self, url: Option<&str>) -> BrowserResult<()> {
        let webview = WebViewBuilder::new(&self.window)
            .with_transparent(true)
            .with_visible(true)
            .build()?;

        let tab_id = self.tab_manager.create_tab(webview)?;
        self.tab_bar.add_tab(&tab_id)?;

        // Navigate to URL if provided
        if let Some(url) = url {
            self.tab_manager.navigate(url)?;
        }

        Ok(())
    }

    /// Handle navigation events from the UI
    pub fn handle_navigation_event(&self, event: &str) -> BrowserResult<()> {
        match event {
            "back" => self.tab_manager.go_back()?,
            "forward" => self.tab_manager.go_forward()?,
            "reload" => self.tab_manager.reload()?,
            "stop" => self.tab_manager.stop_loading()?,
            "split" => self.tab_manager.split_tab()?,
            url => self.tab_manager.navigate(url)?,
        }
        
        // Update UI state
        self.update_navigation_state()?;
        Ok(())
    }

    /// Update the UI with current navigation state
    fn update_navigation_state(&self) -> BrowserResult<()> {
        let can_go_back = self.tab_manager.can_go_back()?;
        let can_go_forward = self.tab_manager.can_go_forward()?;
        let current_url = self.tab_manager.current_url()?;

        // Send state to UI
        if let Some(url) = current_url {
            self.tab_bar.update_url(&url)?;
        }
        self.tab_bar.update_navigation_state(can_go_back, can_go_forward)?;

        Ok(())
    }

    /// Handle page load events
    pub fn handle_page_load(&self, success: bool, error: Option<&str>) -> BrowserResult<()> {
        if !success {
            if let Some(error) = error {
                if let Some(tab) = self.tab_manager.active_tab()? {
                    tab.handle_load_error(error)?;
                }
            }
        }
        self.update_navigation_state()?;
                Ok(())
    }
}
