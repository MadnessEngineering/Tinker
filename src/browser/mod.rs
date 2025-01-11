//! Browser engine implementation

pub mod tabs;
pub mod tab_ui;

use std::sync::{Arc, Mutex};
use anyhow::Result;
use tracing::{debug, error, info};
use tao::event_loop::{ControlFlow, Event, EventLoop};

use crate::event::EventSystem;
use self::tabs::TabManager;
use self::tab_ui::{TabBar, TabCommand};

#[cfg(target_os = "windows")]
use crate::platform::windows::{WindowsManager, WindowsConfig, WindowsTheme};

pub struct BrowserEngine {
    #[cfg(not(target_os = "windows"))]
    window: Window,
    #[cfg(target_os = "windows")]
    window_manager: WindowsManager,
    tab_manager: TabManager,
    events: Option<Arc<Mutex<EventSystem>>>,
}

impl BrowserEngine {
    pub fn new(
        headless: bool,
        events: Option<Arc<Mutex<EventSystem>>>,
        initial_url: Option<String>,
    ) -> Result<Self> {
        #[cfg(target_os = "windows")]
        let window_manager = {
            let config = WindowsConfig {
                decorations: !headless,
                transparent: true,
                dpi_aware: true,
                theme: WindowsTheme::System,
                ..Default::default()
            };
            WindowsManager::new(config)?
        };

        #[cfg(not(target_os = "windows"))]
        let window = {
            let event_loop = EventLoop::new();
            WindowBuilder::new()
                .with_title("Tinker")
                .with_decorations(!headless)
                .with_transparent(true)
                .build(&event_loop)?
        };

        let tab_manager = TabManager::new();

        Ok(Self {
            #[cfg(target_os = "windows")]
            window_manager,
            #[cfg(not(target_os = "windows"))]
            window,
            tab_manager,
            events,
        })
    }

    pub fn create_tab(&mut self, url: &str) -> Result<u32> {
        let id = self.tab_manager.create_tab(url)?;
        
        #[cfg(target_os = "windows")]
        self.window_manager.add_tab(url)?;

        Ok(id)
    }

    pub fn close_tab(&mut self, id: u32) -> Result<()> {
        self.tab_manager.close_tab(id)?;
        
        #[cfg(target_os = "windows")]
        if let Some(index) = self.tab_manager.get_tab_index(id) {
            self.window_manager.remove_tab(index)?;
        }

        Ok(())
    }

    pub fn set_active_tab(&mut self, id: u32) -> Result<()> {
        self.tab_manager.set_active_tab(id)?;
        
        #[cfg(target_os = "windows")]
        if let Some(index) = self.tab_manager.get_tab_index(id) {
            self.window_manager.set_active_tab(index)?;
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            self.window_manager.run()?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            let event_loop = EventLoop::new();
            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Wait;

                match event {
                    Event::WindowEvent { event, .. } => {
                        // Handle window events
                    }
                    _ => (),
                }
            });
        }

        Ok(())
    }
}
