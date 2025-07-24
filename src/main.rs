use clap::Parser;
use tracing::{debug, error, info};
use std::{sync::{Arc, Mutex}, env};

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

    /// Debug mode
    #[arg(long)]
    debug: bool,

    /// Enable API server
    #[arg(long)]
    api: bool,

    /// API server port
    #[arg(long, default_value = "3003")]
    api_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set default debug mode if not explicitly set in environment
    if env::var("DEBUG").is_err() {
        env::set_var("DEBUG", "TRUE");
    }

    // Initialize logging with more detailed format
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("tinker=debug".parse()?)
            .add_directive("wry=debug".parse()?))
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .init();

    info!("Starting Tinker Workshop...");

    // Parse command line arguments
    let args = Args::parse();

    // Create broadcast channels for API server if enabled
    let (api_event_tx, api_event_rx) = if args.api {
        let (tx, rx) = tokio::sync::broadcast::channel(1000);
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    let (api_command_tx, api_command_rx) = if args.api {
        let (tx, rx) = tokio::sync::broadcast::channel(100);
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    // Initialize event system if broker URL is specified
    let events = if let Some(broker_url) = args.broker_url.as_ref() {
        let mut events = EventSystem::new(broker_url, "tinker-browser");
        
        // Set up broadcast channels for API integration
        if let (Some(event_tx), Some(command_rx)) = (api_event_tx.clone(), api_command_rx) {
            events.set_broadcast_channels(event_tx, command_rx);
        }
        
        Some(Arc::new(Mutex::new(events)))
    } else {
        None
    };

    // Create browser instance with default URL if none provided
    let mut browser = BrowserEngine::new(
        args.headless,
        events.clone(),
        args.url.or_else(|| Some("about:blank".to_string())),
    );

    // Connect to event system after browser is initialized
    if let Some(ref events) = events {
        if let Ok(mut events) = events.lock() {
            if let Err(e) = events.connect() {
                error!("Failed to connect to event broker: {}. Continuing without event system.", e);
            } else {
                // Subscribe to all browser events using wildcard
                if let Err(e) = events.subscribe("browser/#") {
                    error!("Failed to subscribe to events: {}. Continuing without event subscription.", e);
                } else {
                    info!("Connected to event broker and subscribed to events");
                }
            }
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
            let _ = browser.set_replay_speed(speed);
        }
        let _ = browser.start_replay();
        info!("Replaying events from {}", path);
    }

    // Start API server if enabled
    if args.api {
        if let (Some(command_tx), Some(event_rx)) = (api_command_tx, api_event_rx) {
            info!("🚀 Starting API server on port {}", args.api_port);
            tokio::spawn(async move {
                if let Err(e) = api::start_api_server(command_tx, event_rx).await {
                    error!("API server error: {}", e);
                }
            });
        }
    }

    // Start event loop
    info!("Starting browser engine...");
    browser.run()?;

    Ok(())
}
