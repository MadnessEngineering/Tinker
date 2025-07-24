pub mod common;

use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum PlatformError {
    WindowCreationFailed(String),
    WebViewCreationFailed(String),
    InvalidHandle(String),
    Other(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WindowCreationFailed(msg) => write!(f, "Window creation failed: {}", msg),
            Self::WebViewCreationFailed(msg) => write!(f, "WebView creation failed: {}", msg),
            Self::InvalidHandle(msg) => write!(f, "Invalid handle: {}", msg),
            Self::Other(msg) => write!(f, "Platform error: {}", msg),
        }
    }
}

impl StdError for PlatformError {}

pub type Result<T> = std::result::Result<T, PlatformError>;

pub use common::{WindowConfig, WebViewConfig, WindowHandle, WebViewHandle};

// #[cfg(target_os = "windows")]
// pub use windows::{PlatformWindow, PlatformWebView};

// #[cfg(target_os = "macos")]
// pub use macos::{PlatformWindow, PlatformWebView};

// #[cfg(target_os = "linux")]
// pub use linux::{PlatformWindow, PlatformWebView};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowTheme {
    Light,
    Dark,
    System,
}

// /// Platform-specific window creation options
// pub trait PlatformWindow {
//     fn create_window(&self) -> Result<tao::window::Window, Box<dyn std::error::Error>>;
//     fn set_theme(&self, theme: WindowTheme);
//     fn show(&self);
//     fn hide(&self);
// }

// /// Platform-specific WebView creation options
// pub trait PlatformWebView {
//     fn create_webview(&self) -> Result<wry::WebView, Box<dyn std::error::Error>>;
//     fn set_visibility(&self, visible: bool);
//     fn update_bounds(&self);
// }
