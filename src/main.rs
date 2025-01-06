use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use crate::browser::BrowserEngine;

mod browser;
mod event;
mod api;
mod cli;

#[derive(Parser)]
#[command(name = "tinker")]
#[command(author = "The Tinker Workshop")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A craftsperson's browser for testing and automation", long_about = None)]
struct Cli {
    /// Start in headless mode
    #[arg(long)]
    headless: bool,

    /// MQTT broker URL
    #[arg(long, default_value = "mqtt://localhost:1883")]
    mqtt_url: String,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,

    /// Initial URL to navigate to
    #[arg(long)]
    url: Option<String>,

    /// Number of tabs to open at startup
    #[arg(long, default_value = "1")]
    tabs: usize,
}

#[tokio::main]
async fn main() -> WryResult<()> {
    let cli = Cli::parse();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(if cli.debug { Level::DEBUG } else { Level::INFO })
        .init();

    info!("Starting Tinker Workshop...");
    info!("MQTT Broker: {}", cli.mqtt_url);
    info!("Headless Mode: {}", cli.headless);

    // Forge our browser engine
    let mut browser = BrowserEngine::forge(cli.headless)?;

    // Open initial tabs if requested
    for _ in 1..cli.tabs {
        browser.new_tab(None)?;
    }

    // Navigate to initial URL if provided
    if let Some(url) = cli.url {
        browser.navigate(&url)?;
    }

    // Keep the main thread alive while the WebView is running
    browser.webview.run();

    Ok(())
} 
