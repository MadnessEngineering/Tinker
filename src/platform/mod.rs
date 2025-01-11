use anyhow::Result;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

// Re-export platform-specific types
#[cfg(target_os = "windows")]
pub use self::windows::WindowsManager;

#[cfg(all(target_os = "windows", feature = "webview"))]
pub use self::windows::WindowsWebView;

#[cfg(target_os = "windows")]
pub use self::windows::{WindowsConfig, WindowsTheme};

#[cfg(target_os = "macos")]
pub use self::macos::MacosManager;

#[cfg(all(target_os = "macos", feature = "webview"))]
pub use self::macos::MacosWebView;

#[cfg(target_os = "macos")]
pub use self::macos::{MacosConfig, MacosTheme};

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

#[cfg(feature = "webview")]
pub trait PlatformWebView {
    fn new<T>(window: &T) -> Result<Self>
    where
        Self: Sized,
        T: raw_window_handle::HasRawWindowHandle;
    fn navigate(&self, url: &str) -> Result<()>;
    fn evaluate_script(&self, script: &str) -> Result<()>;
    fn resize(&self, width: i32, height: i32);
    
    fn set_theme(&self, theme: &str) -> Result<()>;
    fn set_title(&self, title: &str) -> Result<()>;
    fn set_visible(&self, visible: bool) -> Result<()>;
    fn get_position(&self) -> Result<(i32, i32)>;
    fn set_position(&self, x: i32, y: i32) -> Result<()>;
    fn handle_platform_message(&self, message: &str) -> Result<()>;
    fn is_visible(&self) -> bool;
    fn get_parent_window(&self) -> Option<&dyn raw_window_handle::HasRawWindowHandle>;
} 