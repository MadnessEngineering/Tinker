//! Tinker browser library

pub mod browser;
pub mod platform;
pub mod js_engine;

// Re-exports for convenient access
pub use browser::BrowserEngine;
pub use browser::event::{BrowserEvent, EventSystem};
pub use platform::{Platform, PlatformManager, PlatformWebView};
pub use js_engine::{JsEngine, JsEngineType, JsEngineBuilder};

// Feature flags for conditional compilation
#[cfg(feature = "v8")]
pub use js_engine::v8::V8Engine;

#[cfg(feature = "javascriptcore")]
pub use js_engine::javascriptcore::JavaScriptCoreEngine;

#[cfg(feature = "spidermonkey")]
pub use js_engine::spidermonkey::SpiderMonkeyEngine;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_HASH: &str = env!("GIT_HASH", "unknown");

// Platform-specific exports
#[cfg(target_os = "windows")]
pub use platform::windows::{WindowsManager, WindowsWebView, WindowsConfig, WindowsTheme};

#[cfg(target_os = "macos")]
pub use platform::macos::{MacosManager, MacosWebView, MacosConfig, MacosTheme};
