#[derive(Debug, Clone)]
pub struct WindowsConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub decorations: bool,
    pub dpi_aware: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum WindowsTheme {
    Light,
    Dark,
    System,
}

impl WindowsConfig {
    pub fn get_dark_theme_style(&self) -> u32 {
        use windows::Win32::UI::WindowsAndMessaging::{
            WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_CAPTION, WS_SYSMENU, WS_MINIMIZEBOX, WS_MAXIMIZEBOX,
        };

        if self.decorations {
            (WS_OVERLAPPEDWINDOW | WS_VISIBLE).0
        } else {
            (WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_VISIBLE).0
        }
    }

    pub fn get_light_theme_style(&self) -> u32 {
        self.get_dark_theme_style() // Same as dark for now, can be customized later
    }

    pub fn get_default_style(&self) -> u32 {
        use windows::Win32::UI::WindowsAndMessaging::WS_OVERLAPPEDWINDOW;
        WS_OVERLAPPEDWINDOW.0
    }
}

impl Default for WindowsConfig {
    fn default() -> Self {
        Self {
            title: "Tinker".to_string(),
            width: 1024,
            height: 768,
            decorations: true,
            dpi_aware: true,
        }
    }
} 