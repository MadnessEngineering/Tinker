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
    headless: bool,
    events: Option<Arc<Mutex<EventSystem>>>,
    player: Arc<Mutex<EventPlayer>>,
    recorder: Arc<Mutex<EventRecorder>>,
    event_viewer: Arc<Mutex<EventViewer>>,
    tabs: Arc<Mutex<TabManager>>,
    tab_bar: Option<TabBar>,
    content_view: Option<Arc<Mutex<WebView>>>,
    window: Option<Arc<Window>>,
}

impl BrowserEngine {
    pub fn new(headless: bool, events: Option<Arc<Mutex<EventSystem>>>) -> Self {
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

    pub fn navigate(&self, url: &str) -> Result<(), String> {
        info!("Navigating to: {}", url);

        // Update the tab URL first
        if let Ok(mut tabs) = self.tabs.lock() {
            if let Some(tab) = tabs.get_active_tab_mut() {
                tab.url = url.to_string();
            }
        }

        // Then update the WebView
        if let Some(view) = &self.content_view {
            if let Ok(view) = view.lock() {
                view.load_url(url);
            }
        }

        // Finally, emit the navigation event
        if let Some(events) = &self.events {
            if let Ok(mut events) = events.lock() {
                events.publish(BrowserEvent::Navigation {
                    url: url.to_string(),
                }).map_err(|e| e.to_string())?;
            }
        }

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

        let event_loop = EventLoop::new();

        // Create the main window
        let window_builder = WindowBuilder::new()
            .with_title("Tinker Browser")
            .with_inner_size(LogicalSize::new(1024.0, 768.0));

        let window = window_builder.build(&event_loop)?;
        let window = Arc::new(window);
        self.window = Some(window.clone());

        // Create initial tab if none exists
        if let Ok(mut tabs) = self.tabs.lock() {
            if tabs.get_all_tabs().is_empty() {
                tabs.create_tab("about:blank".to_string());
            }
        }

        // Create the content view
        let content_view = WebViewBuilder::new(&window)
            .with_url("about:blank")
            .map_err(|e| {
                error!("Failed to set initial WebView URL: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })
            .and_then(|builder| builder.build()
                .map_err(|e| {
                    error!("Failed to build initial WebView: {}", e);
                    Box::new(e) as Box<dyn std::error::Error>
                }))?;
        self.content_view = Some(Arc::new(Mutex::new(content_view)));

        // Create channels for browser commands
        let (cmd_tx, cmd_rx) = channel::<BrowserCommand>();

        // Create the tab bar if not in headless mode
        if !self.headless {
            // Create the tab bar with the command sender
            let tab_cmd_tx = cmd_tx.clone();
            let tab_bar = TabBar::new(&window, move |cmd| {
                match cmd {
                    TabCommand::Create { url } => {
                        let _ = tab_cmd_tx.send(BrowserCommand::CreateTab { url });
                    }
                    TabCommand::Close { id } => {
                        let _ = tab_cmd_tx.send(BrowserCommand::CloseTab { id });
                    }
                    TabCommand::Switch { id } => {
                        let _ = tab_cmd_tx.send(BrowserCommand::SwitchTab { id });
                    }
                }
            })?;

            // Add existing tabs to the UI
            if let Ok(tabs) = self.tabs.lock() {
                for tab in tabs.get_all_tabs() {
                    tab_bar.add_tab(tab.id, &tab.title, &tab.url);
                }
                if let Some(active_tab) = tabs.get_active_tab() {
                    tab_bar.set_active_tab(active_tab.id);
                }
            }
            self.tab_bar = Some(tab_bar);
        }

        // Set up event handling
        let headless = self.headless;
        let events = self.events.clone();
        let recorder = self.recorder.clone();
        let event_viewer = self.event_viewer.clone();
        let tabs = self.tabs.clone();
        let tab_bar = self.tab_bar.clone();
        let window = self.window.take();

        // Move content_view into the event loop
        let content_view = self.content_view.clone();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = if headless {
                ControlFlow::Exit
            } else {
                ControlFlow::Wait
            };

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
                            if let Some(ref bar) = tab_bar {
                                bar.add_tab(id, "New Tab", &url);
                                bar.set_active_tab(id);
                            }

                            // Create a new WebView for the tab
                            if let Some(window) = &window {
                                if let Err(e) = WebViewBuilder::new(&**window)
                                    .with_url(&url)
                                    .map_err(|e| {
                                        error!("Failed to set WebView URL: {}", e);
                                        e
                                    })
                                    .and_then(|builder| builder.build().map_err(|e| {
                                        error!("Failed to build WebView: {}", e);
                                        e
                                    }))
                                    .map(|new_view| {
                                        if let Some(ref view) = content_view {
                                            if let Ok(mut view_lock) = view.lock() {
                                                *view_lock = new_view;
                                            }
                                        }
                                    }) {
                                    error!("Failed to create WebView: {}", e);
                                }
                            }

                            // Publish event
                            if let Some(events) = &events {
                                if let Ok(mut events) = events.lock() {
                                    let _ = events.publish(BrowserEvent::TabCreated { id });
                                    let _ = events.publish(BrowserEvent::TabUrlChanged { id, url: url.clone() });
                                }
                            }
                        }
                    }
                    BrowserCommand::CloseTab { id } => {
                        if let Ok(mut tabs) = tabs.lock() {
                            if tabs.close_tab(id) {
                                if let Some(ref bar) = tab_bar {
                                    bar.remove_tab(id);
                                    if let Some(active_tab) = tabs.get_active_tab() {
                                        bar.set_active_tab(active_tab.id);
                                        if let Some(ref view) = content_view {
                                            if let Ok(view) = view.lock() {
                                                view.load_url(&active_tab.url);
                                            }
                                        }
                                    }
                                }
                                if let Some(events) = &events {
                                    if let Ok(mut events) = events.lock() {
                                        let _ = events.publish(BrowserEvent::TabClosed { id });
                                    }
                                }
                            }
                        }
                    }
                    BrowserCommand::SwitchTab { id } => {
                        if let Ok(mut tabs) = tabs.lock() {
                            if tabs.switch_to_tab(id) {
                                if let Some(ref bar) = tab_bar {
                                    bar.set_active_tab(id);
                                }
                                if let Some(active_tab) = tabs.get_active_tab() {
                                    if let Some(ref view) = content_view {
                                        if let Ok(view) = view.lock() {
                                            view.load_url(&active_tab.url);
                                        }
                                    }
                                }
                                if let Some(events) = &events {
                                    if let Ok(mut events) = events.lock() {
                                        let _ = events.publish(BrowserEvent::TabSwitched { id });
                                    }
                                }
                            }
                        }
                    }
                    BrowserCommand::RecordEvent { event } => {
                        if let Ok(mut recorder) = recorder.lock() {
                            recorder.record_event(event.clone());
                        }
                        if let Ok(mut event_viewer) = event_viewer.lock() {
                            event_viewer.add_event(event);
                        }
                    }
                    BrowserCommand::PlayEvent { event } => {
                        match &event {
                            BrowserEvent::Navigation { url } => {
                                if let Some(ref view) = content_view {
                                    if let Ok(view) = view.lock() {
                                        view.load_url(url);
                                    }
                                }
                            }
                            BrowserEvent::TabCreated { .. } => {
                                if let Ok(mut tabs) = tabs.lock() {
                                    let id = tabs.create_tab("about:blank".to_string());
                                    if let Some(ref bar) = tab_bar {
                                        bar.add_tab(id, "New Tab", "about:blank");
                                        bar.set_active_tab(id);
                                    }
                                }
                            }
                            BrowserEvent::TabClosed { id } => {
                                if let Ok(mut tabs) = tabs.lock() {
                                    if tabs.close_tab(*id) {
                                        if let Some(ref bar) = tab_bar {
                                            bar.remove_tab(*id);
                                        }
                                    }
                                }
                            }
                            BrowserEvent::TabSwitched { id } => {
                                if let Ok(mut tabs) = tabs.lock() {
                                    if tabs.switch_to_tab(*id) {
                                        if let Some(ref bar) = tab_bar {
                                            bar.set_active_tab(*id);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            }
        });

        #[allow(unreachable_code)]
        Ok(())
    }

    pub fn create_tab(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut tabs) = self.tabs.lock() {
            let id = tabs.create_tab(url.to_string());
            
            // Update tab UI
            if let Some(tab_bar) = &self.tab_bar {
                tab_bar.add_tab(id, "New Tab", url);
                tab_bar.set_active_tab(id);
            }

            // Update content view
            if let Some(content_view) = &self.content_view {
                if let Ok(mut view) = content_view.lock() {
                    view.load_url(url);
                }
            }
            
            // Publish event
            if let Some(events) = &self.events {
                if let Ok(mut events) = events.lock() {
                    events.publish(BrowserEvent::TabCreated { id })?;
                    events.publish(BrowserEvent::TabUrlChanged { id, url: url.to_string() })?;
                }
            }

            Ok(())
        } else {
            Err("Failed to lock tabs".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_navigation() {
        let mut browser = BrowserEngine::new(false, None);

        // Create initial tab and get its URL
        let initial_url = {
            let mut tabs = browser.tabs.lock().unwrap();
            let id = tabs.create_tab("about:blank".to_string());
            tabs.get_active_tab().map(|tab| tab.url.clone()).unwrap()
        };
        assert_eq!(initial_url, "about:blank");

        // Navigate to the test URL
        browser.navigate("https://www.example.com").unwrap();

        // Verify the URL was updated
        let final_url = {
            let tabs = browser.tabs.lock().unwrap();
            tabs.get_active_tab().map(|tab| tab.url.clone()).unwrap()
        };
        assert_eq!(final_url, "https://www.example.com");
    }
}
