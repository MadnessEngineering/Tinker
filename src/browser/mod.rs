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
use std::sync::mpsc::{channel, Receiver};

mod tabs;
mod event_viewer;
mod tab_ui;

use tabs::TabManager;
use event_viewer::EventViewer;
use tab_ui::{TabBar, TabCommand};

pub struct BrowserEngine {
    headless: bool,
    events: Option<Arc<Mutex<EventSystem>>>,
    tabs: TabManager,
    event_viewer: EventViewer,
    tab_bar: Option<TabBar>,
    content_view: Option<WebView>,
    command_rx: Option<Receiver<TabCommand>>,
}

impl BrowserEngine {
    pub fn new() -> Self {
        BrowserEngine {
            headless: false,
            events: None,
            tabs: TabManager::new(),
            event_viewer: EventViewer::new(),
            tab_bar: None,
            content_view: None,
            command_rx: None,
        }
    }

    pub fn set_headless(&mut self, headless: bool) {
        self.headless = headless;
    }

    pub fn init_events(&mut self, broker_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut events = EventSystem::new(broker_url, "tinker-browser");
        events.connect()?;
        self.events = Some(Arc::new(Mutex::new(events)));
        Ok(())
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
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
        if let Some(active_tab) = self.tabs.get_active_tab() {
            let content_view = WebViewBuilder::new(&window)
                .with_url(&active_tab.url)?
                .build()?;
            self.content_view = Some(content_view);
        }

        // Create the tab bar if not in headless mode
        if !self.headless {
            // Create channel for tab commands
            let (command_tx, command_rx) = channel();
            self.command_rx = Some(command_rx);
            
            // Create the tab bar with the command sender
            self.tab_bar = Some(TabBar::new(&window, command_tx)?);
            
            // Add existing tabs to the UI
            if let Some(tab_bar) = &self.tab_bar {
                for tab in self.tabs.get_all_tabs() {
                    tab_bar.add_tab(tab.id, &tab.title, &tab.url);
                }
                if let Some(active_tab) = self.tabs.get_active_tab() {
                    tab_bar.set_active_tab(active_tab.id);
                }
            }
        }

        debug!("Running event loop...");

        // Emit initial events
        if let Some(events) = &self.events {
            if let Ok(mut events) = events.lock() {
                if let Some(active_tab) = self.tabs.get_active_tab() {
                    events.publish(BrowserEvent::PageLoaded {
                        url: active_tab.url.clone(),
                    })?;
                }
            }
        }

        let headless = self.headless;
        let events = self.events.clone();
        let command_rx = self.command_rx.take();
        
        event_loop.run(move |event, _, control_flow| {
            *control_flow = if headless {
                ControlFlow::Exit
            } else {
                ControlFlow::Wait
            };

            // Handle tab commands
            if let Some(rx) = &command_rx {
                while let Ok(command) = rx.try_recv() {
                    match command {
                        TabCommand::Create { url } => {
                            let _ = self.create_tab(&url);
                        }
                        TabCommand::Close { id } => {
                            let _ = self.close_tab(id);
                        }
                        TabCommand::Switch { id } => {
                            let _ = self.switch_to_tab(id);
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

    pub fn navigate(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Navigating to: {}", url);
        
        if let Some(tab) = self.tabs.get_active_tab_mut() {
            tab.url = url.to_string();
            
            // Update tab UI
            if let Some(tab_bar) = &self.tab_bar {
                tab_bar.update_tab(tab.id, &tab.title, url);
            }

            // Update content view
            if let Some(content_view) = &self.content_view {
                content_view.load_url(url);
            }
            
            if let Some(events) = &self.events {
                if let Ok(mut events) = events.lock() {
                    events.publish(BrowserEvent::Navigation {
                        url: url.to_string(),
                    })?;
                }
            }

            // Add to event viewer
            self.event_viewer.add_event(BrowserEvent::Navigation {
                url: url.to_string(),
            });
        }
        
        Ok(())
    }

    pub fn create_tab(&mut self, url: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let id = self.tabs.create_tab(url.to_string());
        
        // Update tab UI
        if let Some(tab_bar) = &self.tab_bar {
            tab_bar.add_tab(id, "New Tab", url);
            tab_bar.set_active_tab(id);
        }

        // Update content view
        if let Some(content_view) = &self.content_view {
            content_view.load_url(url);
        }
        
        if let Some(events) = &self.events {
            if let Ok(mut events) = events.lock() {
                events.publish(BrowserEvent::TabCreated { id })?;
            }
        }

        // Add to event viewer
        self.event_viewer.add_event(BrowserEvent::TabCreated { id });
        
        Ok(id)
    }

    pub fn close_tab(&mut self, id: usize) -> Result<(), Box<dyn std::error::Error>> {
        if self.tabs.close_tab(id) {
            // Update tab UI
            if let Some(tab_bar) = &self.tab_bar {
                tab_bar.remove_tab(id);
                if let Some(active_tab) = self.tabs.get_active_tab() {
                    tab_bar.set_active_tab(active_tab.id);
                    // Update content view
                    if let Some(content_view) = &self.content_view {
                        content_view.load_url(&active_tab.url);
                    }
                }
            }
            
            if let Some(events) = &self.events {
                if let Ok(mut events) = events.lock() {
                    events.publish(BrowserEvent::TabClosed { id })?;
                }
            }

            // Add to event viewer
            self.event_viewer.add_event(BrowserEvent::TabClosed { id });
        }
        
        Ok(())
    }

    pub fn switch_to_tab(&mut self, id: usize) -> Result<(), Box<dyn std::error::Error>> {
        if self.tabs.switch_to_tab(id) {
            // Update tab UI
            if let Some(tab_bar) = &self.tab_bar {
                tab_bar.set_active_tab(id);
            }

            // Update content view
            if let Some(content_view) = &self.content_view {
                if let Some(tab) = self.tabs.get_active_tab() {
                    content_view.load_url(&tab.url);
                }
            }
            
            if let Some(events) = &self.events {
                if let Ok(mut events) = events.lock() {
                    events.publish(BrowserEvent::TabSwitched { id })?;
                }
            }

            // Add to event viewer
            self.event_viewer.add_event(BrowserEvent::TabSwitched { id });
        }
        
        Ok(())
    }

    pub fn update_tab_title(&mut self, id: usize, title: String) -> Result<(), Box<dyn std::error::Error>> {
        if self.tabs.update_tab_title(id, title.clone()) {
            // Update tab UI
            if let Some(tab_bar) = &self.tab_bar {
                if let Some(tab) = self.tabs.get_active_tab() {
                    tab_bar.update_tab(id, &title, &tab.url);
                }
            }
            
            if let Some(events) = &self.events {
                if let Ok(mut events) = events.lock() {
                    events.publish(BrowserEvent::TabTitleChanged { id, title: title.clone() })?;
                }
            }

            // Add to event viewer
            self.event_viewer.add_event(BrowserEvent::TabTitleChanged { id, title });
        }
        
        Ok(())
    }

    pub fn update_tab_url(&mut self, id: usize, url: String) -> Result<(), Box<dyn std::error::Error>> {
        if self.tabs.update_tab_url(id, url.clone()) {
            // Update tab UI
            if let Some(tab_bar) = &self.tab_bar {
                if let Some(tab) = self.tabs.get_active_tab() {
                    tab_bar.update_tab(id, &tab.title, &url);
                }
            }

            // Update content view if this is the active tab
            if let Some(content_view) = &self.content_view {
                if let Some(tab) = self.tabs.get_active_tab() {
                    if tab.id == id {
                        content_view.load_url(&url);
                    }
                }
            }
            
            if let Some(events) = &self.events {
                if let Ok(mut events) = events.lock() {
                    events.publish(BrowserEvent::TabUrlChanged { id, url: url.clone() })?;
                }
            }

            // Add to event viewer
            self.event_viewer.add_event(BrowserEvent::TabUrlChanged { id, url });
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_navigation() {
        let mut browser = BrowserEngine::new();
        browser.navigate("https://www.example.com").unwrap();
        if let Some(tab) = browser.tabs.get_active_tab() {
            assert_eq!(tab.url, "https://www.example.com");
        }
    }
}
