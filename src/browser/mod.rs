//! Browser engine implementation

use std::{
    collections::HashMap,
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

mod tabs;
mod event_viewer;
mod tab_ui;
mod replay;
mod visual;
mod inspector;
mod network;
mod performance;
mod console;
pub mod keyboard;
pub mod session;

use self::{
    tabs::TabManager,
    event_viewer::EventViewer,
    tab_ui::{TabBar, TabCommand},
    replay::{EventRecorder, EventPlayer, PlaybackState},
    visual::{VisualTester, ScreenshotOptions, ScreenshotResult},
    inspector::{DOMInspector, ElementSelector, InteractionType, ElementInfo, WaitCondition},
    network::{NetworkMonitor, NetworkRequest, NetworkResponse, NetworkFilter},
    performance::{PerformanceMonitor, CoreWebVitals, NavigationTiming, ResourceTiming, MemoryMetrics, JavaScriptProfile},
    console::{ConsoleMonitor, ConsoleLevel, ConsoleMessage, JavaScriptError},
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
    pub running: bool,
    pub visual_tester: Arc<Mutex<VisualTester>>,
    pub dom_inspector: Arc<Mutex<DOMInspector>>,
    pub network_monitor: Arc<Mutex<NetworkMonitor>>,
    pub performance_monitor: Arc<Mutex<PerformanceMonitor>>,
    pub console_monitor: Arc<Mutex<ConsoleMonitor>>,
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
            running: true,
            visual_tester: Arc::new(Mutex::new(VisualTester::new("screenshots".to_string()))),
            dom_inspector: Arc::new(Mutex::new(DOMInspector::new())),
            network_monitor: Arc::new(Mutex::new(NetworkMonitor::new())),
            performance_monitor: Arc::new(Mutex::new(PerformanceMonitor::new())),
            console_monitor: Arc::new(Mutex::new(ConsoleMonitor::new())),
        }
    }

    fn publish_event(&self, event: BrowserEvent) -> Result<(), String> {
        // Feed the recorder if recording is active
        if let Ok(mut recorder) = self.recorder.lock() {
            recorder.record_event(event.clone());
        }

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

        // Push URL into active tab's history and update URL
        if let Ok(mut tabs) = self.tabs.lock() {
            let active_id = tabs.get_active_tab().map(|t| t.id);
            if let Some(id) = active_id {
                tabs.push_history(id, url.to_string());
                self.publish_event(BrowserEvent::TabUrlChanged {
                    id,
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

    pub fn go_back(&self) -> Result<Option<String>, String> {
        let url = if let Ok(mut tabs) = self.tabs.lock() {
            let active_id = tabs.get_active_tab().map(|t| t.id);
            active_id.and_then(|id| tabs.go_back(id))
        } else {
            return Err("Failed to lock tabs".to_string());
        };
        if let Some(ref url) = url {
            if let Some(view) = &self.content_view {
                if let Ok(view) = view.lock() {
                    view.load_url(url);
                }
            }
            self.publish_event(BrowserEvent::Navigation { url: url.clone() }).ok();
            info!("Navigating back to: {}", url);
        }
        Ok(url)
    }

    pub fn go_forward(&self) -> Result<Option<String>, String> {
        let url = if let Ok(mut tabs) = self.tabs.lock() {
            let active_id = tabs.get_active_tab().map(|t| t.id);
            active_id.and_then(|id| tabs.go_forward(id))
        } else {
            return Err("Failed to lock tabs".to_string());
        };
        if let Some(ref url) = url {
            if let Some(view) = &self.content_view {
                if let Ok(view) = view.lock() {
                    view.load_url(url);
                }
            }
            self.publish_event(BrowserEvent::Navigation { url: url.clone() }).ok();
            info!("Navigating forward to: {}", url);
        }
        Ok(url)
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

    pub fn start_recording(&mut self, name: String, start_url: String) -> Result<(), String> {
        if let Ok(mut recorder) = self.recorder.lock() {
            recorder.start(name.clone(), start_url.clone());
            info!("Started recording '{}' from URL: {}", name, start_url);
        } else {
            return Err("Failed to lock recorder".to_string());
        }
        // Publish AFTER releasing recorder lock (publish_event also locks recorder)
        self.publish_event(BrowserEvent::RecordingStarted { name, start_url }).ok();
        Ok(())
    }

    pub fn stop_recording(&self) -> Result<(), String> {
        let result = if let Ok(mut recorder) = self.recorder.lock() {
            if let Some(recording) = recorder.stop() {
                info!("Stopped recording: {} ({} events)",
                    recording.metadata.name,
                    recording.metadata.event_count);
                Ok(Some((
                    recording.metadata.name.clone(),
                    recording.metadata.event_count,
                    recording.metadata.duration_ms,
                )))
            } else {
                Err("No active recording".to_string())
            }
        } else {
            Err("Failed to lock recorder".to_string())
        };
        // Publish AFTER releasing recorder lock
        if let Ok(Some((name, event_count, duration_ms))) = result {
            self.publish_event(BrowserEvent::RecordingStopped { name, event_count, duration_ms }).ok();
            Ok(())
        } else {
            result.map(|_| ())
        }
    }

    pub fn pause_recording(&self) -> Result<(), String> {
        if let Ok(mut recorder) = self.recorder.lock() {
            recorder.pause();
            info!("Paused recording");
        } else {
            return Err("Failed to lock recorder".to_string());
        }
        self.publish_event(BrowserEvent::RecordingPaused).ok();
        Ok(())
    }

    pub fn resume_recording(&self) -> Result<(), String> {
        if let Ok(mut recorder) = self.recorder.lock() {
            recorder.resume();
            info!("Resumed recording");
        } else {
            return Err("Failed to lock recorder".to_string());
        }
        self.publish_event(BrowserEvent::RecordingResumed).ok();
        Ok(())
    }

    pub fn save_recording(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(recorder) = self.recorder.lock() {
            recorder.save(path)?;
            info!("Saved recording to: {}", path);
        }
        // Publish AFTER releasing recorder lock
        self.publish_event(BrowserEvent::RecordingSaved { path: path.to_string() }).ok();
        Ok(())
    }

    pub fn add_recording_assertion(&self, expected_state: serde_json::Value, description: Option<String>) -> Result<(), String> {
        if let Ok(mut recorder) = self.recorder.lock() {
            recorder.add_assertion(expected_state, description);
            info!("Added assertion to recording");
            Ok(())
        } else {
            Err("Failed to lock recorder".to_string())
        }
    }

    pub fn enable_recording_snapshots(&self, interval_ms: u64) -> Result<(), String> {
        if let Ok(mut recorder) = self.recorder.lock() {
            recorder.enable_snapshots(interval_ms);
            info!("Enabled snapshots every {}ms", interval_ms);
            Ok(())
        } else {
            Err("Failed to lock recorder".to_string())
        }
    }

    pub fn load_recording(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut player) = self.player.lock() {
            player.load(path)?;
            info!("Loaded recording from: {}", path);

            // Publish event
            self.publish_event(BrowserEvent::RecordingLoaded {
                path: path.to_string()
            }).ok();
        }
        Ok(())
    }

    pub fn start_replay(&mut self) -> Result<(), String> {
        if let Ok(mut player) = self.player.lock() {
            player.start();
            info!("Started replay");

            // Publish event
            self.publish_event(BrowserEvent::PlaybackStarted).ok();

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

            // In GUI mode: spawn a thread that forwards events via the command channel.
            // In headless mode (no cmd_tx): run_headless() polls the player directly.
            if let Some(cmd_tx) = cmd_tx {
                std::thread::spawn(move || {
                    loop {
                        let (should_break, next_event) = match player.lock() {
                            Ok(mut p) => {
                                let state = p.get_state();
                                if state == PlaybackState::Completed || state == PlaybackState::Stopped {
                                    (true, None)
                                } else {
                                    (false, p.next_event())
                                }
                            }
                            Err(_) => (true, None),
                        };

                        if should_break {
                            break;
                        }

                        if let Some(event) = next_event {
                            if let Err(e) = cmd_tx.send(BrowserCommand::PlayEvent { event }) {
                                error!("Failed to send replay event: {}", e);
                                break;
                            }
                        } else {
                            std::thread::sleep(Duration::from_millis(1));
                        }
                    }
                    info!("Replay completed");
                });
            }

            Ok(())
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn stop_replay(&self) -> Result<(), String> {
        if let Ok(mut player) = self.player.lock() {
            player.stop();
            info!("Stopped replay");

            self.publish_event(BrowserEvent::PlaybackStopped).ok();
            Ok(())
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn pause_replay(&self) -> Result<(), String> {
        if let Ok(mut player) = self.player.lock() {
            player.pause();
            info!("Paused replay");

            self.publish_event(BrowserEvent::PlaybackPaused).ok();
            Ok(())
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn resume_replay(&self) -> Result<(), String> {
        if let Ok(mut player) = self.player.lock() {
            player.resume();
            info!("Resumed replay");

            self.publish_event(BrowserEvent::PlaybackResumed).ok();
            Ok(())
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn seek_replay(&self, timestamp_ms: u64) -> Result<(), String> {
        if let Ok(mut player) = self.player.lock() {
            player.seek(timestamp_ms);
            info!("Seeked to {}ms", timestamp_ms);

            self.publish_event(BrowserEvent::PlaybackSeeked { timestamp_ms }).ok();
            Ok(())
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn step_forward_replay(&self) -> Result<Option<BrowserEvent>, String> {
        if let Ok(mut player) = self.player.lock() {
            if let Some(event) = player.step_forward() {
                info!("Stepped forward to next event");
                Ok(Some(event))
            } else {
                Ok(None)
            }
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn step_backward_replay(&self) -> Result<(), String> {
        if let Ok(mut player) = self.player.lock() {
            player.step_backward();
            info!("Stepped backward to previous event");
            Ok(())
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn set_replay_speed(&self, speed: f32) -> Result<(), String> {
        if let Ok(mut player) = self.player.lock() {
            player.set_speed(speed);
            info!("Set replay speed to {}x", speed);

            self.publish_event(BrowserEvent::PlaybackSpeedChanged { speed }).ok();
            Ok(())
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn set_replay_loop(&self, enable: bool) -> Result<(), String> {
        if let Ok(mut player) = self.player.lock() {
            player.set_loop(enable);
            info!("Set replay loop: {}", enable);
            Ok(())
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    /// Execute a single BrowserEvent against the browser (used during replay).
    fn execute_event(&mut self, event: BrowserEvent) -> Result<(), String> {
        match event {
            BrowserEvent::Navigation { url } => {
                debug!("Replay: navigate to {}", url);
                self.navigate(&url)
            }
            BrowserEvent::TabCreated { url, .. } => {
                debug!("Replay: create tab {}", url);
                self.create_tab(&url).map(|_| ()).map_err(|e| e.to_string())
            }
            BrowserEvent::TabClosed { id } => {
                debug!("Replay: close tab {}", id);
                self.close_tab(id).map_err(|e| e.to_string())
            }
            BrowserEvent::TabActivated { id } => {
                debug!("Replay: activate tab {}", id);
                self.switch_to_tab(id).map_err(|e| e.to_string())
            }
            _ => {
                debug!("Replay: skipping informational event {:?}", event);
                Ok(())
            }
        }
    }

    pub fn get_replay_state(&self) -> Result<serde_json::Value, String> {
        if let Ok(player) = self.player.lock() {
            let state = player.get_state();
            let position = player.get_position();
            let duration = player.get_duration();

            Ok(serde_json::json!({
                "state": format!("{:?}", state),
                "position_ms": position,
                "duration_ms": duration,
                "progress": if duration > 0 {
                    (position as f64 / duration as f64) * 100.0
                } else {
                    0.0
                }
            }))
        } else {
            Err("Failed to lock player".to_string())
        }
    }

    pub fn run(&mut self) -> Result<(), WebViewError> {
        debug!("Starting browser engine");

        if self.headless {
            return self.run_headless();
        }

        let event_loop = EventLoop::new();
        debug!("Created event loop: {:?}", event_loop);

        debug!("Building window with WindowBuilder");
        let window = WindowBuilder::new()
            .with_title("Browser")
            .with_inner_size(LogicalSize::new(800, 600))
            .with_visible(true)
            .with_resizable(true)
            .with_decorations(true)
            .with_transparent(false)
            .with_maximized(false)
            .build(&event_loop)
            .map_err(|e| {
                error!("Failed to create window: {}", e);
                WebViewError::WindowError(e.to_string())
            })?;

        debug!("Window properties - size: {:?}, position: {:?}, visible: {}",
            window.inner_size(),
            window.outer_position().unwrap_or_default(),
            window.is_visible()
        );

        // Create command channel for tab bar
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<TabCommand>();
        debug!("Created command channel for tab bar");

        // Create content view before tab bar
        debug!("Creating content view");
        let webview = self.create_content_view(&window)?;
        self.content_view = Some(Arc::new(Mutex::new(webview)));
        debug!("Content view created and stored in Arc<Mutex>");

        // Create tab bar if not in headless mode
        if !self.headless {
            debug!("Creating tab bar");
            match TabBar::new(&window, cmd_tx) {
                Ok(tab_bar) => {
                    self.tab_bar = Some(tab_bar);
                    debug!("Tab bar created successfully");
                }
                Err(e) => {
                    error!("Failed to create tab bar: {}", e);
                    return Err(WebViewError::TabBarError(e.to_string()));
                }
            }
        }

        // Store window reference
        let window = Arc::new(window);
        self.window = Some(window.clone());
        debug!("Window reference stored in Arc");

        // Create initial tab if URL provided
        if let Some(url) = self.initial_url.clone() {
            debug!("Creating initial tab with URL: {}", url);
            self.create_tab(&url)?;
        } else {
            debug!("Creating default blank tab");
            self.create_tab("about:blank")?;
        }

        let browser = Arc::new(Mutex::new(self.clone()));
        debug!("Created browser Arc<Mutex>");

        // Force window to front and request focus
        window.set_visible(true);
        window.set_focus();
        debug!("Window set to visible and focused");

        window.request_redraw();
        debug!("Initial redraw requested");

        event_loop.run(move |event, _window_target, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event, .. } => {
                    debug!("Window event received: {:?}", event);
                    if let Ok(mut browser) = browser.lock() {
                        if let Err(e) = browser.handle_window_event(&event, &window) {
                            error!("Error handling window event: {}", e);
                        }

                        if let WindowEvent::CloseRequested = event {
                            debug!("Window close requested, exiting");
                            *control_flow = ControlFlow::Exit;
                        }
                    } else {
                        error!("Failed to lock browser in event loop");
                    }
                }
                Event::MainEventsCleared => {
                    debug!("Main events cleared");
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    debug!("Redraw requested");
                    if let Ok(browser) = browser.lock() {
                        if let Some(window) = &browser.window {
                            debug!("Window state - size: {:?}, visible: {}, position: {:?}",
                                window.inner_size(),
                                window.is_visible(),
                                window.outer_position().unwrap_or_default()
                            );
                            // Ensure window is visible and focused
                            if !window.is_visible() {
                                window.set_visible(true);
                                window.set_focus();
                                debug!("Forced window visibility and focus");
                            }
                        }
                    }
                }
                Event::NewEvents(start_cause) => {
                    debug!("New events started: {:?}", start_cause);
                }
                Event::Resumed => {
                    debug!("Window resumed");
                    window.set_visible(true);
                    window.set_focus();
                    window.request_redraw();
                }
                _ => {
                    debug!("Other event: {:?}", event);
                },
            }
        });
    }

    fn run_headless(&mut self) -> Result<(), WebViewError> {
        info!("Running in headless mode");

        // Create initial tab only if none have been pre-created (e.g. via --tabs)
        let has_tabs = self.tabs.lock()
            .map(|t| !t.get_all_tabs().is_empty())
            .unwrap_or(false);

        if !has_tabs {
            let url = self.initial_url.clone().unwrap_or_else(|| "about:blank".to_string());
            let mut tabs = self.tabs.lock()
                .map_err(|_| WebViewError::LockError("Failed to lock tabs".to_string()))?;
            let id = tabs.create_tab(url);
            tabs.switch_to_tab(id);
        }

        // Navigate to initial URL (this logs "Navigating to: ...")
        if let Some(url) = self.initial_url.clone() {
            self.navigate(&url)
                .map_err(|e| WebViewError::GenericError(e))?;
        }

        // Drive the replay player if it was started before run() was called.
        // Lock briefly to get state + next event, then release before executing.
        loop {
            let (state, next_event) = if let Ok(mut player) = self.player.lock() {
                let state = player.get_state();
                let next = if state == PlaybackState::Playing {
                    player.next_event()
                } else {
                    None
                };
                (state, next)
            } else {
                break;
            };

            match state {
                PlaybackState::Playing => {
                    if let Some(event) = next_event {
                        if let Err(e) = self.execute_event(event) {
                            error!("Replay event execution failed: {}", e);
                        }
                    } else {
                        std::thread::sleep(Duration::from_millis(1));
                    }
                }
                PlaybackState::Paused => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                PlaybackState::Completed | PlaybackState::Stopped => break,
            }
        }

        Ok(())
    }

    pub fn create_tab(&mut self, url: &str) -> Result<usize, WebViewError> {
        // Create the tab in the manager — release lock before calling update_tab_visibility
        let (id, is_first) = {
            let mut tabs = self.tabs.lock()
                .map_err(|_| WebViewError::LockError("Failed to lock tab manager".to_string()))?;
            let id = tabs.create_tab(url.to_string());

            // Update the tab bar
            if let Some(ref tab_bar) = self.tab_bar {
                tab_bar.update_tab_url(id, url);
            }

            // Publish tab created event
            self.publish_event(BrowserEvent::TabCreated {
                id,
                url: url.to_string()
            }).map_err(|e| WebViewError::GenericError(e.to_string()))?;

            let is_first = tabs.get_all_tabs().len() == 1;
            if is_first {
                tabs.switch_to_tab(id);
            }
            (id, is_first)
        }; // tabs lock released here

        if is_first {
            self.update_tab_visibility()?;
            self.publish_event(BrowserEvent::TabActivated { id })
                .map_err(|e| WebViewError::GenericError(e.to_string()))?;
        }

        Ok(id)
    }

    pub fn switch_to_tab(&mut self, id: usize) -> Result<(), WebViewError> {
        // Switch in manager — release lock before calling update_tab_visibility
        let switched = {
            let mut tabs = self.tabs.lock()
                .map_err(|_| WebViewError::LockError("Failed to lock tabs".to_string()))?;
            tabs.switch_to_tab(id)
        }; // tabs lock released here

        if switched {
            self.update_tab_visibility()?;
            self.publish_event(BrowserEvent::TabActivated { id })
                .map_err(|e| WebViewError::GenericError(e.to_string()))?;
            Ok(())
        } else {
            Err(WebViewError::TabError(format!("Failed to switch to tab {}", id)))
        }
    }

    pub fn close_tab(&mut self, id: usize) -> Result<(), WebViewError> {
        // Get the next active tab ID before closing
        let next_active_id = {
            let tabs = self.tabs.lock()
                .map_err(|_| WebViewError::LockError("Failed to lock tabs".to_string()))?;
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
            let mut tabs = self.tabs.lock()
                .map_err(|_| WebViewError::LockError("Failed to lock tabs".to_string()))?;
            if !tabs.close_tab(id) {
                return Err(WebViewError::TabError("Tab not found".to_string()));
            }
            // Publish tab closed event
            self.publish_event(BrowserEvent::TabClosed { id })
                .map_err(|e| WebViewError::GenericError(e.to_string()))?;
        }

        // Switch to next tab if needed
        if let Some(next_id) = next_active_id {
            self.switch_to_tab(next_id)?;
        }

        Ok(())
    }

    /// Save current tabs to a session file.
    pub fn save_session(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        use session::{Session, SavedTab, save_session};
        let tabs_snapshot = self.tabs.lock()
            .map_err(|_| "Failed to lock tabs")?;
        let active_id = tabs_snapshot.get_active_tab().map(|t| t.id);
        let all_tabs = tabs_snapshot.get_all_tabs();
        let saved_tabs: Vec<SavedTab> = all_tabs.iter().map(|t| SavedTab {
            url: t.url.clone(),
            title: t.title.clone(),
        }).collect();
        // Determine active index by position in the collected vec
        let active_index = all_tabs.iter()
            .position(|t| Some(t.id) == active_id)
            .unwrap_or(0);
        let session = Session { tabs: saved_tabs, active_index };
        save_session(&session, path)
    }

    /// Restore tabs from a session file, replacing any existing tabs.
    pub fn restore_session(&mut self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        use session::load_session;
        let session = match load_session(path) {
            Some(s) if !s.tabs.is_empty() => s,
            _ => return Ok(()),
        };
        // Create a tab for each saved entry
        let mut last_id = None;
        for (i, saved) in session.tabs.iter().enumerate() {
            let id = self.create_tab(&saved.url)?;
            if !saved.title.is_empty() {
                if let Ok(mut tabs) = self.tabs.lock() {
                    tabs.update_tab_title(id, saved.title.clone());
                }
            }
            if i == session.active_index {
                last_id = Some(id);
            }
        }
        // Switch to the previously active tab
        if let Some(id) = last_id {
            self.switch_to_tab(id).ok();
        }
        info!("Session restored: {} tab(s)", session.tabs.len());
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

    /// Take a screenshot of the current active tab
    pub fn take_screenshot(&self, options: Option<ScreenshotOptions>) -> Result<ScreenshotResult, String> {
        let options = options.unwrap_or_default();
        
        if let Some(ref content_view) = self.content_view {
            if let Ok(view) = content_view.lock() {
                // For now, we'll create a mock screenshot since WebView screenshot requires platform-specific code
                // In a real implementation, this would capture the actual WebView content
                info!("Taking screenshot with options: {:?}", options);
                
                // Create a simple test pattern for demonstration
                let width = 800u32;
                let height = 600u32;
                let mut image_data = Vec::with_capacity((width * height * 4) as usize);
                
                // Generate a gradient pattern for testing
                for y in 0..height {
                    for x in 0..width {
                        let r = ((x as f32 / width as f32) * 255.0) as u8;
                        let g = ((y as f32 / height as f32) * 255.0) as u8;
                        let b = 128u8;
                        let a = 255u8;
                        image_data.extend_from_slice(&[r, g, b, a]);
                    }
                }
                
                if let Ok(visual_tester) = self.visual_tester.lock() {
                    match visual_tester.capture_from_data(&image_data, width, height, options) {
                        Ok(screenshot) => {
                            self.publish_event(BrowserEvent::CommandExecuted {
                                command: "screenshot".to_string(),
                                success: true,
                            }).ok();
                            Ok(screenshot)
                        },
                        Err(e) => {
                            let error_msg = format!("Failed to capture screenshot: {}", e);
                            self.publish_event(BrowserEvent::Error {
                                message: error_msg.clone(),
                            }).ok();
                            Err(error_msg)
                        }
                    }
                } else {
                    Err("Failed to lock visual tester".to_string())
                }
            } else {
                Err("Failed to lock content view".to_string())
            }
        } else {
            Err("No content view available".to_string())
        }
    }

    /// Save a screenshot to file
    pub fn save_screenshot(&self, screenshot: &ScreenshotResult, filename: &str) -> Result<String, String> {
        if let Ok(visual_tester) = self.visual_tester.lock() {
            visual_tester.save_screenshot(screenshot, filename)
                .map_err(|e| format!("Failed to save screenshot: {}", e))
        } else {
            Err("Failed to lock visual tester".to_string())
        }
    }

    /// Compare two screenshots
    pub fn compare_screenshots(&self, img1: &ScreenshotResult, img2: &ScreenshotResult, tolerance: f64) -> Result<visual::VisualComparisonResult, String> {
        if let Ok(visual_tester) = self.visual_tester.lock() {
            visual_tester.compare_screenshots(img1, img2, tolerance)
                .map_err(|e| format!("Failed to compare screenshots: {}", e))
        } else {
            Err("Failed to lock visual tester".to_string())
        }
    }

    /// Create a baseline screenshot for visual regression testing
    pub fn create_baseline(&self, test_name: &str, options: Option<ScreenshotOptions>) -> Result<String, String> {
        let screenshot = self.take_screenshot(options)?;
        
        if let Ok(visual_tester) = self.visual_tester.lock() {
            visual_tester.create_baseline(&screenshot, test_name)
                .map_err(|e| format!("Failed to create baseline: {}", e))
        } else {
            Err("Failed to lock visual tester".to_string())
        }
    }

    /// Run a visual regression test against a baseline
    pub fn run_visual_test(&self, test_name: &str, tolerance: f64, options: Option<ScreenshotOptions>) -> Result<visual::VisualComparisonResult, String> {
        let current_screenshot = self.take_screenshot(options)?;
        
        if let Ok(visual_tester) = self.visual_tester.lock() {
            let baseline = visual_tester.load_baseline(test_name)
                .map_err(|e| format!("Failed to load baseline '{}': {}", test_name, e))?;
            
            visual_tester.compare_screenshots(&baseline, &current_screenshot, tolerance)
                .map_err(|e| format!("Failed to compare with baseline: {}", e))
        } else {
            Err("Failed to lock visual tester".to_string())
        }
    }

    /// Execute JavaScript in the active WebView and return result
    pub fn execute_javascript(&self, script: &str) -> Result<String, String> {
        if let Some(ref content_view) = self.content_view {
            if let Ok(view) = content_view.lock() {
                // For now, we'll simulate JavaScript execution since WebView evaluation is async
                // In a real implementation, this would use WebView's evaluate_script method
                debug!("Executing JavaScript: {}", script);
                
                // Mock result for demonstration - in real implementation this would be the actual JS result
                Ok("{}".to_string())
            } else {
                Err("Failed to lock content view".to_string())
            }
        } else {
            Err("No content view available".to_string())
        }
    }

    /// Find element using various selector strategies
    pub fn find_element(&self, selector: &ElementSelector) -> Result<Option<ElementInfo>, String> {
        if let Ok(inspector) = self.dom_inspector.lock() {
            let js_code = inspector.find_element(selector);
            let result = self.execute_javascript(&js_code)?;
            
            if result == "{}" || result.trim().is_empty() {
                Ok(None)
            } else {
                // In real implementation, parse the JSON result from JavaScript
                // For now, return a mock ElementInfo
                Ok(Some(ElementInfo {
                    tag_name: "div".to_string(),
                    attributes: std::collections::HashMap::new(),
                    text_content: "Mock element".to_string(),
                    inner_html: "<span>Mock content</span>".to_string(),
                    outer_html: "<div><span>Mock content</span></div>".to_string(),
                    computed_styles: std::collections::HashMap::new(),
                    bounds: inspector::ElementBounds {
                        x: 100.0,
                        y: 100.0,
                        width: 200.0,
                        height: 50.0,
                        viewport_x: 100.0,
                        viewport_y: 100.0,
                    },
                    is_visible: true,
                    is_enabled: true,
                    css_path: "div".to_string(),
                    xpath: "//div[1]".to_string(),
                }))
            }
        } else {
            Err("Failed to lock DOM inspector".to_string())
        }
    }

    /// Interact with an element
    pub fn interact_with_element(&self, selector: &ElementSelector, interaction: &InteractionType) -> Result<inspector::InteractionResult, String> {
        if let Ok(inspector) = self.dom_inspector.lock() {
            let js_code = inspector.interact_with_element(selector, interaction);
            let result = self.execute_javascript(&js_code)?;
            
            info!("Element interaction: {:?} on {:?}", interaction, selector);
            
            // Publish interaction event
            self.publish_event(BrowserEvent::CommandExecuted {
                command: format!("interact_element_{:?}", interaction),
                success: true,
            }).ok();
            
            // Return mock result
            Ok(inspector::InteractionResult {
                success: true,
                error: None,
                element_info: self.find_element(selector)?,
                screenshot_data: None,
            })
        } else {
            Err("Failed to lock DOM inspector".to_string())
        }
    }

    /// Highlight an element for visual debugging
    pub fn highlight_element(&self, selector: &ElementSelector, color: Option<&str>) -> Result<Option<ElementInfo>, String> {
        if let Ok(inspector) = self.dom_inspector.lock() {
            let js_code = inspector.highlight_element(selector, color);
            let result = self.execute_javascript(&js_code)?;
            
            info!("Highlighting element: {:?}", selector);
            
            self.find_element(selector)
        } else {
            Err("Failed to lock DOM inspector".to_string())
        }
    }

    /// Wait for a condition to be met
    pub fn wait_for_condition(&self, condition: &WaitCondition) -> Result<bool, String> {
        if let Ok(inspector) = self.dom_inspector.lock() {
            let start_time = std::time::Instant::now();
            let timeout = std::time::Duration::from_millis(condition.timeout_ms as u64);
            let poll_interval = std::time::Duration::from_millis(condition.poll_interval_ms as u64);
            
            while start_time.elapsed() < timeout {
                let js_code = inspector.check_wait_condition(condition);
                let result = self.execute_javascript(&js_code)?;
                
                // In real implementation, parse boolean result from JavaScript
                // For mock purposes, assume condition is met after 1 second
                if start_time.elapsed() > std::time::Duration::from_secs(1) {
                    info!("Wait condition met: {:?}", condition);
                    return Ok(true);
                }
                
                std::thread::sleep(poll_interval);
            }
            
            info!("Wait condition timeout: {:?}", condition);
            Ok(false)
        } else {
            Err("Failed to lock DOM inspector".to_string())
        }
    }

    /// Get page information
    pub fn get_page_info(&self) -> Result<serde_json::Value, String> {
        if let Ok(inspector) = self.dom_inspector.lock() {
            let js_code = inspector.get_page_info();
            let result = self.execute_javascript(&js_code)?;
            
            // Return mock page info
            Ok(serde_json::json!({
                "title": "Mock Page Title",
                "url": "https://example.com",
                "readyState": "complete",
                "elementCount": 42,
                "viewport": {
                    "width": 800,
                    "height": 600,
                    "scrollX": 0,
                    "scrollY": 0
                },
                "performance": {
                    "domContentLoaded": 500,
                    "loadComplete": 1200
                }
            }))
        } else {
            Err("Failed to lock DOM inspector".to_string())
        }
    }

    /// Find all elements matching a CSS selector
    pub fn find_all_elements(&self, css_selector: &str) -> Result<Vec<ElementInfo>, String> {
        if let Ok(inspector) = self.dom_inspector.lock() {
            let js_code = inspector.find_all_elements(css_selector);
            let result = self.execute_javascript(&js_code)?;

            info!("Finding all elements: {}", css_selector);

            // Return mock list of elements
            Ok(vec![])
        } else {
            Err("Failed to lock DOM inspector".to_string())
        }
    }

    /// Start performance monitoring
    pub fn start_performance_monitoring(&self) -> Result<(), String> {
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            monitor.start_monitoring();
            info!("Performance monitoring started");

            // Publish event
            self.publish_event(BrowserEvent::CommandExecuted {
                command: "start_performance_monitoring".to_string(),
                success: true,
            }).ok();

            Ok(())
        } else {
            Err("Failed to lock performance monitor".to_string())
        }
    }

    /// Stop performance monitoring
    pub fn stop_performance_monitoring(&self) -> Result<(), String> {
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            monitor.stop_monitoring();
            info!("Performance monitoring stopped");

            // Publish event
            self.publish_event(BrowserEvent::CommandExecuted {
                command: "stop_performance_monitoring".to_string(),
                success: true,
            }).ok();

            Ok(())
        } else {
            Err("Failed to lock performance monitor".to_string())
        }
    }

    /// Collect performance metrics from the current page
    pub fn collect_performance_metrics(&self) -> Result<serde_json::Value, String> {
        if let Ok(monitor) = self.performance_monitor.lock() {
            // Execute the performance collection script
            let script = monitor.generate_collection_script();
            let result = self.execute_javascript(&script)?;

            // Parse the result
            let metrics: serde_json::Value = serde_json::from_str(&result)
                .unwrap_or_else(|_| serde_json::json!({}));

            info!("Collected performance metrics");
            Ok(metrics)
        } else {
            Err("Failed to lock performance monitor".to_string())
        }
    }

    /// Get Core Web Vitals for the current page
    pub fn get_core_web_vitals(&self) -> Result<CoreWebVitals, String> {
        // Collect current metrics
        let metrics = self.collect_performance_metrics()?;

        // Parse Core Web Vitals from the metrics
        let vitals = CoreWebVitals {
            lcp: metrics["coreWebVitals"]["lcp"].as_f64(),
            fid: metrics["coreWebVitals"]["fid"].as_f64(),
            cls: metrics["coreWebVitals"]["cls"].as_f64(),
            inp: metrics["coreWebVitals"]["inp"].as_f64(),
            ttfb: metrics["coreWebVitals"]["ttfb"].as_f64(),
            fcp: metrics["coreWebVitals"]["fcp"].as_f64(),
        };

        // Update the monitor with the new vitals
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            monitor.update_core_web_vitals(vitals.clone());
        }

        Ok(vitals)
    }

    /// Get memory usage metrics
    pub fn get_memory_metrics(&self) -> Result<MemoryMetrics, String> {
        let metrics = self.collect_performance_metrics()?;

        let memory = MemoryMetrics {
            js_heap_size_limit: metrics["memory"]["js_heap_size_limit"].as_u64().unwrap_or(0),
            total_js_heap_size: metrics["memory"]["total_js_heap_size"].as_u64().unwrap_or(0),
            used_js_heap_size: metrics["memory"]["used_js_heap_size"].as_u64().unwrap_or(0),
            dom_node_count: metrics["memory"]["dom_node_count"].as_u64().unwrap_or(0) as u32,
            event_listener_count: metrics["memory"]["event_listener_count"].as_u64().unwrap_or(0) as u32,
            detached_node_count: metrics["memory"]["detached_node_count"].as_u64().unwrap_or(0) as u32,
            timestamp: metrics["memory"]["timestamp"].as_u64().unwrap_or(0),
        };

        // Add to monitor
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            monitor.add_memory_snapshot(memory.clone());
        }

        Ok(memory)
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> Result<serde_json::Value, String> {
        if let Ok(monitor) = self.performance_monitor.lock() {
            let summary = monitor.get_summary();
            Ok(serde_json::to_value(summary)
                .map_err(|e| format!("Failed to serialize performance summary: {}", e))?)
        } else {
            Err("Failed to lock performance monitor".to_string())
        }
    }

    /// Start JavaScript profiling
    pub fn start_js_profiling(&self, script_url: String) -> Result<String, String> {
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            let profile_id = monitor.start_js_profile(script_url);
            info!("Started JavaScript profiling: {}", profile_id);
            Ok(profile_id)
        } else {
            Err("Failed to lock performance monitor".to_string())
        }
    }

    /// Stop JavaScript profiling
    pub fn stop_js_profiling(&self) -> Result<Option<JavaScriptProfile>, String> {
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            let profile = monitor.stop_js_profile();
            if let Some(ref p) = profile {
                info!("Stopped JavaScript profiling: {} ({}ms)", p.id, p.duration);
            }
            Ok(profile)
        } else {
            Err("Failed to lock performance monitor".to_string())
        }
    }

    /// Add a custom performance marker
    pub fn add_performance_marker(&self, name: String, metadata: Option<HashMap<String, String>>) -> Result<(), String> {
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            monitor.add_marker(name, metadata);
            Ok(())
        } else {
            Err("Failed to lock performance monitor".to_string())
        }
    }

    /// Add a custom performance measure
    pub fn add_performance_measure(&self, name: String, start_mark: String, end_mark: String) -> Result<(), String> {
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            monitor.add_measure(name, start_mark, end_mark)
                .map_err(|e| e.to_string())
        } else {
            Err("Failed to lock performance monitor".to_string())
        }
    }

    /// Start console monitoring
    pub fn start_console_monitoring(&self) -> Result<(), String> {
        if let Ok(mut monitor) = self.console_monitor.lock() {
            monitor.start_monitoring();
            info!("Console monitoring started");

            // Inject console interceptor script
            let script = monitor.generate_injection_script();
            self.execute_javascript(&script)
                .map(|_| ())
                .map_err(|e| format!("Failed to inject console interceptor: {}", e))
        } else {
            Err("Failed to lock console monitor".to_string())
        }
    }

    /// Stop console monitoring
    pub fn stop_console_monitoring(&self) -> Result<(), String> {
        if let Ok(mut monitor) = self.console_monitor.lock() {
            monitor.stop_monitoring();
            info!("Console monitoring stopped");
            Ok(())
        } else {
            Err("Failed to lock console monitor".to_string())
        }
    }

    /// Get console logs, optionally filtered by level
    pub fn get_console_logs(&self, level: Option<ConsoleLevel>) -> Result<Vec<ConsoleMessage>, String> {
        if let Ok(monitor) = self.console_monitor.lock() {
            Ok(monitor.get_messages(level))
        } else {
            Err("Failed to lock console monitor".to_string())
        }
    }

    /// Clear console logs
    pub fn clear_console_logs(&self) -> Result<(), String> {
        if let Ok(mut monitor) = self.console_monitor.lock() {
            monitor.clear();
            Ok(())
        } else {
            Err("Failed to lock console monitor".to_string())
        }
    }

    /// Set console filter level
    pub fn set_console_filter(&self, level: Option<ConsoleLevel>) -> Result<(), String> {
        if let Ok(mut monitor) = self.console_monitor.lock() {
            monitor.set_filter(level);
            Ok(())
        } else {
            Err("Failed to lock console monitor".to_string())
        }
    }

    fn handle_command(&mut self, cmd: BrowserCommand) -> Result<(), WebViewError> {
        match cmd {
            BrowserCommand::CreateTab { url } => {
                self.create_tab(&url)?;
            }
            BrowserCommand::CloseTab { id } => {
                self.close_tab(id)?;
            }
            BrowserCommand::SwitchTab { id } => {
                self.switch_to_tab(id)?;
            }
            BrowserCommand::Navigate { url } => {
                if let Ok(tabs) = self.tabs.lock() {
                    if let Some(active_tab) = tabs.get_active_tab() {
                        if let Some(view) = &self.content_view {
                            if let Ok(view) = view.lock() {
                                view.load_url(&url);
                            }
                        }
                        // Update tab bar
                        if let Some(ref tab_bar) = self.tab_bar {
                            tab_bar.update_tab_url(active_tab.id, &url);
                        }
                    }
                }
            }
            BrowserCommand::RecordEvent { event } => {
                if let Ok(mut recorder) = self.recorder.lock() {
                    recorder.record_event(event);
                }
            }
            BrowserCommand::PlayEvent { event } => {
                if let Err(e) = self.execute_event(event) {
                    error!("Replay event execution failed: {}", e);
                }
            }
            BrowserCommand::TakeScreenshot { options } => {
                let screenshot_options = if let Some(opts) = options {
                    serde_json::from_value(opts).unwrap_or_default()
                } else {
                    ScreenshotOptions::default()
                };
                
                if let Ok(result) = self.take_screenshot(Some(screenshot_options)) {
                    info!("Screenshot taken: {}x{}", result.width, result.height);
                }
            }
            BrowserCommand::CreateBaseline { test_name, options } => {
                let screenshot_options = if let Some(opts) = options {
                    serde_json::from_value(opts).unwrap_or_default()
                } else {
                    ScreenshotOptions::default()
                };
                
                if let Ok(path) = self.create_baseline(&test_name, Some(screenshot_options)) {
                    info!("Baseline created: {}", path);
                }
            }
            BrowserCommand::RunVisualTest { test_name, tolerance, options } => {
                let screenshot_options = if let Some(opts) = options {
                    serde_json::from_value(opts).unwrap_or_default()
                } else {
                    ScreenshotOptions::default()
                };
                
                if let Ok(result) = self.run_visual_test(&test_name, tolerance, Some(screenshot_options)) {
                    info!("Visual test '{}' completed: {:.2}% difference", 
                          test_name, result.difference_percentage * 100.0);
                }
            }
            BrowserCommand::FindElement { selector } => {
                if let Ok(element_selector) = serde_json::from_value::<ElementSelector>(selector) {
                    if let Ok(element) = self.find_element(&element_selector) {
                        info!("Element found: {:?}", element.is_some());
                    }
                }
            }
            BrowserCommand::InteractElement { selector, interaction } => {
                if let (Ok(element_selector), Ok(interaction_type)) = (
                    serde_json::from_value::<ElementSelector>(selector),
                    serde_json::from_value::<InteractionType>(interaction)
                ) {
                    if let Ok(result) = self.interact_with_element(&element_selector, &interaction_type) {
                        info!("Element interaction successful: {}", result.success);
                    }
                }
            }
            BrowserCommand::HighlightElement { selector, color } => {
                if let Ok(element_selector) = serde_json::from_value::<ElementSelector>(selector) {
                    if let Ok(element) = self.highlight_element(&element_selector, color.as_deref()) {
                        info!("Element highlighted: {:?}", element.is_some());
                    }
                }
            }
            BrowserCommand::WaitForCondition { condition } => {
                if let Ok(wait_condition) = serde_json::from_value::<WaitCondition>(condition) {
                    if let Ok(met) = self.wait_for_condition(&wait_condition) {
                        info!("Wait condition result: {}", met);
                    }
                }
            }
            BrowserCommand::GetPageInfo => {
                if let Ok(page_info) = self.get_page_info() {
                    info!("Page info retrieved: {}", page_info);
                }
            }
            BrowserCommand::ExecuteJavaScript { script } => {
                if let Ok(result) = self.execute_javascript(&script) {
                    info!("JavaScript executed successfully");
                }
            }
            BrowserCommand::StartNetworkMonitoring => {
                if let Ok(mut monitor) = self.network_monitor.lock() {
                    monitor.start_monitoring();
                    info!("Network monitoring started");
                }
            }
            BrowserCommand::StopNetworkMonitoring => {
                if let Ok(mut monitor) = self.network_monitor.lock() {
                    monitor.stop_monitoring();
                    info!("Network monitoring stopped");
                }
            }
            BrowserCommand::GetNetworkStats => {
                if let Ok(monitor) = self.network_monitor.lock() {
                    let stats = monitor.get_stats();
                    info!("Network stats: {:?}", stats);
                }
            }
            BrowserCommand::ExportNetworkHAR => {
                if let Ok(monitor) = self.network_monitor.lock() {
                    if let Ok(har) = monitor.export_har() {
                        info!("Network HAR exported successfully");
                        // TODO: Save HAR to file or return via event
                    }
                }
            }
            BrowserCommand::AddNetworkFilter { filter } => {
                if let Ok(filter_obj) = serde_json::from_value::<NetworkFilter>(filter) {
                    if let Ok(mut monitor) = self.network_monitor.lock() {
                        monitor.add_filter(filter_obj);
                        info!("Network filter added");
                    }
                }
            }
            BrowserCommand::ClearNetworkFilters => {
                if let Ok(mut monitor) = self.network_monitor.lock() {
                    monitor.clear_filters();
                    info!("Network filters cleared");
                }
            }
            // Performance monitoring commands
            BrowserCommand::StartPerformanceMonitoring => {
                if let Ok(_) = self.start_performance_monitoring() {
                    info!("Performance monitoring started via command");
                }
            }
            BrowserCommand::StopPerformanceMonitoring => {
                if let Ok(_) = self.stop_performance_monitoring() {
                    info!("Performance monitoring stopped via command");
                }
            }
            BrowserCommand::CollectPerformanceMetrics => {
                if let Ok(metrics) = self.collect_performance_metrics() {
                    self.publish_event(BrowserEvent::PerformanceMetricsCollected { metrics }).ok();
                }
            }
            BrowserCommand::GetCoreWebVitals => {
                if let Ok(vitals) = self.get_core_web_vitals() {
                    if let Ok(vitals_json) = serde_json::to_value(&vitals) {
                        self.publish_event(BrowserEvent::CoreWebVitalsUpdated { vitals: vitals_json }).ok();
                    }
                    info!("Core Web Vitals collected - Score: {}", vitals.get_score());
                }
            }
            BrowserCommand::GetMemoryMetrics => {
                if let Ok(memory) = self.get_memory_metrics() {
                    if let Ok(memory_json) = serde_json::to_value(&memory) {
                        self.publish_event(BrowserEvent::MemoryMetricsUpdated { metrics: memory_json }).ok();
                    }
                    info!("Memory metrics collected - {}% used", memory.heap_usage_percentage());
                }
            }
            BrowserCommand::GetPerformanceSummary => {
                if let Ok(summary) = self.get_performance_summary() {
                    info!("Performance summary: {:?}", summary);
                }
            }
            BrowserCommand::StartJSProfiling { script_url } => {
                if let Ok(profile_id) = self.start_js_profiling(script_url) {
                    self.publish_event(BrowserEvent::JSProfilingStarted { profile_id }).ok();
                }
            }
            BrowserCommand::StopJSProfiling => {
                if let Ok(Some(profile)) = self.stop_js_profiling() {
                    if let Ok(profile_json) = serde_json::to_value(&profile) {
                        self.publish_event(BrowserEvent::JSProfilingCompleted { profile: profile_json }).ok();
                    }
                }
            }
            BrowserCommand::AddPerformanceMarker { name, metadata } => {
                let meta_map = metadata.and_then(|v| {
                    if let serde_json::Value::Object(map) = v {
                        Some(map.into_iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                            .collect())
                    } else {
                        None
                    }
                });

                if let Ok(_) = self.add_performance_marker(name.clone(), meta_map) {
                    self.publish_event(BrowserEvent::PerformanceMarkerAdded { name }).ok();
                }
            }
            BrowserCommand::AddPerformanceMeasure { name, start_mark, end_mark } => {
                if let Ok(_) = self.add_performance_measure(name.clone(), start_mark, end_mark) {
                    // Get the duration from the monitor
                    if let Ok(monitor) = self.performance_monitor.lock() {
                        if let Some(measure) = monitor.get_measures().last() {
                            self.publish_event(BrowserEvent::PerformanceMeasureAdded {
                                name,
                                duration: measure.duration
                            }).ok();
                        }
                    }
                }
            }
            // Console monitoring commands
            BrowserCommand::StartConsoleMonitoring => {
                if let Ok(_) = self.start_console_monitoring() {
                    info!("Console monitoring started via command");
                }
            }
            BrowserCommand::StopConsoleMonitoring => {
                if let Ok(_) = self.stop_console_monitoring() {
                    info!("Console monitoring stopped via command");
                }
            }
            BrowserCommand::GetConsoleLogs { level } => {
                let console_level = level.as_ref().and_then(|l| ConsoleLevel::from_str(l));
                if let Ok(logs) = self.get_console_logs(console_level) {
                    // Publish console messages as events
                    for log in logs {
                        self.publish_event(BrowserEvent::ConsoleMessage {
                            level: log.level.as_str().to_string(),
                            message: log.message,
                            timestamp: log.timestamp,
                            stack_trace: log.stack_trace,
                        }).ok();
                    }
                }
            }
            BrowserCommand::ClearConsoleLogs => {
                if let Ok(_) = self.clear_console_logs() {
                    self.publish_event(BrowserEvent::ConsoleCleared).ok();
                    info!("Console logs cleared via command");
                }
            }
            BrowserCommand::SetConsoleFilter { level } => {
                let console_level = level.as_ref().and_then(|l| ConsoleLevel::from_str(l));
                if let Ok(_) = self.set_console_filter(console_level) {
                    info!("Console filter set via command");
                }
            }
            // Recording/Replay commands
            BrowserCommand::StartRecording { name, start_url } => {
                if let Ok(_) = self.start_recording(name, start_url) {
                    info!("Recording started via command");
                }
            }
            BrowserCommand::StopRecording => {
                if let Ok(_) = self.stop_recording() {
                    info!("Recording stopped via command");
                }
            }
            BrowserCommand::PauseRecording => {
                if let Ok(_) = self.pause_recording() {
                    info!("Recording paused via command");
                }
            }
            BrowserCommand::ResumeRecording => {
                if let Ok(_) = self.resume_recording() {
                    info!("Recording resumed via command");
                }
            }
            BrowserCommand::SaveRecording { path } => {
                if let Ok(_) = self.save_recording(&path) {
                    info!("Recording saved to: {}", path);
                }
            }
            BrowserCommand::LoadRecording { path } => {
                if let Ok(_) = self.load_recording(&path) {
                    info!("Recording loaded from: {}", path);
                }
            }
            BrowserCommand::AddRecordingAssertion { expected_state, description } => {
                if let Ok(_) = self.add_recording_assertion(expected_state, description) {
                    info!("Assertion added to recording via command");
                }
            }
            BrowserCommand::EnableRecordingSnapshots { interval_ms } => {
                if let Ok(_) = self.enable_recording_snapshots(interval_ms) {
                    info!("Recording snapshots enabled via command");
                }
            }
            BrowserCommand::StartPlayback => {
                if let Ok(_) = self.start_replay() {
                    info!("Playback started via command");
                }
            }
            BrowserCommand::StopPlayback => {
                if let Ok(_) = self.stop_replay() {
                    info!("Playback stopped via command");
                }
            }
            BrowserCommand::PausePlayback => {
                if let Ok(_) = self.pause_replay() {
                    info!("Playback paused via command");
                }
            }
            BrowserCommand::ResumePlayback => {
                if let Ok(_) = self.resume_replay() {
                    info!("Playback resumed via command");
                }
            }
            BrowserCommand::SeekPlayback { timestamp_ms } => {
                if let Ok(_) = self.seek_replay(timestamp_ms) {
                    info!("Playback seeked to {}ms via command", timestamp_ms);
                }
            }
            BrowserCommand::StepForward => {
                if let Ok(Some(event)) = self.step_forward_replay() {
                    info!("Stepped forward, playing event: {:?}", event);
                    // Play the event
                    if let Ok(mut recorder) = self.recorder.lock() {
                        recorder.record_event(event);
                    }
                }
            }
            BrowserCommand::StepBackward => {
                if let Ok(_) = self.step_backward_replay() {
                    info!("Stepped backward via command");
                }
            }
            BrowserCommand::SetPlaybackSpeed { speed } => {
                if let Ok(_) = self.set_replay_speed(speed) {
                    info!("Playback speed set to {}x via command", speed);
                }
            }
            BrowserCommand::SetPlaybackLoop { enable } => {
                if let Ok(_) = self.set_replay_loop(enable) {
                    info!("Playback loop set to {} via command", enable);
                }
            }
            BrowserCommand::GetPlaybackState => {
                if let Ok(state) = self.get_replay_state() {
                    info!("Playback state: {:?}", state);
                }
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Event<()>) -> Result<(), WebViewError> {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        debug!("Window close requested");
                        self.running = false;
                    }
                    WindowEvent::Resized(size) => {
                        debug!("Window resized to: {:?}", size);
                        if let Some(view) = &self.content_view {
                            if let Ok(view) = view.lock() {
                                let tab_height: u32 = 40; // Match the tab bar height
                                let bounds = wry::Rect {
                                    x: 0,
                                    y: tab_height as i32,
                                    width: size.width,
                                    height: size.height.saturating_sub(tab_height),
                                };
                                view.set_bounds(bounds);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                // Update window here
            }
            _ => {}
        }
        Ok(())
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

    fn update_tab_visibility(&self) -> Result<(), WebViewError> {
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
            Err(WebViewError::LockError("Failed to lock tab manager".to_string()))
        }
    }

    fn create_content_view(&mut self, window: &Window) -> Result<WebView, WebViewError> {
        debug!("Starting content view creation");
        let tab_height: u32 = 40; // Match the tab bar height
        
        let window_size = window.inner_size();
        debug!("Window size for content view: {:?}", window_size);
        
        let webview_bounds = wry::Rect {
            x: 0,
            y: tab_height as i32,
            width: window_size.width,
            height: window_size.height.saturating_sub(tab_height),
        };
        debug!("Creating WebView with bounds: {:?}", webview_bounds);

        debug!("Creating WebView");
        let builder = WebViewBuilder::new(window)
            .with_bounds(webview_bounds)
            .with_visible(true)  // Ensure WebView is visible
            .with_transparent(false)
            .with_initialization_script(include_str!("../templates/window_chrome.js"))
            .with_html(include_str!("../templates/window_chrome.html"))?;
            
        debug!("Created WebViewBuilder");
        
        let webview = builder
            .build()
            .map_err(WebViewError::InitError)?;

        debug!("WebView created successfully");
        Ok(webview)
    }

    fn update_webview_bounds(&self, window: &Window) {
        let tab_height: u32 = 40;

        // Update tab bar bounds
        if let Some(ref tab_bar) = self.tab_bar {
            tab_bar.update_bounds(window);
        }

        // Update content view bounds and ensure visibility
        if let Some(ref content_view) = self.content_view {
            if let Ok(view) = content_view.lock() {
                view.set_bounds(wry::Rect {
                    x: 0_i32,
                    y: tab_height as i32,
                    width: window.inner_size().width,
                    height: window.inner_size().height.saturating_sub(tab_height),
                });
                view.set_visible(true);  // Ensure WebView remains visible after bounds update
            }
        }
    }

    /// Handle keyboard and window events with proper error handling and state management.
    ///
    /// # Supported keyboard shortcuts:
    /// - Ctrl+T: Create new tab
    /// - Ctrl+W: Close current tab
    /// - Ctrl+Tab: Switch to next tab
    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) -> Result<(), String> {
        match event {
            WindowEvent::Resized(size) => {
                debug!("Handling window resize: {:?}", size);
                self.update_webview_bounds(window);
                Ok(())
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use tao::keyboard::Key;
                use crate::browser::keyboard::{KeyCode, ModifiersState, handle_keyboard_input, KeyCommand};

                // Early return if not a key press or is a repeat
                if !matches!(event.state, ElementState::Pressed) || event.repeat {
                    return Ok(());
                }

                // Convert tao key event to our internal types
                let key_code = match &event.logical_key {
                    Key::Character(c) if *c == String::from("t") || *c == String::from("T") => Some(KeyCode::KeyT),
                    Key::Character(c) if *c == String::from("w") || *c == String::from("W") => Some(KeyCode::KeyW),
                    Key::Tab => Some(KeyCode::Digit1), // For now, map Tab to first tab
                    _ => None
                };

                // Create modifiers state
                let modifiers = ModifiersState {
                    ctrl: event.state == ElementState::Pressed,
                    alt: false,
                    shift: false,
                    meta: false,
                };

                // Handle the key command if we have a valid key code
                if let Some(key_code) = key_code {
                    if let Some(command) = handle_keyboard_input(key_code, modifiers) {
                        match command {
                            KeyCommand::NewTab => {
                                debug!("Creating new tab");
                                self.create_tab("about:blank").map_err(|e| e.to_string())?;
                            }
                            KeyCommand::CloseTab => {
                                debug!("Closing current tab");
                                // Get the tab ID before releasing the lock
                                let tab_id = if let Ok(tabs) = self.tabs.lock() {
                                    tabs.get_active_tab().map(|tab| tab.id)
                                } else {
                                    None
                                };

                                // Close the tab if we got an ID
                                if let Some(id) = tab_id {
                                    self.close_tab(id).map_err(|e| e.to_string())?;
                                }
                            }
                            KeyCommand::SwitchTab(index) => {
                                debug!("Switching to tab {}", index);
                                // Get the tab ID before releasing the lock
                                let tab_id = if let Ok(tabs) = self.tabs.lock() {
                                    tabs.get_all_tabs().get(index).map(|tab| tab.id)
                                } else {
                                    None
                                };

                                // Switch to the tab if we got an ID
                                if let Some(id) = tab_id {
                                    self.switch_to_tab(id).map_err(|e| e.to_string())?;
                                }
                            }
                            KeyCommand::Back => {
                                debug!("Navigating back");
                                if let Err(e) = self.go_back() {
                                    error!("Failed to go back: {}", e);
                                }
                            }
                            KeyCommand::Forward => {
                                debug!("Navigating forward");
                                if let Err(e) = self.go_forward() {
                                    error!("Failed to go forward: {}", e);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(())
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
            running: self.running,
            visual_tester: self.visual_tester.clone(),
            dom_inspector: self.dom_inspector.clone(),
            network_monitor: self.network_monitor.clone(),
            performance_monitor: self.performance_monitor.clone(),
            console_monitor: self.console_monitor.clone(),
        }
    }
}
