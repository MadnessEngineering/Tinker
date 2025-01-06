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

pub struct BrowserEngine {
    url: String,
    headless: bool,
    events: Option<Arc<Mutex<EventSystem>>>,
}

impl BrowserEngine {
    pub fn new() -> Self {
        BrowserEngine {
            url: String::from("about:blank"),
            headless: false,
            events: None,
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

        let _webview = WebViewBuilder::new(&window)
            .with_url(&self.url)?
            .build()?;

        debug!("Running event loop...");

        // Emit initial events
        if let Some(events) = &self.events {
            if let Ok(mut events) = events.lock() {
                events.publish(BrowserEvent::PageLoaded {
                    url: self.url.clone(),
                })?;
            }
        }

        let headless = self.headless;
        let events = self.events.clone();  // Clone the Arc for the closure
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
        self.url = url.to_string();
        
        if let Some(events) = &self.events {
            if let Ok(mut events) = events.lock() {
                events.publish(BrowserEvent::Navigation {
                    url: url.to_string(),
                })?;
            }
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
        assert_eq!(browser.url, "https://www.example.com");
    }
}
