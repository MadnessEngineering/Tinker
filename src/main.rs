use clap::Parser;
use tracing::{debug, error, info};
use std::{sync::{Arc, Mutex}, env};

mod api;
mod browser;
mod event;
mod mcp;
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

    /// Enable MCP server (Model Context Protocol over stdio)
    #[arg(long)]
    mcp: bool,
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

    // Create broadcast channels for API server or MCP server if enabled
    let (api_event_tx, api_event_rx) = if args.api || args.mcp {
        let (tx, rx) = tokio::sync::broadcast::channel(1000);
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    let (api_command_tx, api_command_rx) = if args.api || args.mcp {
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

    // Get initial URL (default to about:blank if not provided)
    let initial_url = args.url.or_else(|| Some("about:blank".to_string()));

    // Create browser instance with default URL if none provided
    let mut browser = BrowserEngine::new(
        args.headless,
        events.clone(),
        initial_url.clone(),
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
            // Generate recording name from path
            let name = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("recording")
                .to_string();

            // Use the initial URL
            let start_url = initial_url.clone().unwrap_or_else(|| "about:blank".to_string());

            if let Err(e) = browser.start_recording(name, start_url) {
                error!("Failed to start recording: {}", e);
            } else {
                info!("Recording started, will be saved to {}", path);
            }
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

    // Start API server and/or MCP server if enabled
    if let (Some(command_tx), Some(mut event_rx)) = (api_command_tx, api_event_rx) {
        // Start API server if enabled
        if args.api {
            let command_tx_clone = command_tx.clone();
            let event_rx_clone = event_rx.resubscribe();
            info!("🚀 Starting API server on port {}", args.api_port);
            tokio::spawn(async move {
                if let Err(e) = api::start_api_server(command_tx_clone, event_rx_clone).await {
                    error!("API server error: {}", e);
                }
            });
        }

        // Start MCP server if enabled
        if args.mcp {
            let command_tx_clone = command_tx.clone();
            let event_rx_clone = event_rx.resubscribe();
            info!("🚀 Starting MCP server on stdio");
            info!("📡 MCP server ready for JSON-RPC protocol messages");

            // MCP server must run on a separate thread since it blocks on stdin
            std::thread::spawn(move || {
                let mut mcp_server = mcp::McpServer::new(command_tx_clone, event_rx_clone);
                if let Err(e) = mcp_server.run() {
                    error!("MCP server error: {}", e);
                }
            });
        }
    }

    info!("✅ Tinker Workshop fully operational!");
    info!("🔗 API Server: http://127.0.0.1:3003");
    info!("🔗 WebSocket: ws://127.0.0.1:3003/ws");
    info!("🔗 Health Check: curl http://127.0.0.1:3003/health");
    info!("🌐 Starting browser engine on main thread...");

    // Browser engine must run on main thread (platform requirement)
    // API server runs in background tokio tasks - this is correct!
    browser.run()?;

    info!("👋 Tinker Workshop shutting down...");
    Ok(())
}
