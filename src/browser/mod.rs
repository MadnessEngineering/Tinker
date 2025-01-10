//! Browser engine implementation

use std::{
    sync::{Arc, Mutex, mpsc::channel},
    time::{Duration, Instant},
};
use tao::{
    event::{Event, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    dpi::LogicalSize,
    keyboard::ModifiersState,
};
use wry::{WebView, WebViewBuilder};
use tracing::{debug, info, error};

mod tabs;
mod event_viewer;
mod tab_ui;
mod replay;
pub mod keyboard;

use self::{
    tabs::TabManager,
    event_viewer::EventViewer,
    tab_ui::{TabBar, TabCommand},
    replay::{EventRecorder, EventPlayer},
};

use crate::event::{BrowserEvent, EventSystem, BrowserCommand};

pub struct BrowserEngine {
    pub headless: bool,
    pub events: Option<Arc<Mutex<EventSystem>>>,
    pub player: Arc<Mutex<EventPlayer>>,
    pub recorder: Arc<Mutex<EventRecorder>>,
    pub event_viewer: Arc<Mutex<EventViewer>>,
    pub tabs: Arc<Mutex<TabManager>>,
    pub tab_bar: Option<TabBar>,
    pub content_view: Option<Arc<Mutex<WebView>>>,
    pub window: Option<Arc<Window>>,
    pub initial_url: Option<String>,
}

impl BrowserEngine {
    pub fn new(headless: bool, events: Option<Arc<Mutex<EventSystem>>>, initial_url: Option<String>) -> Self {
        if let Some(ref events) = events {
            if let Ok(_events) = events.lock() {
                info!("Browser engine initialized with event system");
            } else {
                error!("Failed to lock event system during initialization");
            }
        } else {
            debug!("Browser engine initialized without event system");
        }

        BrowserEngine {
            headless,
            events,
            player: Arc::new(Mutex::new(EventPlayer::default())),
            recorder: Arc::new(Mutex::new(EventRecorder::default())),
            event_viewer: Arc::new(Mutex::new(EventViewer::default())),
            tabs: Arc::new(Mutex::new(TabManager::default())),
            tab_bar: None,
            content_view: None,
            window: None,
            initial_url,
        }
    }

    fn publish_event(&self, event: BrowserEvent) -> Result<(), String> {
        // First, add to event viewer for monitoring
        if let Ok(mut viewer) = self.event_viewer.lock() {
            viewer.add_event(event.clone());
        }

        // Then publish to event system if available
        if let Some(events) = &self.events {
            if let Ok(mut events) = events.lock() {
                events.publish(event)
                    .map_err(|e| format!("Failed to publish event: {}", e))
            } else {
                Err("Failed to lock event system".to_string())
            }
        } else {
            // In headless mode or when events are disabled, just log
            debug!("Event published (no event system): {:?}", event);
            Ok(())
        }
    }

    pub fn navigate(&self, url: &str) -> Result<(), String> {
        info!("Navigating to: {}", url);

        // Update the tab URL first
        if let Ok(mut tabs) = self.tabs.lock() {
            if let Some(tab) = tabs.get_active_tab_mut() {
                tab.url = url.to_string();
                // Publish URL changed event
                self.publish_event(BrowserEvent::TabUrlChanged {
                    id: tab.id,
                    url: url.to_string(),
                })?;
            }
        }

        // Then update the WebView
        if let Some(view) = &self.content_view {
            if let Ok(view) = view.lock() {
                // Send navigation message to WebView
                let msg = serde_json::json!({
                    "type": "navigate",
                    "url": url
                });
                if let Err(e) = view.evaluate_script(&format!("window.ipc.handleMessage('{}')", msg.to_string())) {
                    error!("Failed to send navigation message to WebView: {}", e);
                }

                // Load the URL
                view.load_url(url);
            }
        }

        // Finally, emit the navigation event
        self.publish_event(BrowserEvent::Navigation {
            url: url.to_string(),
        })?;

        Ok(())
    }

    pub fn get_active_tab(&self) -> Option<String> {
        if let Ok(tabs) = self.tabs.lock() {
            tabs.get_active_tab().map(|tab| tab.url.clone())
        } else {
            None
        }
    }

    pub fn init_events(&mut self, broker_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut events = EventSystem::new(broker_url, "tinker-browser");
        events.connect()?;
        self.events = Some(Arc::new(Mutex::new(events)));
        Ok(())
    }

    pub fn start_recording(&mut self, path: &str) {
        if let Ok(mut recorder) = self.recorder.lock() {
            recorder.set_save_path(path.to_string());
            recorder.start();
            info!("Started recording to {}", path);
        }
    }

    pub fn stop_recording(&self) -> Result<(), String> {
        if let Ok(mut recorder) = self.recorder.lock() {
            recorder.stop();
            info!("Stopped recording");
            Ok(())
        } else {
            Err("Failed to lock recorder".to_string())
        }
    }

    pub fn save_recording(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(recorder) = self.recorder.lock() {
            recorder.save(path)?;
        }
        Ok(())
    }

    pub fn load_recording(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut player) = self.player.lock() {
            player.load(path)?;
        }
        Ok(())
    }

    pub fn start_replay(&mut self) -> Result<(), String> {
        if let Ok(mut player) = self.player.lock() {
            player.start();
            info!("Started replay");

            // Clone necessary handles for the replay thread
            let player = self.player.clone();
            let cmd_tx = if let Some(ref events) = self.events {
                if let Ok(events) = events.lock() {
                    events.get_command_sender()
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(cmd_tx) = cmd_tx {
                // Spawn replay thread
                std::thread::spawn(move || {
                    let mut last_check = Instant::now();
                    while let Ok(mut player) = player.lock() {
                        if let Some(event) = player.next_event() {
                            if let Err(e) = cmd_tx.send(BrowserCommand::PlayEvent { event }) {
                                error!("Failed to send replay event: {}", e);
                                break;
                            }
                        }

                        // Sleep a bit to prevent busy waiting
                        if last_check.elapsed() < Duration::from_millis(10) {
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        last_check = Instant::now();
                    }
                    info!("Replay completed");
                });

                Ok(())
            } else {
                Err("No command sender available".to_string())
            }
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn stop_replay(&self) -> Result<(), String> {
        if let Ok(mut player) = self.player.lock() {
            player.stop();
            info!("Stopped replay");
            Ok(())
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn set_replay_speed(&self, speed: f32) -> Result<(), String> {
        if let Ok(mut player) = self.player.lock() {
            player.set_speed(speed);
            info!("Set replay speed to {}", speed);
            Ok(())
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        debug!("Starting browser engine");
        
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Browser")
            .with_inner_size(LogicalSize::new(800, 600))
            .build(&event_loop)
            .map_err(|e| format!("Failed to create window: {}", e))?;

        debug!("Created main window");

        // Create command channel for tab bar
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();

        // Create content view
        self.create_content_view(&window)?;
        debug!("Content view created successfully");

        // Create tab bar if not in headless mode
        if !self.headless {
            debug!("Creating tab bar");
            self.tab_bar = Some(TabBar::new(&window, cmd_tx).map_err(|e| format!("Failed to create tab bar: {}", e))?);
            debug!("Tab bar created successfully");
        }

        // Store window reference
        let window = Arc::new(window);
        self.window = Some(window.clone());

        // Create initial tab if URL provided
        let initial_url = self.initial_url.clone();
        if let Some(url) = initial_url {
            debug!("Creating initial tab with URL: {}", url);
            self.create_tab(&url)?;
            debug!("Initial tab created successfully");
        } else {
            // Create a default blank tab
            self.create_tab("about:blank")?;
            debug!("Created default blank tab");
        }

        let browser = Arc::new(Mutex::new(self.clone()));
        
        debug!("Starting event loop");
        
        event_loop.run(move |event, _window_target, control_flow| {
            *control_flow = ControlFlow::Wait;

            // Handle tab commands
            if let Ok(cmd) = cmd_rx.try_recv() {
                if let Ok(mut browser) = browser.lock() {
                    match cmd {
                        TabCommand::Create { url } => {
                            if let Err(e) = browser.create_tab(&url) {
                                error!("Failed to create tab: {}", e);
                            }
                        },
                        TabCommand::Close { id } => {
                            if let Err(e) = browser.close_tab(id) {
                                error!("Failed to close tab: {}", e);
                            }
                        },
                        TabCommand::Switch { id } => {
                            if let Err(e) = browser.switch_to_tab(id) {
                                error!("Failed to switch tab: {}", e);
                            }
                        },
                        TabCommand::UpdateUrl { id, url } => {
                            if let Err(e) = browser.update_tab_content(id, &url) {
                                error!("Failed to update tab URL: {}", e);
                            }
                        },
                        TabCommand::UpdateTitle { id, title } => {
                            if let Ok(mut tabs) = browser.tabs.lock() {
                                if let Some(tab) = tabs.get_tab_mut(id) {
                                    tab.title = title;
                                }
                            }
                        },
                    }
                }
            }

            match event {
                Event::WindowEvent { event, .. } => {
                    if let Ok(mut browser) = browser.lock() {
                        if let Err(e) = browser.handle_window_event(&event, &window) {
                            error!("Error handling window event: {}", e);
                        }
                        
                        if let WindowEvent::CloseRequested = event {
                            debug!("Window close requested, exiting");
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                _ => (),
            }
        })
    }

    pub fn create_tab(&mut self, url: &str) -> Result<usize, String> {
        // Create the tab in the manager
        let id = if let Ok(mut tabs) = self.tabs.lock() {
            let id = tabs.create_tab(url.to_string());

            // Update the tab bar
            if let Some(ref tab_bar) = self.tab_bar {
                tab_bar.update_tab_url(id, url);
            }

            // Publish tab created event
            self.publish_event(BrowserEvent::TabCreated {
                id,
                url: url.to_string()
            })?;

            // If this is the first tab, make it active
            if tabs.get_all_tabs().len() == 1 {
                tabs.switch_to_tab(id);
                self.update_tab_visibility()?;
                self.publish_event(BrowserEvent::TabActivated { id })?;
            }

            Ok(id)
        } else {
            Err("Failed to lock tab manager".to_string())
        }?;

        // Load the URL if this is the active tab
        if let Ok(tabs) = self.tabs.lock() {
            if let Some(_tab) = tabs.get_tab(id) {
                if tabs.is_active_tab(id) {
                    // Handle active tab
                }
            }
        }

        Ok(id)
    }

    pub fn switch_to_tab(&mut self, id: usize) -> Result<(), String> {
        // First switch the tab in the manager
        if let Ok(mut tabs) = self.tabs.lock() {
            if tabs.switch_to_tab(id) {
                // Update WebView content and tab bar
                self.update_tab_visibility()?;

                // Publish tab activated event
                self.publish_event(BrowserEvent::TabActivated { id })?;
                Ok(())
            } else {
                Err(format!("Failed to switch to tab {}", id))
            }
        } else {
            Err("Failed to lock tabs".to_string())
        }
    }

    pub fn close_tab(&mut self, id: usize) -> Result<(), String> {
        // Get the next active tab ID before closing
        let next_active_id = {
            let tabs = self.tabs.lock().map_err(|_| "Failed to lock tabs".to_string())?;
            if tabs.is_active_tab(id) {
                tabs.get_all_tabs().iter()
                    .find(|t| t.id != id)
                    .map(|t| t.id)
            } else {
                None
            }
        };

        // Close the tab
        {
            let mut tabs = self.tabs.lock().map_err(|_| "Failed to lock tabs".to_string())?;
            if !tabs.close_tab(id) {
                return Err("Tab not found".to_string());
            }
            // Publish tab closed event
            self.publish_event(BrowserEvent::TabClosed { id })?;
        }

        // Switch to another tab if needed
        if let Some(next_id) = next_active_id {
            self.switch_to_tab(next_id)?;
        } else {
            // Create a new blank tab if this was the last one
            let tabs = self.tabs.lock().map_err(|_| "Failed to lock tabs".to_string())?;
            if tabs.get_all_tabs().is_empty() {
                drop(tabs); // Release the lock before creating a new tab
                self.create_tab("about:blank")?;
            }
        }

        Ok(())
    }

    pub fn get_recent_events(&self, count: usize) -> Vec<String> {
        if let Ok(viewer) = self.event_viewer.lock() {
            viewer.get_recent_events(count)
                .iter()
                .map(|entry| format!("[{}] {:?}", entry.timestamp.format("%H:%M:%S"), entry.event))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn clear_event_history(&self) {
        if let Ok(mut viewer) = self.event_viewer.lock() {
            viewer.clear();
        }
    }

    fn handle_command(&mut self, command: BrowserCommand) -> Result<(), String> {
        match command {
            BrowserCommand::Navigate { url } => {
                self.navigate(&url)
            },
            BrowserCommand::CreateTab { url } => {
                if let Ok(mut tabs) = self.tabs.lock() {
                    let id = tabs.create_tab(url.clone());
                    self.publish_event(BrowserEvent::TabCreated {
                        id,
                        url: url.clone()
                    })?;
                    Ok(())
                } else {
                    Err("Failed to lock tab manager".to_string())
                }
            },
            BrowserCommand::CloseTab { id } => {
                if let Ok(mut tabs) = self.tabs.lock() {
                    if tabs.close_tab(id) {
                        self.publish_event(BrowserEvent::TabClosed { id })?;
                        Ok(())
                    } else {
                        Err(format!("Failed to close tab {}", id))
                    }
                } else {
                    Err("Failed to lock tab manager".to_string())
                }
            },
            BrowserCommand::SwitchTab { id } => {
                if let Ok(mut tabs) = self.tabs.lock() {
                    if tabs.switch_to_tab(id) {
                        self.publish_event(BrowserEvent::TabActivated { id })?;
                        Ok(())
                    } else {
                        Err(format!("Failed to switch to tab {}", id))
                    }
                } else {
                    Err("Failed to lock tab manager".to_string())
                }
            },
            BrowserCommand::RecordEvent { event } => {
                if let Ok(mut recorder) = self.recorder.lock() {
                    recorder.record_event(event);
                    Ok(())
                } else {
                    Err("Failed to lock event recorder".to_string())
                }
            },
            BrowserCommand::PlayEvent { event } => {
                // Handle the event based on its type
                match event {
                    BrowserEvent::Navigation { url } => self.navigate(&url),
                    BrowserEvent::TabCreated { url, .. } => {
                        if let Ok(mut tabs) = self.tabs.lock() {
                            tabs.create_tab(url);
                            Ok(())
                        } else {
                            Err("Failed to lock tab manager".to_string())
                        }
                    },
                    _ => {
                        // Publish other events directly
                        self.publish_event(event)
                    }
                }
            }
        }
    }

    fn handle_ipc_message(&self, msg: &str) -> Result<(), String> {
        let data: serde_json::Value = serde_json::from_str(msg)
            .map_err(|e| format!("Failed to parse IPC message: {}", e))?;

        match data["type"].as_str() {
            Some("pageLoaded") => {
                if let Some(url) = data["url"].as_str() {
                    self.publish_event(BrowserEvent::PageLoaded {
                        url: url.to_string(),
                    })?;
                }
            }
            Some("titleChanged") => {
                if let Some(title) = data["title"].as_str() {
                    // Update tab title
                    if let Ok(mut tabs) = self.tabs.lock() {
                        if let Some(tab) = tabs.get_active_tab_mut() {
                            tab.title = title.to_string();
                            self.publish_event(BrowserEvent::TabTitleChanged {
                                id: tab.id,
                                title: title.to_string(),
                            })?;
                        }
                    }

                    // Also publish general title changed event
                    self.publish_event(BrowserEvent::TitleChanged {
                        title: title.to_string(),
                    })?;
                }
            }
            Some("navigation") => {
                if let Some(url) = data["url"].as_str() {
                    // Update tab URL
                    if let Ok(mut tabs) = self.tabs.lock() {
                        if let Some(tab) = tabs.get_active_tab_mut() {
                            tab.url = url.to_string();
                            self.publish_event(BrowserEvent::TabUrlChanged {
                                id: tab.id,
                                url: url.to_string(),
                            })?;
                        }
                    }

                    // Also publish navigation event
                    self.publish_event(BrowserEvent::Navigation {
                        url: url.to_string(),
                    })?;
                }
            }
            Some(type_) => {
                error!("Unknown IPC message type: {}", type_);
            }
            None => {
                error!("IPC message missing type field");
            }
        }

        Ok(())
    }

    fn update_tab_content(&self, id: usize, url: &str) -> Result<(), String> {
        // First update the tab URL
        if let Ok(mut tabs) = self.tabs.lock() {
            if let Some(tab) = tabs.get_tab_mut(id) {
                tab.url = url.to_string();

                // If this is the active tab, update the WebView
                if tabs.is_active_tab(id) {
                    if let Some(view) = &self.content_view {
                        if let Ok(view) = view.lock() {
                            view.load_url(url);
                        }
                    }
                }

                // Publish URL changed event
                self.publish_event(BrowserEvent::TabUrlChanged {
                    id,
                    url: url.to_string(),
                })?;

                Ok(())
            } else {
                Err(format!("Tab {} not found", id))
            }
        } else {
            Err("Failed to lock tab manager".to_string())
        }
    }

    fn update_tab_visibility(&self) -> Result<(), String> {
        if let Ok(tabs) = self.tabs.lock() {
            if let Some(active_tab) = tabs.get_active_tab() {
                // Update WebView content
                if let Some(view) = &self.content_view {
                    if let Ok(view) = view.lock() {
                        view.load_url(&active_tab.url);
                    }
                }

                // Update tab bar
                if let Some(ref tab_bar) = self.tab_bar {
                    tab_bar.update_tab_url(active_tab.id, &active_tab.url);
                    tab_bar.update_tab_title(active_tab.id, &active_tab.title);
                }
            }
            Ok(())
        } else {
            Err("Failed to lock tab manager".to_string())
        }
    }

    fn create_content_view(&mut self, window: &Window) -> Result<(), String> {
        let tab_height: u32 = 40; // Match the tab bar height

        let webview = WebViewBuilder::new(window)
            .with_bounds(wry::Rect {
                x: 0_i32,
                y: tab_height as i32,
                width: window.inner_size().width,
                height: window.inner_size().height.saturating_sub(tab_height),
            })
            .with_initialization_script(include_str!("../templates/window_chrome.js"))
            .with_html(include_str!("../templates/window_chrome.html"))
            .map_err(|e| e.to_string())?
            .build()
            .map_err(|e| e.to_string())?;

        self.content_view = Some(Arc::new(Mutex::new(webview)));
        Ok(())
    }

    fn update_webview_bounds(&self, window: &Window) {
        let tab_height: u32 = 40;

        // Update tab bar bounds
        if let Some(ref tab_bar) = self.tab_bar {
            tab_bar.update_bounds(window);
        }

        // Update content view bounds
        if let Some(ref content_view) = self.content_view {
            if let Ok(view) = content_view.lock() {
                view.set_bounds(wry::Rect {
                    x: 0_i32,
                    y: tab_height as i32,
                    width: window.inner_size().width,
                    height: window.inner_size().height.saturating_sub(tab_height),
                });
            }
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) -> Result<(), String> {
        match event {
            WindowEvent::Resized(size) => {
                debug!("Handling window resize: {:?}", size);
                self.update_webview_bounds(window);
                Ok(())
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use tao::keyboard::Key;
                
                let is_pressed = matches!(event.state, ElementState::Pressed);
                
                if is_pressed && !event.repeat {
                    match &event.logical_key {
                        Key::Character(c) if *c == String::from("t") => {
                            debug!("Ctrl+T pressed, creating new tab");
                            self.create_tab("about:blank")
                                .map(|_| ())
                        }
                        Key::Character(c) if *c == String::from("w") => {
                            debug!("Ctrl+W pressed, closing current tab");
                            // Get the active tab ID before locking for close
                            let active_tab_id = {
                                let tabs = self.tabs.lock()
                                    .map_err(|_| "Failed to lock tabs".to_string())?;
                                tabs.get_active_tab()
                                    .map(|tab| tab.id)
                                    .ok_or_else(|| "No active tab".to_string())?
                            };
                            self.close_tab(active_tab_id)
                        }
                        Key::Tab => {
                            debug!("Ctrl+Tab pressed, switching tab");
                            // Get the next tab ID before locking for switch
                            let next_tab_id = {
                                let tabs = self.tabs.lock()
                                    .map_err(|_| "Failed to lock tabs".to_string())?;
                                let active_tab = tabs.get_active_tab()
                                    .ok_or_else(|| "No active tab".to_string())?;
                                let current_id = active_tab.id;
                                let all_tabs = tabs.get_all_tabs();
                                all_tabs.iter()
                                    .find(|t| t.id > current_id)
                                    .or_else(|| all_tabs.first())
                                    .map(|tab| tab.id)
                                    .ok_or_else(|| "No tab to switch to".to_string())?
                            };
                            self.switch_to_tab(next_tab_id)
                        }
                        _ => Ok(())
                    }
                } else {
                    Ok(())
                }
            }
            WindowEvent::CloseRequested => {
                debug!("Window close requested");
                Ok(())
            }
            _ => Ok(())
        }
    }
}

impl Clone for BrowserEngine {
    fn clone(&self) -> Self {
        BrowserEngine {
            headless: self.headless,
            events: self.events.clone(),
            player: self.player.clone(),
            recorder: self.recorder.clone(),
            event_viewer: self.event_viewer.clone(),
            tabs: self.tabs.clone(),
            tab_bar: self.tab_bar.clone(),
            content_view: self.content_view.clone(),
            window: self.window.clone(),
            initial_url: self.initial_url.clone(),
        }
    }
}
