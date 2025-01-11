mod webview;
mod window;

pub use webview::WindowsWebView;
pub use window::WindowsManager;

use windows::Win32::UI::{
    WindowsAndMessaging as win32_window,
    Controls as win32_controls,
    Shell as win32_shell,
    HiDpi as win32_dpi,
};

#[derive(Debug)]
pub struct WindowsConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub decorations: bool,
    pub transparent: bool,
    pub dpi_aware: bool,
    pub theme: WindowsTheme,
}

#[derive(Debug, Clone, Copy)]
pub enum WindowsTheme {
    Light,
    Dark,
    System,
}

impl Default for WindowsConfig {
    fn default() -> Self {
        Self {
            title: "Tinker".to_string(),
            width: 1024,
            height: 768,
            decorations: true,
            transparent: false,
            dpi_aware: true,
            theme: WindowsTheme::System,
        }
    }
} 