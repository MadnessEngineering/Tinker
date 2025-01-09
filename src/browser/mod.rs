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
pub enum BrowserCommand {
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
                self.publish_event(BrowserEvent::TabCreated { 
                    id,
                    url: "about:blank".to_string()
                })?;
                info!("Created initial tab {}", id);
            }
        }

        // Store command sender for event system
        let cmd_tx_clone = cmd_tx.clone();
        if let Some(events) = &self.events {
            if let Ok(mut events) = events.lock() {
                events.set_command_sender(cmd_tx_clone);
            }
        }

        let mut browser = self;
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            // Process any pending commands
            if let Ok(cmd) = cmd_rx.try_recv() {
                if let Err(e) = browser.handle_command(cmd) {
                    error!("Failed to handle command: {}", e);
                }
            }

            // Process any pending tab commands
            if let Ok(cmd) = tab_rx.try_recv() {
                if let Some(ref tab_bar) = browser.tab_bar {
                    match cmd {
                        TabCommand::UpdateUrl { id, url } => {
                            tab_bar.update_tab_url(id, &url);
                        },
                        TabCommand::UpdateTitle { id, title } => {
                            tab_bar.update_tab_title(id, &title);
                        }
                    }
                }
            }

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit
                },
                Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                    if let Some(content_view) = &browser.content_view {
                        if let Ok(view) = content_view.lock() {
                            let tab_height: u32 = 40;
                            view.set_bounds(wry::Rect {
                                x: 0,
                                y: tab_height as i32,
                                width: size.width,
                                height: size.height.saturating_sub(tab_height),
                            });
                        }
                    }
                },
                _ => (),
            }
        });
    }

    pub fn create_tab(&mut self, url: &str) -> Result<usize, String> {
        if let Ok(mut tabs) = self.tabs.lock() {
            let id = tabs.create_tab(url.to_string());
            self.publish_event(BrowserEvent::TabCreated { 
                id,
                url: url.to_string() 
            })?;
            Ok(id)
        } else {
            Err("Failed to lock tab manager".to_string())
        }
    }

    pub fn switch_to_tab(&mut self, id: usize) -> Result<(), String> {
        if let Ok(mut tabs) = self.tabs.lock() {
            if tabs.switch_to_tab(id) {
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
}
