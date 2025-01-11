#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub use self::windows::*;
#[cfg(target_os = "macos")]
pub use self::macos::*;

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

pub trait PlatformWebView {
    fn new(window: &impl raw_window_handle::HasWindowHandle) -> anyhow::Result<wry::WebView>;
    fn set_bounds(&self, bounds: wry::Rect);
    fn load_url(&self, url: &str);
    fn evaluate_script(&self, script: &str) -> anyhow::Result<()>;
} 