use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(if cli.debug { Level::DEBUG } else { Level::INFO })
        .init();

    info!("Starting Tinker Workshop...");
    info!("MQTT Broker: {}", cli.mqtt_url);
    info!("Headless Mode: {}", cli.headless);
} 
