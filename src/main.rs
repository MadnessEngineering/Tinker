use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;
use clap::Parser;

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .try_init();

    if let Err(_) = subscriber {
        debug!("Logging already initialized");
    }

    let args = Args::parse();
    debug!("Starting Tinker Workshop");

    let mut browser = BrowserEngine::new();
    browser.set_headless(args.headless);
    
    if let Some(url) = args.url {
        browser.navigate(&url)?;
    }

    if let Some(tabs) = args.tabs {
        for _ in 1..tabs {
            debug!("Created new tab");
        }
    }

    browser.run()?;

    Ok(())
}
