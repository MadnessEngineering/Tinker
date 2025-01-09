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
#[command(author, version, about = "A craftsperson's browser", long_about = None)]
struct Args {
    /// URL to load
    #[arg(short, long)]
    url: Option<String>,

    /// Run in headless mode
    #[arg(short = 'H', long)]
    headless: bool,

    /// Event broker URL
    #[arg(short, long)]
    broker_url: Option<String>,

    /// Number of tabs to open
    #[arg(long)]
    tabs: Option<usize>,

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting Tinker Workshop...");

    // Parse command line arguments
    let args = Args::parse();

    // Initialize event system if broker URL is specified
    let events = if let Some(broker_url) = args.broker_url.as_ref() {
        let mut events = EventSystem::new(broker_url, "tinker-browser");
        events.connect()?;
        info!("Connected to event broker at {}", broker_url);
        Some(Arc::new(Mutex::new(events)))
    } else {
        None
    };

    // Create browser instance
    let mut browser = BrowserEngine::new(
        args.headless,
        args.broker_url.as_deref(),
    );

    // Subscribe to relevant topics if events are enabled
    if let Some(ref events) = events {
        if let Ok(mut events) = events.lock() {
            events.subscribe("browser/events")?;
            events.subscribe("browser/commands")?;
            info!("Subscribed to browser event topics");
        }
    }

    // Start recording if enabled
    if args.record {
        if let Some(path) = args.record_path.as_deref() {
            browser.start_recording(path);
            info!("Recording will be saved to {}", path);
        } else {
            return Err("--record-path is required when --record is specified".into());
        }
    }

    // Start replay if enabled
    if let Some(path) = args.replay {
        if !std::path::Path::new(&path).exists() {
            error!("Replay file not found: {}", path);
            return Err("Replay file not found".into());
        }
        browser.load_recording(&path)?;
        if let Some(speed) = args.replay_speed {
            browser.set_replay_speed(speed);
        }
        browser.start_replay();
        info!("Replaying events from {}", path);
    }

    // Create initial tab
    let tab_id = browser.create_tab("about:blank")?;

    // Create additional tabs if requested
    if let Some(num_tabs) = args.tabs {
        info!("Creating {} tabs", num_tabs);
        for i in 1..num_tabs {
            browser.create_tab("about:blank")?;
            info!("Created new tab {}", i);
        }
    }

    // Load URL if provided
    if let Some(url) = args.url {
        browser.navigate(tab_id, &url)?;
        info!("Navigating to: {}", url);
    }

    // Start event loop
    browser.run()?;

    Ok(())
}
