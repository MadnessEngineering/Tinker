//! Browser engine implementation

use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;
use tracing::debug;
use crate::event::{EventSystem, BrowserEvent};
use std::sync::{Arc, Mutex};

mod tabs;
mod event_viewer;

use tabs::TabManager;
use event_viewer::EventViewer;

pub struct BrowserEngine {
    headless: bool,
    events: Option<Arc<Mutex<EventSystem>>>,
    tabs: TabManager,
    event_viewer: EventViewer,
}

impl BrowserEngine {
    pub fn new() -> Self {
        BrowserEngine {
            headless: false,
            events: None,
            tabs: TabManager::new(),
            event_viewer: EventViewer::new(),
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

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Starting browser engine...");

        let event_loop = EventLoop::new();
        let mut window_builder = WindowBuilder::new()
            .with_title("Tinker Workshop")
            .with_inner_size(tao::dpi::LogicalSize::new(1024.0, 768.0));

        if self.headless {
            window_builder = window_builder.with_visible(false);
        }

        let window = window_builder.build(&event_loop)?;

        // Create the main browser window
        if let Some(active_tab) = self.tabs.get_active_tab() {
            let _webview = WebViewBuilder::new(&window)
                .with_url(&active_tab.url)?
                .build()?;
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
        event_loop.run(move |event, _, control_flow| {
            *control_flow = if headless {
                ControlFlow::Exit
            } else {
                ControlFlow::Wait
            };

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
