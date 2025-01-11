use anyhow::Result;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
pub use self::windows::{WindowsManager, WindowsWebView, WindowsConfig, WindowsTheme};

#[cfg(target_os = "macos")]
pub use self::macos::{MacosManager, MacosWebView, MacosConfig, MacosTheme};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Windows,
    MacOS,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else {
            panic!("Unsupported platform")
        }
    }
}

pub trait PlatformManager {
    fn new(config: impl Into<String>) -> Result<Self>
    where
        Self: Sized;
    fn run(&self) -> Result<()>;
}

pub trait PlatformWebView {
    fn new<T>(window: &T) -> Result<Self>
    where
        Self: Sized,
        T: raw_window_handle::HasRawWindowHandle;
    fn navigate(&self, url: &str) -> Result<()>;
    fn evaluate_script(&self, script: &str) -> Result<()>;
    fn resize(&self, width: i32, height: i32);
} 