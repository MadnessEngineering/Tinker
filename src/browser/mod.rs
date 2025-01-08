//! Browser engine implementation

use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use wry::{WebView, WebViewBuilder};
use tracing::debug;
use crate::event::{EventSystem, BrowserEvent};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::{Instant, Duration};

mod tabs;
mod event_viewer;
mod tab_ui;
mod replay;

use tabs::TabManager;
use event_viewer::EventViewer;
use tab_ui::{TabBar, TabCommand};
use replay::{EventRecorder, EventPlayer};

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
    content_view: Option<WebView>,
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
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Starting browser engine...");

        let event_loop = EventLoop::new();
        let mut window_builder = WindowBuilder::new()
            .with_title("Tinker Workshop")
            .with_inner_size(LogicalSize::new(1024.0, 768.0));

        if self.headless {
            window_builder = window_builder.with_visible(false);
        }

        let window = window_builder.build(&event_loop)?;

        // Create the content view
        if let Ok(tabs) = self.tabs.lock() {
            if let Some(active_tab) = tabs.get_active_tab() {
                let content_view = WebViewBuilder::new(&window)
                    .with_url(&active_tab.url)?
                    .build()?;
                self.content_view = Some(content_view);
            }
        }

        // Create channels for browser commands
        let (cmd_tx, cmd_rx) = channel::<BrowserCommand>();

        // Create the tab bar if not in headless mode
        if !self.headless {
            // Create the tab bar with the command sender
            let tab_cmd_tx = cmd_tx.clone();
            self.tab_bar = Some(TabBar::new(&window, move |cmd| {
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
            })?);
            
            // Add existing tabs to the UI
            if let Ok(tabs) = self.tabs.lock() {
                if let Some(tab_bar) = &self.tab_bar {
                    for tab in tabs.get_all_tabs() {
                        tab_bar.add_tab(tab.id, &tab.title, &tab.url);
                    }
                    if let Some(active_tab) = tabs.get_active_tab() {
                        tab_bar.set_active_tab(active_tab.id);
                    }
                }
            }
        }

        debug!("Running event loop...");

        // Emit initial events
        if let Some(events) = &self.events {
            if let Ok(mut events) = events.lock() {
                if let Ok(tabs) = self.tabs.lock() {
                    if let Some(active_tab) = tabs.get_active_tab() {
                        events.publish(BrowserEvent::PageLoaded {
                            url: active_tab.url.clone(),
                        })?;
                    }
                }
            }
        }

        let headless = self.headless;
        let events = self.events.clone();
        let player = self.player.clone();
        let recorder = self.recorder.clone();
        let event_viewer = self.event_viewer.clone();
        let tabs = self.tabs.clone();
        let tab_bar = self.tab_bar.clone();
        let content_view = self.content_view.clone();
        let mut last_replay_time = Instant::now();
        
        event_loop.run(move |event, _, control_flow| {
            *control_flow = if headless {
                ControlFlow::Exit
            } else {
                ControlFlow::Wait
            };

            // Check for replay events
            if let Ok(mut player) = player.lock() {
                if let Some(event) = player.next_event() {
                    let now = Instant::now();
                    if now.duration_since(last_replay_time) >= Duration::from_millis(100) {
                        match &event {
                            BrowserEvent::Navigation { url } => {
                                if let Some(view) = &content_view {
                                    view.load_url(url);
                                }
                            }
                            BrowserEvent::TabCreated { .. } => {
                                if let Ok(mut tabs) = tabs.lock() {
                                    let id = tabs.create_tab("about:blank".to_string());
                                    if let Some(bar) = &tab_bar {
                                        bar.add_tab(id, "New Tab", "about:blank");
                                        bar.set_active_tab(id);
                                    }
                                }
                            }
                            BrowserEvent::TabClosed { id } => {
                                if let Ok(mut tabs) = tabs.lock() {
                                    if tabs.close_tab(*id) {
                                        if let Some(bar) = &tab_bar {
                                            bar.remove_tab(*id);
                                        }
                                    }
                                }
                            }
                            BrowserEvent::TabSwitched { id } => {
                                if let Ok(mut tabs) = tabs.lock() {
                                    if tabs.switch_to_tab(*id) {
                                        if let Some(bar) = &tab_bar {
                                            bar.set_active_tab(*id);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        if let Ok(mut recorder) = recorder.lock() {
                            recorder.record_event(event.clone());
                        }
                        if let Ok(mut event_viewer) = event_viewer.lock() {
                            event_viewer.add_event(event);
                        }
                        last_replay_time = now;
                    }
                }
            }

            // Handle browser commands
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    BrowserCommand::Navigate { url } => {
                        if let Some(view) = &content_view {
                            view.load_url(&url);
                        }
                    }
                    BrowserCommand::CreateTab { url } => {
                        if let Ok(mut tabs) = tabs.lock() {
                            let id = tabs.create_tab(url.clone());
                            if let Some(bar) = &tab_bar {
                                bar.add_tab(id, "New Tab", &url);
                                bar.set_active_tab(id);
                            }
                            if let Some(view) = &content_view {
                                view.load_url(&url);
                            }
                            if let Some(events) = &events {
                                if let Ok(mut events) = events.lock() {
                                    let _ = events.publish(BrowserEvent::TabCreated { id });
                                }
                            }
                        }
                    }
                    BrowserCommand::CloseTab { id } => {
                        if let Ok(mut tabs) = tabs.lock() {
                            if tabs.close_tab(id) {
                                if let Some(bar) = &tab_bar {
                                    bar.remove_tab(id);
                                    if let Some(active_tab) = tabs.get_active_tab() {
                                        bar.set_active_tab(active_tab.id);
                                        if let Some(view) = &content_view {
                                            view.load_url(&active_tab.url);
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
                                if let Some(bar) = &tab_bar {
                                    bar.set_active_tab(id);
                                }
                                if let Some(active_tab) = tabs.get_active_tab() {
                                    if let Some(view) = &content_view {
                                        view.load_url(&active_tab.url);
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
                                if let Some(view) = &content_view {
                                    view.load_url(url);
                                }
                            }
                            BrowserEvent::TabCreated { .. } => {
                                if let Ok(mut tabs) = tabs.lock() {
                                    let id = tabs.create_tab("about:blank".to_string());
                                    if let Some(bar) = &tab_bar {
                                        bar.add_tab(id, "New Tab", "about:blank");
                                        bar.set_active_tab(id);
                                    }
                                }
                            }
                            BrowserEvent::TabClosed { id } => {
                                if let Ok(mut tabs) = tabs.lock() {
                                    if tabs.close_tab(*id) {
                                        if let Some(bar) = &tab_bar {
                                            bar.remove_tab(*id);
                                        }
                                    }
                                }
                            }
                            BrowserEvent::TabSwitched { id } => {
                                if let Ok(mut tabs) = tabs.lock() {
                                    if tabs.switch_to_tab(*id) {
                                        if let Some(bar) = &tab_bar {
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
                Event::NewEvents(StartCause::Init) => {
                    debug!("Browser window initialized");
                    if let Some(events) = &events {
                        if let Ok(mut events) = events.lock() {
                            let _ = events.publish(BrowserEvent::TitleChanged {
                                title: "Tinker Workshop".to_string(),
                            });
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });

        #[allow(unreachable_code)]
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_navigation() {
        let mut browser = BrowserEngine::new(false, None);
        browser.navigate("https://www.example.com").unwrap();
        if let Some(tab) = browser.tabs.get_active_tab() {
            assert_eq!(tab.url, "https://www.example.com");
        }
    }
}
