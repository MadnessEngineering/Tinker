//! Tinker browser library

pub mod browser;
pub mod platform;

pub use browser::BrowserEngine;
pub use browser::event::{BrowserEvent, EventSystem};
pub use platform::{Platform, PlatformManager, PlatformWebView};
