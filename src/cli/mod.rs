use std::path::PathBuf;

#[cfg(feature = "cli")]
use clap::Parser;

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
#[cfg_attr(feature = "cli", clap(version, about = "Tinker Browser - A testing-focused browser"))]
pub struct Args {
    #[cfg_attr(feature = "cli", clap(long, help = "Run in headless mode"))]
    pub headless: bool,

    #[cfg_attr(feature = "cli", clap(long, help = "Path to configuration file"))]
    pub config: Option<PathBuf>,
}

impl Args {
    pub fn parse() -> Self {
        #[cfg(feature = "cli")]
        {
            Args::parse()
        }

        #[cfg(not(feature = "cli"))]
        {
            Self {
                headless: false,
                config: None,
            }
        }
    }
} 
