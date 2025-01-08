//! Browser engine implementation

use std::{
    sync::{Arc, Mutex, mpsc::{channel, Sender}},
    time::{Duration, Instant},
};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    dpi::{LogicalSize, PhysicalSize},
};
use wry::{WebView, WebViewBuilder};
use tracing::{debug, info, error};

use crate::event::{BrowserEvent, EventSystem};

mod tabs;
mod event_viewer;
mod tab_ui;
mod replay;
mod menu;

use self::{
    tabs::TabManager,
    event_viewer::EventViewer,
    tab_ui::{TabBar, TabCommand},
    replay::{EventRecorder, EventPlayer},
    menu::{create_application_menu, MenuCommand},
};

#[derive(Debug)]
enum BrowserCommand {
    Navigate { url: String },
    CreateTab { url: String },
    CloseTab { id: usize },
    SwitchTab { id: usize },
    RecordEvent { event: BrowserEvent },
    PlayEvent { event: BrowserEvent },
    Menu(MenuCommand),
}

#[derive(Clone)]
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
    cmd_tx: Option<Sender<BrowserCommand>>,
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
            cmd_tx: None,
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

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new();
        
        // Create the main window with decorations
        let window = Arc::new(WindowBuilder::new()
            .with_title("Tinker Browser")
            .with_inner_size(LogicalSize::new(1024.0, 768.0))
            .with_decorations(true)  // Enable window decorations
            .build(&event_loop)?);

        // Set up command channels
        let (browser_cmd_tx, browser_cmd_rx) = channel();
        let (menu_cmd_tx, menu_cmd_rx) = channel();

        // Create the native menu if not in headless mode
        if !self.headless {
            let _ = create_application_menu(&window, menu_cmd_tx);
        }

        self.window = Some(window.clone());
        self.cmd_tx = Some(browser_cmd_tx.clone());

        // Create the tab bar first if not in headless mode
        let tab_bar_height = if !self.headless {
            let tab_cmd_tx = browser_cmd_tx.clone();
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

            self.tab_bar = Some(tab_bar);
            40 // Tab bar height
        } else {
            0
        };

        // Create initial tab if none exists
        if let Ok(mut tabs) = self.tabs.lock() {
            if tabs.get_all_tabs().is_empty() {
                let default_url = "https://github.com/DanEdens/Tinker";
                let id = tabs.create_tab(default_url.to_string());

                // Create the initial WebView
                let window_size = window.inner_size();
                let content_view = WebViewBuilder::new(&window)
                    .with_url(default_url)
                    .map_err(|e| {
                        error!("Failed to set initial WebView URL: {}", e);
                        Box::new(e) as Box<dyn std::error::Error>
                    })?
                    .with_initialization_script("window.addEventListener('DOMContentLoaded', () => { document.body.style.backgroundColor = '#ffffff'; });")
                    .with_transparent(false)
                    .with_visible(true)
                    .with_bounds(wry::Rect {
                        x: 0,
                        y: tab_bar_height as i32,
                        width: window_size.width,
                        height: window_size.height.saturating_sub(tab_bar_height),
                    })
                    .build()
                    .map_err(|e| {
                        error!("Failed to build initial WebView: {}", e);
                        Box::new(e) as Box<dyn std::error::Error>
                    })?;

                // Wrap in Arc<Mutex>
                let content_view = Arc::new(Mutex::new(content_view));

                // Store the WebView in both the tab and as current content view
                tabs.set_tab_webview(id, content_view.clone());
                self.content_view = Some(content_view);

                // Add the tab to the UI
                if let Some(ref tab_bar) = self.tab_bar {
                    tab_bar.add_tab(id, "New Tab", default_url);
                    tab_bar.set_active_tab(id);
                }
            }
        }

        // Set up event handling
        let headless = self.headless;
        let events = self.events.clone();
        let recorder = self.recorder.clone();
        let event_viewer = self.event_viewer.clone();
        let tabs = self.tabs.clone();
        let tab_bar = self.tab_bar.clone();
        let window = self.window.take();
        let content_view = self.content_view.clone();
        let engine = Arc::new(Mutex::new(self.clone()));
        let tab_bar_height = tab_bar_height;

        // Handle window resize events to adjust WebView sizes
        event_loop.run(move |event, _, control_flow| {
            *control_flow = if headless {
                ControlFlow::Exit
            } else {
                ControlFlow::Wait
            };

            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    // Resize the tab bar
                    if let Some(ref bar) = tab_bar {
                        if let Ok(container) = bar.container.lock() {
                            container.set_bounds(wry::Rect {
                                x: 0,
                                y: 0,
                                width: new_size.width,
                                height: tab_bar_height,
                            });
                        }
                    }

                    // Resize the content view
                    if let Some(ref view) = content_view {
                        if let Ok(view) = view.lock() {
                            view.set_bounds(wry::Rect {
                                x: 0,
                                y: tab_bar_height as i32,
                                width: new_size.width,
                                height: new_size.height.saturating_sub(tab_bar_height),
                            });
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            }

            // Handle browser commands
            while let Ok(cmd) = browser_cmd_rx.try_recv() {
                match cmd {
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
                                let window_size = window.inner_size();
                                match WebViewBuilder::new(&**window)
                                    .with_url(&url)
                                    .map_err(|e| {
                                        error!("Failed to set WebView URL: {}", e);
                                        e
                                    })
                                    .map(|builder| {
                                        builder
                                            .with_initialization_script("window.addEventListener('DOMContentLoaded', () => { document.body.style.backgroundColor = '#ffffff'; });")
                                            .with_transparent(false)
                                            .with_visible(true)
                                            .with_bounds(wry::Rect {
                                                x: 0,
                                                y: tab_bar_height as i32,
                                                width: window_size.width,
                                                height: window_size.height.saturating_sub(tab_bar_height),
                                            })
                                            .build()
                                    })
                                    .and_then(|result| result.map_err(|e| {
                                        error!("Failed to build WebView: {}", e);
                                        e
                                    })) {
                                    Ok(new_view) => {
                                        let new_view = Arc::new(Mutex::new(new_view));
                                        
                                        // Store the WebView in the tab
                                        tabs.set_tab_webview(id, new_view.clone());
                                        
                                        // Set as current content view
                                        if let Ok(mut engine) = engine.lock() {
                                            engine.content_view = Some(new_view);
                                        }
                                    }
                                    Err(e) => error!("Failed to create WebView: {}", e)
                                }
                            }
                        }
                    }
                    BrowserCommand::Navigate { url } => {
                        if let Some(ref view) = content_view {
                            if let Ok(view) = view.lock() {
                                view.load_url(&url);
                            }
                        }
                    }
                    BrowserCommand::CloseTab { id } => {
                        if let Ok(mut tabs) = tabs.lock() {
                            if tabs.close_tab(id) {
                                if let Some(ref bar) = tab_bar {
                                    bar.remove_tab(id);
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
                            }
                        }
                    }
                    BrowserCommand::RecordEvent { event } => {
                        if let Ok(mut recorder) = recorder.lock() {
                            recorder.record_event(event);
                        }
                    }
                    BrowserCommand::PlayEvent { event } => {
                        // For now, we'll just log the event since we need to implement proper event playback
                        debug!("Event playback not yet implemented: {:?}", event);
                    }
                    BrowserCommand::Menu(cmd) => {
                        if let Ok(mut engine) = engine.lock() {
                            engine.handle_menu_command(cmd);
                        }
                    }
                }
            }

            // Handle menu commands
            while let Ok(cmd) = menu_cmd_rx.try_recv() {
                if let Ok(mut engine) = engine.lock() {
                    engine.handle_menu_command(cmd);
                }
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

    fn handle_menu_command(&mut self, cmd: MenuCommand) {
        use menu::MenuCommand::*;
        match cmd {
            NewTab => {
                let _ = self.create_tab("about:blank");
            }
            NewWindow => {
                debug!("New window functionality not yet implemented");
            }
            CloseTab => {
                if let Ok(tabs) = self.tabs.lock() {
                    if let Some(active_tab) = tabs.get_active_tab() {
                        let id = active_tab.id;
                        drop(tabs); // Release the lock before sending command
                        if let Some(cmd_tx) = &self.cmd_tx {
                            let _ = cmd_tx.send(BrowserCommand::CloseTab { id });
                        }
                    }
                }
            }
            CloseWindow => {
                if let Some(window) = &self.window {
                    let _ = window.set_visible(false);
                    // The window will be closed when the event loop receives the CloseRequested event
                }
            }
            ZoomIn => {
                if let Some(view) = &self.content_view {
                    if let Ok(view) = view.lock() {
                        let _ = view.evaluate_script("document.body.style.zoom = (parseFloat(document.body.style.zoom || '1.0') * 1.1).toString()");
                    }
                }
            }
            ZoomOut => {
                if let Some(view) = &self.content_view {
                    if let Ok(view) = view.lock() {
                        let _ = view.evaluate_script("document.body.style.zoom = (parseFloat(document.body.style.zoom || '1.0') * 0.9).toString()");
                    }
                }
            }
            ZoomReset => {
                if let Some(view) = &self.content_view {
                    if let Ok(view) = view.lock() {
                        let _ = view.evaluate_script("document.body.style.zoom = '1.0'");
                    }
                }
            }
            ToggleDevTools => {
                debug!("DevTools functionality not yet implemented");
            }
            StartRecording => {
                self.start_recording("test_recording.json");
            }
            StopRecording => {
                self.stop_recording();
            }
            RunTests => {
                debug!("Test runner not yet implemented");
            }
            ViewTestReport => {
                debug!("Test report viewer not yet implemented");
            }
            SwitchEngine(engine) => {
                debug!("Engine switching not yet implemented: {}", engine);
            }
            OpenJsConsole => {
                debug!("JavaScript console not yet implemented");
            }
            OpenPerformanceMonitor => {
                debug!("Performance monitor not yet implemented");
            }
            OpenDocumentation => {
                let _ = self.create_tab("https://github.com/DanEdens/Tinker/wiki");
            }
            ReportIssue => {
                let _ = self.create_tab("https://github.com/DanEdens/Tinker/issues/new");
            }
            About => {
                debug!("About dialog not yet implemented");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_URL: &str = "https://github.com/DanEdens/Tinker";

    #[test]
    fn test_browser_navigation() {
        let mut browser = BrowserEngine::new(false, None);

        // Create initial tab and get its URL
        let initial_url = {
            let mut tabs = browser.tabs.lock().unwrap();
            let id = tabs.create_tab(DEFAULT_URL.to_string());
            tabs.get_active_tab().map(|tab| tab.url.clone()).unwrap()
        };
        assert_eq!(initial_url, DEFAULT_URL);

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
