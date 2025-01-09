//! Browser engine implementation

use std::{
    sync::{Arc, Mutex, mpsc::{channel, Sender}},
    time::{Duration, Instant},
    env,
};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    dpi::{LogicalSize, PhysicalSize},
};
use wry::{WebView, WebViewBuilder};
use tracing::{debug, info, error};
use dotenv::dotenv;

use crate::event::{BrowserEvent, EventSystem};
use crate::templates::get_window_chrome;

mod tabs;
mod event_viewer;
mod tab_ui;
mod replay;
mod menu;
mod native_ui;

use self::{
    tabs::TabManager,
    event_viewer::EventViewer,
    tab_ui::{TabBar, TabCommand},
    replay::{EventRecorder, EventPlayer},
    menu::{create_application_menu, MenuCommand},
    native_ui::NativeTabBar,
};

fn get_default_url() -> String {
    dotenv().ok(); // Load .env file
    env::var("DEFAULT_URL").unwrap_or_else(|_| "https://github.com/DanEdens/Tinker".to_string())
}

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
    tab_bar: Option<Arc<Mutex<NativeTabBar>>>,
    content_view: Option<Arc<Mutex<WebView>>>,
    window: Option<Arc<Window>>,
    cmd_tx: Option<Sender<BrowserCommand>>,
    chrome_view: Option<Arc<Mutex<WebView>>>,
}

impl BrowserEngine {
    pub fn new(headless: bool, broker_url: Option<&str>) -> Self {
        let events = Arc::new(Mutex::new(EventSystem::new(
            broker_url.unwrap_or("ws://localhost:8080"),
            "browser-client",
        )));

        BrowserEngine {
            tabs: Arc::new(Mutex::new(TabManager::new())),
            events: Some(events),
            player: Arc::new(Mutex::new(EventPlayer::default())),
            recorder: Arc::new(Mutex::new(EventRecorder::default())),
            event_viewer: Arc::new(Mutex::new(EventViewer::new())),
            headless,
            tab_bar: None,
            content_view: None,
            window: None,
            cmd_tx: None,
            chrome_view: None,
        }
    }

