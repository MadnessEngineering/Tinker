use clap::Parser;
use tracing::{debug, error, info};
use std::sync::{Arc, Mutex};

mod api;
mod browser;
mod event;
mod templates;

use crate::{
    browser::BrowserEngine,
    event::EventSystem,
};

#[derive(Parser, Debug)]
struct Args {
    /// URL to load
    #[arg(long)]
    url: Option<String>,

    /// Run in headless mode
    #[arg(long)]
    headless: bool,

    /// MQTT broker URL
    #[arg(long)]
    mqtt_broker: Option<String>,

    /// Enable event recording
    #[arg(long)]
    record: bool,

    /// Path to save recorded events
    #[arg(long)]
    record_path: Option<String>,

    /// Path to replay events from
    #[arg(long)]
    replay: Option<String>,

    /// Speed multiplier for replay
    #[arg(long)]
    replay_speed: Option<f32>,

    /// Number of tabs to open
    #[arg(long)]
    tabs: Option<usize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting Tinker Workshop");

    // Parse command line arguments
    let args = Args::parse();

    // Create event system if needed
    let events = if args.record || args.replay.is_some() || args.mqtt_broker.is_some() {
        let broker_url = args.mqtt_broker.unwrap_or_else(|| "localhost".to_string());
        Some(Arc::new(Mutex::new(EventSystem::new(&broker_url, "tinker-browser"))))
    } else {
        None
    };

    // Create browser engine
    let mut browser = BrowserEngine::new(args.headless, events);

    // Create initial tabs
    let num_tabs = args.tabs.unwrap_or(1);
    for _ in 0..num_tabs {
        browser.create_tab("about:blank")?;
    }

    // Load initial URL if provided
    if let Some(url) = args.url {
        browser.navigate(&url)?;
    }

    // Run the browser
    browser.run()?;

    Ok(())
}
