//! Browser engine implementation

use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    platform::macos::WindowExtMacOS,
};
use wry::WebViewBuilder;
use tracing::debug;

pub struct BrowserEngine {
    url: String,
}

impl BrowserEngine {
    pub fn new() -> Self {
        BrowserEngine {
            url: String::from("about:blank"),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Starting browser engine...");

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Tinker Workshop")
            .with_inner_size(tao::dpi::LogicalSize::new(1024.0, 768.0))
            .build(&event_loop)?;

        let _webview = WebViewBuilder::new(&window)
            .with_url("about:blank")?
            .build()?;

        debug!("Running event loop...");

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => debug!("Browser window initialized"),
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
        self.url = url.to_string();
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