    pub fn navigate(&mut self, tab_id: usize, url: &str) -> Result<(), String> {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.get_tab_mut(tab_id) {
            // In headless mode, just update the URL without using WebView
            if self.headless {
                tab.url = url.to_string();
                Ok(())
            } else if let Some(webview) = tab.webview.as_ref() {
                let webview = webview.lock().unwrap();
                webview.load_url(url);
                Ok(())
            } else {
                Err("WebView not initialized".to_string())
            }
        } else {
            Err("Tab not found".to_string())
        }
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
            .with_decorations(true)  // Enable default decorations for now
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

        // Create initial tab if none exists
        if let Ok(mut tabs) = self.tabs.lock() {
            if tabs.get_all_tabs().is_empty() {
                let default_url = get_default_url();
                let id = tabs.create_tab(default_url.clone());

                // Create the initial WebView
                let window_size = window.inner_size();
                let content_view = WebViewBuilder::new(&window)
                    .with_url(&default_url)
                    .map_err(|e| {
                        error!("Failed to set initial WebView URL: {}", e);
                        Box::new(e) as Box<dyn std::error::Error>
                    })?
                    .with_initialization_script("
                        document.documentElement.style.background = '#ffffff';
                        document.body.style.background = '#ffffff';
                    ")
                    .with_transparent(false)
                    .with_visible(true)
                    .with_bounds(wry::Rect {
                        x: 0,
                        y: 80,  // Position below both title bar and tab bar (40px + 40px)
                        width: window_size.width,
                        height: window_size.height.saturating_sub(80),  // Subtract height of both bars
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
            }
        }

        // Create the window chrome if not in headless mode
        if !self.headless {
            let window_size = window.inner_size();
            let window_handle = window.clone();
            let chrome_view = WebViewBuilder::new(&window)
                .with_html(&get_window_chrome())?
                .with_transparent(true)
                .with_visible(true)
                .with_bounds(wry::Rect {
                    x: 0,
                    y: 0,
                    width: window_size.width,
                    height: 40,  // Fixed height for tab bar
                })
                .with_ipc_handler(move |msg: String| {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&msg) {
                        match json["type"].as_str() {
                            Some("WindowControl") => {
                                if let Some(action) = json["action"].as_str() {
                                    match action {
                                        "minimize" => window_handle.set_minimized(true),
                                        "maximize" => {
                                            if window_handle.is_maximized() {
                                                window_handle.set_maximized(false);
                                            } else {
                                                window_handle.set_maximized(true);
                                            }
                                        }
                                        "close" => window_handle.set_visible(false),
                                        _ => (),
                                    }
                                }
                            }
                            Some("TabCreated") => {
                                if let Some(url) = json["url"].as_str() {
                                    let _ = browser_cmd_tx.send(BrowserCommand::CreateTab {
                                        url: url.to_string(),
                                    });
                                }
                            }
                            Some("TabClosed") => {
                                if let Some(id) = json["id"].as_u64() {
                                    let _ = browser_cmd_tx.send(BrowserCommand::CloseTab {
                                        id: id as usize,
                                    });
                                }
                            }
                            Some("TabSwitched") => {
                                if let Some(id) = json["id"].as_u64() {
                                    let _ = browser_cmd_tx.send(BrowserCommand::SwitchTab {
                                        id: id as usize,
                                    });
                                }
                            }
                            _ => (),
                        }
                    }
                })
                .build()?;

            let chrome_view = Arc::new(Mutex::new(chrome_view));
            self.chrome_view = Some(chrome_view);
        }

        // Set up event handling
        let headless = self.headless;
        let events = self.events.clone();
        let player = self.player.clone();
        let recorder = self.recorder.clone();
        let event_viewer = self.event_viewer.clone();
        let tabs = self.tabs.clone();
        let recorder = self.recorder.clone();
        let tab_bar = self.tab_bar.clone();
        let mut content_view = self.content_view.clone();
        let cmd_tx = self.cmd_tx.clone();
        let window = window.clone();
        let chrome_view = self.chrome_view.clone();

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
                    // Resize the chrome view
                    if let Some(ref view) = chrome_view {
                        if let Ok(view) = view.lock() {
                            view.set_bounds(wry::Rect {
                                x: 0,
                                y: 0,
                                width: new_size.width,
                                height: 40,  // Fixed height for chrome/tab bar
                            });
                        }
                    }

                    // Resize the content view
                    if let Some(ref view) = content_view {
                        if let Ok(view) = view.lock() {
                            view.set_bounds(wry::Rect {
                                x: 0,
                                y: 80,  // Position below both title bar and tab bar
                                width: new_size.width,
                                height: new_size.height.saturating_sub(80),
                            });
                        }
                    }
                }
                Event::MainEventsCleared => {
                    // Handle browser commands
                    while let Ok(cmd) = browser_cmd_rx.try_recv() {
                        match cmd {
                            BrowserCommand::CreateTab { url } => {
                                let url = if url == "about:blank" {
                                    get_default_url()
                                } else {
                                    url
                                };

                                if let Ok(mut tabs_guard) = tabs.lock() {
                                    let id = tabs_guard.create_tab(url.clone());

                                    // Create a new WebView for the tab
                                    let window_size = window.inner_size();
                                    match WebViewBuilder::new(&*window)
                                        .with_url(&url)
                                        .map_err(|e| {
                                            error!("Failed to set URL: {}", e);
                                            e
                                        })
                                        .and_then(|builder| {
                                            builder
                                                .with_initialization_script("
                                                    document.documentElement.style.background = '#ffffff';
                                                    document.body.style.background = '#ffffff';
                                                ")
                                                .with_transparent(false)
                                                .with_visible(true)
                                                .with_bounds(wry::Rect {
                                                    x: 0,
                                                    y: 80,
                                                    width: window_size.width,
                                                    height: window_size.height.saturating_sub(80),
                                                })
                                                .build()
                                                .map_err(|e| {
                                                    error!("Failed to build WebView: {}", e);
                                                    e
                                                })
                                        }) {
                                        Ok(new_view) => {
                                            let new_view = Arc::new(Mutex::new(new_view));
                                            
                                            // Store the WebView in the tab
                                            tabs_guard.set_tab_webview(id, new_view.clone());
                                            
                                            // Update content view
                                            content_view = Some(new_view);

                                            // Notify UI of successful tab creation
                                            if let Some(ref chrome_view) = chrome_view {
                                                if let Ok(view) = chrome_view.lock() {
                                                    let _ = view.evaluate_script(&format!(
                                                        "addTab({}, '{}', '{}')",
                                                        id, "New Tab", url
                                                    ));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to create WebView: {}", e);
                                            // Remove the tab since WebView creation failed
                                            tabs_guard.close_tab(id);
                                            // Notify UI of failure
                                            if let Some(ref chrome_view) = chrome_view {
                                                if let Ok(view) = chrome_view.lock() {
                                                    let _ = view.evaluate_script(
                                                        "console.error('Failed to create new tab')"
                                                    );
                                                }
                                            }
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
                                if let Ok(mut tabs_guard) = tabs.lock() {
                                    if tabs_guard.close_tab(id) {
                                        if let Some(ref tab_bar) = tab_bar {
                                            if let Ok(mut bar) = tab_bar.lock() {
                                                bar.remove_tab(id);
                                            }
                                        }
                                    }
                                }
                            }
                            BrowserCommand::SwitchTab { id } => {
                                if let Ok(mut tabs_guard) = tabs.lock() {
                                    if tabs_guard.switch_to_tab(id) {
                                        if let Some(ref tab_bar) = tab_bar {
                                            if let Ok(mut bar) = tab_bar.lock() {
                                                bar.set_active_tab(id);
                                            }
                                        }
                                    }
                                }
                            }
                            BrowserCommand::RecordEvent { event } => {
                                if let Ok(mut recorder_guard) = recorder.lock() {
                                    recorder_guard.record_event(event);
                                }
                            }
                            BrowserCommand::PlayEvent { event } => {
                                // For now, we'll just log the event since we need to implement proper event playback
                                debug!("Event playback not yet implemented: {:?}", event);
                            }
                            BrowserCommand::Menu(cmd) => {
                                Self::handle_menu_command(
                                    &tabs,
                                    &tab_bar,
                                    &content_view,
                                    &cmd_tx,
                                    cmd
                                );
                            }
                        }
                    }

                    // Handle menu commands
                    while let Ok(cmd) = menu_cmd_rx.try_recv() {
                        Self::handle_menu_command(
                            &tabs,
                            &tab_bar,
                            &content_view,
                            &cmd_tx,
                            cmd
                        );
                    }
                }
                _ => {}
            }
        });

        #[allow(unreachable_code)]
        Ok(())
    }

    pub fn create_tab(&mut self, url: &str) -> Result<usize, String> {
        let default_url = get_default_url();
        let url = if url == "about:blank" { &default_url } else { url };

        let mut tabs = self.tabs.lock().unwrap();
        let id = tabs.create_tab(url.to_string());

        // Create WebView for the tab
        if !self.headless {
            if let Some(window) = &self.window {
                let webview = WebView::new(window).map_err(|e| e.to_string())?;
                webview.load_url(url);
                tabs.set_tab_webview(id, Arc::new(Mutex::new(webview)));
            }
        }

        Ok(id)
    }

    fn handle_menu_command(
        tabs: &Arc<Mutex<TabManager>>,
        tab_bar: &Option<Arc<Mutex<NativeTabBar>>>,
        content_view: &Option<Arc<Mutex<WebView>>>,
        cmd_tx: &Option<Sender<BrowserCommand>>,
        cmd: MenuCommand
    ) {
        match cmd {
            MenuCommand::NewTab => {
                if let Some(cmd_tx) = cmd_tx {
                    let _ = cmd_tx.send(BrowserCommand::CreateTab {
                        url: get_default_url(),
                    });
                }
            }
            MenuCommand::CloseTab => {
                if let Ok(tabs) = tabs.lock() {
                    if let Some(active_tab) = tabs.get_active_tab() {
                        if let Some(cmd_tx) = cmd_tx {
                            let _ = cmd_tx.send(BrowserCommand::CloseTab { 
                                id: active_tab.id 
                            });
                        }
                    }
                }
            }
            MenuCommand::NewWindow => {
                debug!("New window functionality not yet implemented");
            }
            MenuCommand::CloseWindow => {
                debug!("Close window functionality not yet implemented");
            }
            MenuCommand::ZoomIn => {
                if let Some(ref view) = content_view {
                    if let Ok(view) = view.lock() {
                        let _ = view.evaluate_script("document.body.style.zoom = (parseFloat(document.body.style.zoom || '1.0') * 1.1).toString()");
                    }
                }
            }
            MenuCommand::ZoomOut => {
                if let Some(ref view) = content_view {
                    if let Ok(view) = view.lock() {
                        let _ = view.evaluate_script("document.body.style.zoom = (parseFloat(document.body.style.zoom || '1.0') * 0.9).toString()");
                    }
                }
            }
            MenuCommand::ZoomReset => {
                if let Some(ref view) = content_view {
                    if let Ok(view) = view.lock() {
                        let _ = view.evaluate_script("document.body.style.zoom = '1.0'");
                    }
                }
            }
            MenuCommand::ToggleDevTools => {
                debug!("DevTools functionality not yet implemented");
            }
            MenuCommand::StartRecording => {
                debug!("Recording functionality not yet implemented");
            }
            MenuCommand::StopRecording => {
                debug!("Stop recording functionality not yet implemented");
            }
            MenuCommand::RunTests => {
                debug!("Test runner not yet implemented");
            }
            MenuCommand::ViewTestReport => {
                debug!("Test report viewer not yet implemented");
            }
            MenuCommand::SwitchEngine(_) => {
                debug!("Engine switching not yet implemented");
            }
            MenuCommand::OpenJsConsole => {
                debug!("JavaScript console not yet implemented");
            }
            MenuCommand::OpenPerformanceMonitor => {
                debug!("Performance monitor not yet implemented");
            }
            MenuCommand::OpenDocumentation => {
                if let Some(cmd_tx) = cmd_tx {
                    let _ = cmd_tx.send(BrowserCommand::CreateTab {
                        url: "https://github.com/DanEdens/Tinker/wiki".to_string(),
                    });
                }
            }
            MenuCommand::ReportIssue => {
                if let Some(cmd_tx) = cmd_tx {
                    let _ = cmd_tx.send(BrowserCommand::CreateTab {
                        url: "https://github.com/DanEdens/Tinker/issues/new".to_string(),
                    });
                }
            }
            MenuCommand::About => {
                debug!("About dialog not yet implemented");
            }
        }
    }

    pub fn add_tab(&self, id: usize, title: &str) {
        if let Some(tab_bar) = &self.tab_bar {
            if let Ok(mut bar) = tab_bar.lock() {
                bar.add_tab(id, title);
            }
        }
    }

    pub fn remove_tab(&self, id: usize) {
        if let Some(tab_bar) = &self.tab_bar {
            if let Ok(mut bar) = tab_bar.lock() {
                bar.remove_tab(id);
            }
        }
    }

    pub fn set_active_tab(&self, id: usize) {
        if let Some(tab_bar) = &self.tab_bar {
            if let Ok(mut bar) = tab_bar.lock() {
                bar.set_active_tab(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_url_from_env() {
        // Save the original environment variable if it exists
        let original = env::var("DEFAULT_URL").ok();

        // Test with environment variable set
        env::set_var("DEFAULT_URL", "https://github.com/DanEdens/Tinker");
        assert_eq!(get_default_url(), "https://github.com/DanEdens/Tinker");

        // Test fallback when environment variable is not set
        env::remove_var("DEFAULT_URL");
        assert_eq!(get_default_url(), "https://github.com/DanEdens/Tinker");

        // Restore the original environment variable if it existed
        if let Some(url) = original {
            env::set_var("DEFAULT_URL", url);
        }
    }

    #[test]
    fn test_tab_creation_with_invalid_url() {
        let mut browser = BrowserEngine::new(true, None);
        let result = browser.create_tab("not a valid url");
        // In headless mode, we don't validate URLs since we don't create WebViews
        assert!(result.is_ok(), "Tab creation should succeed in headless mode");
    }

    #[test]
    fn test_tab_creation_with_about_blank() {
        // Set up environment for test
        env::remove_var("DEFAULT_URL");
        
        let mut browser = BrowserEngine::new(true, None);
        let result = browser.create_tab("about:blank");
        assert!(result.is_ok(), "Failed to create tab with about:blank");
        
        if let Ok(tab) = result {
            let tabs = browser.tabs.lock().unwrap();
            let tab_info = tabs.get_tab_info(tab).unwrap();
            assert_eq!(tab_info.url, "https://github.com/DanEdens/Tinker", "about:blank should be replaced with default URL");
        }
    }

    #[test]
    fn test_tab_state_sync() {
        let mut browser = BrowserEngine::new(true, None);
        let tab_id = browser.create_tab("https://example.com").unwrap();
        
        let tabs = browser.tabs.lock().unwrap();
        assert!(tabs.get_tab_info(tab_id).is_some(), "Tab should exist after creation");
        assert_eq!(tabs.get_tab_count(), 1, "Should have exactly one tab");
    }

    #[test]
    fn test_browser_navigation() {
        let mut browser = BrowserEngine::new(true, None);
        let tab_id = browser.create_tab("https://example.com").unwrap();
        
        let result = browser.navigate(tab_id, "https://github.com");
        // In headless mode, navigation should succeed since we just update the URL
        assert!(result.is_ok(), "Navigation should succeed in headless mode");
        
        let tabs = browser.tabs.lock().unwrap();
        let tab_info = tabs.get_tab_info(tab_id).unwrap();
        assert_eq!(tab_info.url, "https://github.com", "URL should be updated in headless mode");
    }
}
