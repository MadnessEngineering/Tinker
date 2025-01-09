//! Browser engine implementation

use std::{
    sync::{Arc, Mutex, mpsc::channel},
    time::{Duration, Instant},
};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    dpi::LogicalSize,
};
use wry::{WebView, WebViewBuilder, Error};
use tracing::{debug, info, error};

use crate::event::{BrowserEvent, EventSystem};

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

#[derive(Debug)]
enum BrowserCommand {
    Navigate { url: String },
    CreateTab { url: String },
    CloseTab { id: usize },
    SwitchTab { id: usize },
    RecordEvent { event: BrowserEvent },
    PlayEvent { event: BrowserEvent },
}

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
}

impl BrowserEngine {
    pub fn new(headless: bool, events: Option<Arc<Mutex<EventSystem>>>) -> Self {
        if let Some(ref events) = events {
            if let Ok(events) = events.lock() {
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
        }
    }

    fn publish_event(&self, event: BrowserEvent) -> Result<(), String> {
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

    pub fn stop_recording(&self) {
        if let Ok(mut recorder) = self.recorder.lock() {
            recorder.stop();
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

    pub fn start_replay(&self) {
        if let Ok(mut player) = self.player.lock() {
            player.start();
        }
    }

    pub fn set_replay_speed(&self, speed: f32) {
        if let Ok(mut player) = self.player.lock() {
            player.set_speed(speed);
        }
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Starting browser engine...");

        if self.headless {
            // In headless mode, we don't need to create a window or event loop
            // Just perform the necessary operations and return
            if let Ok(mut tabs) = self.tabs.lock() {
                if tabs.get_all_tabs().is_empty() {
                    let id = tabs.create_tab("about:blank".to_string());
                    info!("Created initial tab {} in headless mode", id);
                }
            }
            return Ok(());
        }

        let event_loop = EventLoop::new();

        // Create the main window
        let window_builder = WindowBuilder::new()
            .with_title("Tinker Browser")
            .with_inner_size(LogicalSize::new(1024.0, 768.0))
            .with_decorations(true);

        let window = window_builder.build(&event_loop)?;
        let window = Arc::new(window);
        self.window = Some(window.clone());

        // Create channels for browser commands and tab commands
        let (cmd_tx, cmd_rx) = channel::<BrowserCommand>();
        let (tab_tx, tab_rx) = channel::<TabCommand>();

        // Create the tab bar first
        let tab_bar = TabBar::new(&window, tab_tx.clone())?;
        self.tab_bar = Some(tab_bar);

        // Create initial tab if none exists
        if let Ok(mut tabs) = self.tabs.lock() {
            if tabs.get_all_tabs().is_empty() {
                let id = tabs.create_tab("about:blank".to_string());

                // Create the initial WebView with proper positioning
                let content_view = {
                    let events = self.events.clone();
                    let tab_height: u32 = 40; // Match the tab bar height

                    let view = WebViewBuilder::new(&window)
                        .with_bounds(wry::Rect {
                            x: 0_i32,
                            y: tab_height as i32,
                            width: window.inner_size().width,
                            height: window.inner_size().height.saturating_sub(tab_height),
                        })
                        .with_url("about:blank")
                        .map_err(|e| {
                            error!("Failed to set initial WebView URL: {}", e);
                            Box::new(e) as Box<dyn std::error::Error>
                        })?
                        .with_initialization_script(
                            r#"
                            window.addEventListener('DOMContentLoaded', () => { 
                                document.body.style.backgroundColor = '#ffffff';
                                document.body.style.marginTop = '40px';
                                window.addEventListener('load', () => {
                                    window.ipc.postMessage(JSON.stringify({
                                        type: 'page_loaded',
                                        url: window.location.href
                                    }));
                                });
                                const observer = new MutationObserver(() => {
                                    window.ipc.postMessage(JSON.stringify({
                                        type: 'title_changed',
                                        title: document.title
                                    }));
                                });
                                observer.observe(document.querySelector('title'), { 
                                    childList: true 
                                });
                            });
                            "#
                        )
                        .with_ipc_handler(move |msg: String| {
                            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&msg) {
                                match msg["type"].as_str() {
                                    Some("page_loaded") => {
                                        if let Some(url) = msg["url"].as_str() {
                                            debug!("Page loaded: {}", url);
                                            if let Some(events) = &events {
                                                if let Ok(mut events) = events.lock() {
                                                    let _ = events.publish(BrowserEvent::PageLoaded {
                                                        url: url.to_string()
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    Some("title_changed") => {
                                        if let Some(title) = msg["title"].as_str() {
                                            debug!("Title changed: {}", title);
                                            if let Some(events) = &events {
                                                if let Ok(mut events) = events.lock() {
                                                    let _ = events.publish(BrowserEvent::TitleChanged {
                                                        title: title.to_string()
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        })
                        .build()?;

                    Arc::new(Mutex::new(view))
                };

                self.content_view = Some(content_view.clone());
            }
        }

        // Add existing tabs to the UI
        if let Ok(mut tabs) = self.tabs.lock() {
            for tab in tabs.get_all_tabs() {
                if let Some(ref bar) = &self.tab_bar {
                    bar.add_tab(tab.id.try_into().unwrap(), &tab.title, &tab.url);
                }
            }
            if let Some(active_tab) = tabs.get_active_tab() {
                if let Some(ref bar) = &self.tab_bar {
                    bar.set_active_tab(active_tab.id.try_into().unwrap());
                }
            }
        }

        // Set up event handling
        let events = self.events.clone();
        let recorder = self.recorder.clone();
        let event_viewer = self.event_viewer.clone();
        let tabs = self.tabs.clone();
        let tab_bar = self.tab_bar.clone();
        let window = self.window.take();

        // Move content_view into the event loop
        let mut content_view = self.content_view.clone();

        // Set up the event loop
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::NewEvents(_) => {
                    *control_flow = ControlFlow::Wait;
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    // Clean up resources before exiting
                    content_view = None;
                    if let Some(window) = &window {
                        window.set_visible(false);
                    }
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    if let Some(window) = &window {
                        // Update tab bar bounds
                        if let Some(ref bar) = &tab_bar {
                            bar.update_bounds(window);
                        }

                        // Update content view bounds
                        if let Some(ref content_view) = content_view {
                            if let Ok(content_view) = content_view.lock() {
                                content_view.set_bounds(wry::Rect {
                                    x: 0_i32,
                                    y: 40_i32,
                                    width: new_size.width,
                                    height: new_size.height.saturating_sub(40),
                                });
                            }
                        }
                    }
                }
                Event::LoopDestroyed => {
                    // Clean up resources when the event loop is destroyed
                    content_view = None;
                    if let Some(window) = &window {
                        window.set_visible(false);
                    }
                }
                _ => (),
            }

            // Handle tab commands
            while let Ok(cmd) = tab_rx.try_recv() {
                match cmd {
                    TabCommand::Create { url } => {
                        if let Ok(mut tabs) = tabs.lock() {
                            let id = tabs.create_tab(url.clone());
                            if let Some(ref bar) = &tab_bar {
                                bar.add_tab(id.try_into().unwrap(), "New Tab", &url);
                                bar.set_active_tab(id.try_into().unwrap());
                            }
                            if let Some(events) = &events {
                                if let Ok(mut events) = events.lock() {
                                    let _ = events.publish(BrowserEvent::TabCreated { id });
                                }
                            }
                        }
                    }
                    TabCommand::Close { id } => {
                        if let Ok(mut tabs) = tabs.lock() {
                            if tabs.close_tab(id.try_into().unwrap()) {
                                if let Some(ref bar) = &tab_bar {
                                    bar.remove_tab(id.try_into().unwrap());

                                    // Get the new active tab
                                    if let Some(active_tab) = tabs.get_active_tab() {
                                        bar.set_active_tab(active_tab.id.try_into().unwrap());

                                        // Update content view
                                        if let Some(webview) = tabs.get_tab_webview(active_tab.id) {
                                            content_view = Some(webview.clone());

                                            // Ensure proper positioning
                                            if let Some(window) = &window {
                                                if let Ok(view) = webview.lock() {
                                                    view.set_bounds(wry::Rect {
                                                        x: 0_i32,
                                                        y: 40_i32, // Tab bar height
                                                        width: window.inner_size().width,
                                                        height: window.inner_size().height.saturating_sub(40),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }

                                // Ensure tab bar stays visible
                                if let Some(ref bar) = &tab_bar {
                                    if let Some(window) = &window {
                                        bar.update_bounds(window);
                                    }
                                }
                            }
                        }
                    }
                    TabCommand::Switch { id } => {
                        if let Ok(mut tabs) = tabs.lock() {
                            if tabs.switch_to_tab(id.try_into().unwrap()) {
                                if let Some(ref bar) = &tab_bar {
                                    bar.set_active_tab(id.try_into().unwrap());
                                }

                                // Switch to the tab's WebView
                                if let Some(webview) = tabs.get_tab_webview(id.try_into().unwrap()) {
                                    content_view = Some(webview.clone());
                                }

                                let _ = self.publish_event(BrowserEvent::TabSwitched { id: id.try_into().unwrap() });
                            }
                        }
                    }
                }
            }

            // Handle browser commands
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    BrowserCommand::Navigate { url } => {
                        if let Some(ref view) = content_view {
                            if let Ok(view) = view.lock() {
                                view.load_url(&url);
                            }
                        }
                    }
                    BrowserCommand::CreateTab { url } => {
                        if let Ok(mut tabs) = tabs.lock() {
                            let id = tabs.create_tab(url.clone());
                            info!("Created new tab {} with URL: {}", id, url);

                            // Update tab UI
                            if let Some(ref bar) = &tab_bar {
                                bar.add_tab(id.try_into().unwrap(), "New Tab", &url);
                                bar.set_active_tab(id.try_into().unwrap());
                            }

                            // Create a new WebView for the tab
                            if let Some(window) = &window {
                                let events = events.clone();
                                match WebViewBuilder::new(&**window)
                                    .with_url(&url)
                                    .map_err(|e| {
                                        error!("Failed to set WebView URL: {}", e);
                                        e
                                    })
                                    .map(|builder| {
                                        let events = events.clone();
                                        builder
                                            .with_initialization_script(
                                                r#"
                                                window.addEventListener('DOMContentLoaded', () => { 
                                                    document.body.style.backgroundColor = '#ffffff';
                                                    window.addEventListener('load', () => {
                                                        window.ipc.postMessage(JSON.stringify({
                                                            type: 'page_loaded',
                                                            url: window.location.href
                                                        }));
                                                    });
                                                    const observer = new MutationObserver(() => {
                                                        window.ipc.postMessage(JSON.stringify({
                                                            type: 'title_changed',
                                                            title: document.title
                                                        }));
                                                    });
                                                    observer.observe(document.querySelector('title'), { 
                                                        childList: true 
                                                    });
                                                });
                                                "#
                                            )
                                            .with_ipc_handler(move |msg: String| {
                                                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&msg) {
                                                    match msg["type"].as_str() {
                                                        Some("page_loaded") => {
                                                            if let Some(url) = msg["url"].as_str() {
                                                                debug!("Page loaded: {}", url);
                                                                if let Some(events) = &events {
                                                                    if let Ok(mut events) = events.lock() {
                                                                        let _ = events.publish(BrowserEvent::PageLoaded {
                                                                            url: url.to_string()
                                                                        });
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        Some("title_changed") => {
                                                            if let Some(title) = msg["title"].as_str() {
                                                                debug!("Title changed: {}", title);
                                                                if let Some(events) = &events {
                                                                    if let Ok(mut events) = events.lock() {
                                                                        let _ = events.publish(BrowserEvent::TitleChanged {
                                                                            title: title.to_string()
                                                                        });
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            })
                                            .build()
                                    }) {
                                    Ok(Ok(view)) => {
                                        let webview = Arc::new(Mutex::new(view));
                                        tabs.set_tab_webview(id, webview.clone());
                                        content_view = Some(webview);
                                    }
                                    _ => error!("Failed to create WebView for tab {}", id),
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        })
    }

    pub fn create_tab(&mut self, url: &str) -> Result<usize, String> {
        if let Ok(mut tabs) = self.tabs.lock() {
            let id = tabs.create_tab(url.to_string());
            self.publish_event(BrowserEvent::TabCreated { id })?;
            self.publish_event(BrowserEvent::TabUrlChanged {
                id,
                url: url.to_string(),
            })?;
            Ok(id)
        } else {
            Err("Failed to lock tabs".to_string())
        }
    }

    pub fn switch_to_tab(&mut self, id: usize) -> Result<(), String> {
        if let Ok(mut tabs) = self.tabs.lock() {
            if tabs.switch_to_tab(id) {
                self.publish_event(BrowserEvent::TabSwitched { id })?;
                Ok(())
            } else {
                Err("Tab not found".to_string())
            }
        } else {
            Err("Failed to lock tabs".to_string())
        }
    }

    pub fn close_tab(&mut self, id: usize) -> Result<(), String> {
        if let Ok(mut tabs) = self.tabs.lock() {
            if tabs.close_tab(id) {
                self.publish_event(BrowserEvent::TabClosed { id })?;
                Ok(())
            } else {
                Err("Tab not found".to_string())
            }
        } else {
            Err("Failed to lock tabs".to_string())
        }
    }
}
