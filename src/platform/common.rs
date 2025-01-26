use std::sync::Arc;
use tao::window::Window;
use wry::WebView;
use super::PlatformError;

/// Common window configuration across platforms
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub always_on_top: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("Tinker Browser"),
            width: 1024,
            height: 768,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
        }
    }
}

/// Common WebView configuration across platforms
#[derive(Debug, Clone)]
pub struct WebViewConfig {
    pub url: String,
    pub transparent: bool,
    pub initialization_scripts: Vec<String>,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            url: String::from("about:blank"),
            transparent: false,
            initialization_scripts: Vec::new(),
        }
    }
}

/// Platform-agnostic window handle
pub type WindowHandle = Arc<Window>;

/// Platform-agnostic WebView handle
pub type WebViewHandle = Arc<WebView>;

/// Utility functions for platform-specific operations
pub mod utils {
    use super::*;
    use std::env::consts::OS;

    pub fn get_platform() -> &'static str {
        OS
    }

    pub fn is_macos() -> bool {
        OS == "macos"
    }

    pub fn is_windows() -> bool {
        OS == "windows"
    }

    pub fn is_linux() -> bool {
        OS == "linux"
    }

    pub fn get_default_window_config() -> WindowConfig {
        WindowConfig::default()
    }

    pub fn get_default_webview_config() -> WebViewConfig {
        WebViewConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_config_default() {
        let config = WindowConfig::default();
        assert_eq!(config.title, "Tinker Browser");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert!(config.resizable);
        assert!(config.decorations);
        assert!(!config.transparent);
        assert!(!config.always_on_top);
    }

    #[test]
    fn test_webview_config_default() {
        let config = WebViewConfig::default();
        assert_eq!(config.url, "about:blank");
        assert!(!config.transparent);
        assert!(config.initialization_scripts.is_empty());
    }

    #[test]
    fn test_platform_detection() {
        let platform = utils::get_platform();
        assert!(!platform.is_empty());
        
        // At least one of these should be true
        assert!(utils::is_macos() || utils::is_windows() || utils::is_linux());
    }
} 
