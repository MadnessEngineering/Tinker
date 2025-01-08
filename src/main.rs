use clap::Parser;
use tracing::{debug, error};
use tracing_subscriber::FmtSubscriber;
use std::sync::{Arc, Mutex};
use crate::event::EventSystem;

mod api;
mod browser;
mod event;
mod templates;

use browser::BrowserEngine;

/// Tinker Browser - A browser built for testing
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to open on startup
    #[arg(short, long)]
    url: Option<String>,

    /// Run in headless mode
    #[arg(long)]
    headless: bool,

    /// MQTT broker URL for events
    #[arg(long, default_value = "localhost")]
    mqtt_broker: String,

    /// Start recording events on startup
    #[arg(long)]
    record: bool,

    /// Path to save the recording (required if --record is used)
    #[arg(long)]
    record_path: Option<String>,

    /// Path to load a recording for replay
    #[arg(long)]
    replay: Option<String>,

    /// Playback speed for replay (default: 1.0)
    #[arg(long, default_value = "1.0")]
    replay_speed: f32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .pretty()
        .init();

    let args = Args::parse();

    // Initialize event system if MQTT broker is specified
    let events = if !args.mqtt_broker.is_empty() {
        let mut events = EventSystem::new(&args.mqtt_broker, "tinker-browser");
        if let Err(e) = events.connect() {
            error!("Failed to connect to MQTT broker: {}", e);
            None
        } else {
            Some(Arc::new(Mutex::new(events)))
        }
    } else {
        None
    };

    // Create and initialize browser
    let mut browser = BrowserEngine::new(args.headless, events);
    
    // Handle recording
    if args.record {
        if let Some(path) = args.record_path.as_deref() {
            browser.start_recording();
            debug!("Recording will be saved to: {}", path);
        } else {
            return Err("--record-path is required when --record is used".into());
        }
    }

    // Handle replay
    if let Some(path) = args.replay.as_deref() {
        browser.load_recording(path)?;
        browser.set_replay_speed(args.replay_speed);
        browser.start_replay();
        debug!("Replaying events from: {} at {}x speed", path, args.replay_speed);
    }

    if let Some(url) = args.url {
        browser.navigate(&url)?;
    }

    // Run the browser
    browser.run()?;

    // Save recording if we were recording
    if args.record {
        if let Some(path) = args.record_path.as_deref() {
            browser.save_recording(path)?;
        }
    }

    Ok(())
}
