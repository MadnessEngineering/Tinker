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

    let args = Args::parse();
    info!("Starting Tinker Workshop");

    // Initialize event system if MQTT broker is specified
    let events = if let Some(broker_url) = args.mqtt_broker {
        let mut events = EventSystem::new(&broker_url, "tinker-browser");
        let _ = events.connect();
        Some(Arc::new(Mutex::new(events)))
    } else {
        None
    };

    // Create browser engine
    let mut browser = BrowserEngine::new(args.headless, events);

    // Create initial tabs
    if let Some(num_tabs) = args.tabs {
        info!("Creating {} tabs", num_tabs);
        for i in 0..num_tabs {
            browser.create_tab("about:blank")?;
            info!("Created new tab {}", i);
        }
    }

    // Load URL if specified
    if let Some(url) = args.url {
        browser.navigate(&url)?;
    }

    // Start recording if enabled
    if args.record {
        if let Some(path) = args.record_path {
            browser.start_recording();
            info!("Recording will be saved to {}", path);
        } else {
            error!("--record-path is required when using --record");
            return Err("--record-path is required".into());
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
        info!("Started replay from {}", path);
    }

    // Run the browser
    browser.run()?;

    Ok(())
}
