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
pub mod keyboard;

use self::{
    tabs::TabManager,
    event_viewer::EventViewer,
    tab_ui::{TabBar, TabCommand},
    replay::{EventRecorder, EventPlayer},
    visual::{VisualTester, ScreenshotOptions, ScreenshotResult},
    inspector::{DOMInspector, ElementSelector, InteractionType, ElementInfo, WaitCondition},
    network::{NetworkMonitor, NetworkRequest, NetworkResponse, NetworkFilter},
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

    pub fn run(&mut self) -> Result<(), WebViewError> {
        debug!("Starting browser engine");
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

    pub fn create_tab(&mut self, url: &str) -> Result<usize, WebViewError> {
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
            }).map_err(|e| WebViewError::GenericError(e.to_string()))?;

            // If this is the first tab, make it active
            if tabs.get_all_tabs().len() == 1 {
                tabs.switch_to_tab(id);
                self.update_tab_visibility()?;
                self.publish_event(BrowserEvent::TabActivated { id })
                    .map_err(|e| WebViewError::GenericError(e.to_string()))?;
            }

            Ok(id)
        } else {
            Err(WebViewError::LockError("Failed to lock tab manager".to_string()))
        }?;

        Ok(id)
    }

    pub fn switch_to_tab(&mut self, id: usize) -> Result<(), WebViewError> {
        // First switch the tab in the manager
        if let Ok(mut tabs) = self.tabs.lock() {
            if tabs.switch_to_tab(id) {
                // Update WebView content and tab bar
                self.update_tab_visibility()?;

                // Publish tab activated event
                self.publish_event(BrowserEvent::TabActivated { id })
                    .map_err(|e| WebViewError::GenericError(e.to_string()))?;
                Ok(())
            } else {
                Err(WebViewError::TabError(format!("Failed to switch to tab {}", id)))
            }
        } else {
            Err(WebViewError::LockError("Failed to lock tabs".to_string()))
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
                if let Ok(mut player) = self.player.lock() {
                    player.play_event(event);
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
                            _ => {} // Ignore other commands for now
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
        }
    }
}
