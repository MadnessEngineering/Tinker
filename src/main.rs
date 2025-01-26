use tinker::{Browser, Result};
use tracing::info;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting Tinker browser");

    // Create browser instance
    let (browser, event_loop) = Browser::new()?;

    // Run the event loop
    event_loop.run(move |event, _, control_flow| {
        browser.handle_event(event, control_flow);
    });
}
