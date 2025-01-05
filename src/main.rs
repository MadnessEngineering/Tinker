use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod browser;
mod event;
mod api;

#[tokio::main]
async fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .init();

    info!("Starting TestBrowser...");

    // TODO: Initialize components
} 
