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