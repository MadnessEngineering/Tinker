use anyhow::Result;
use clap::Parser;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

mod browser;
mod platform;

use browser::Browser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to load
    #[arg(short, long)]
    url: Option<String>,

    /// Run in headless mode
    #[arg(short = 'H', long)]
    headless: bool,

    /// Debug mode
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let filter = if args.debug {
        "debug"
    } else {
        "info"
    };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .init();

    info!("Starting Tinker browser...");
    debug!("Debug mode: {}", args.debug);

    // Create and run browser
    let mut browser = Browser::new("Tinker")?;
    if let Some(url) = args.url {
        browser = browser.with_url(url);
    }
    browser.run()?;

    Ok(())
}
