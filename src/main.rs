use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;
use clap::Parser;
use tokio;

mod browser;
mod api;
mod event;

use browser::BrowserEngine;

#[derive(Parser, Debug)]
#[command(
    name = "tinker",
    author = "Cursor",
    version,
    about = "A craftsperson's browser",
    long_about = None
)]
struct Args {
    /// Run in headless mode without a visible window
    #[arg(long)]
    headless: bool,

    /// URL to navigate to on startup
    #[arg(long)]
    url: Option<String>,

    /// Number of tabs to open on startup
    #[arg(long)]
    tabs: Option<usize>,

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
        .with_max_level(Level::DEBUG)
        .try_init();

    if let Err(_) = subscriber {
        debug!("Logging already initialized");
    }

    let args = Args::parse();
    debug!("Starting Tinker Workshop");

    // Start API server in a separate task
    tokio::spawn(async {
        if let Err(e) = api::start_api_server().await {
            debug!("API server error: {}", e);
        }
    });

    let mut browser = BrowserEngine::new();
    browser.set_headless(args.headless);
    
    // Initialize event system
    if let Err(e) = browser.init_events(&args.mqtt_broker) {
        debug!("Failed to initialize event system: {}", e);
    }
    
    // Handle recording
    let record_path = args.record_path.as_deref();
    if args.record {
        if let Some(path) = record_path {
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

    if let Some(tabs) = args.tabs {
        for _ in 1..tabs {
            debug!("Created new tab");
        }
    }

    browser.run()?;

    // Save recording if we were recording
    if args.record {
        if let Some(path) = record_path {
            browser.save_recording(path)?;
        }
    }

    Ok(())
}
