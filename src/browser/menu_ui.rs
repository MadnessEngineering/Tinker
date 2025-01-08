use wry::{WebView, WebViewBuilder};
use tao::window::Window;
use tracing::debug;
use std::sync::{Arc, Mutex};
use crate::templates::MENU_BAR_HTML;
use serde_json;

pub enum MenuCommand {
    NewTab,
    NewWindow,
    CloseTab,
    CloseWindow,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleDevTools,
    StartRecording,
    StopRecording,
    RunTests,
    ViewTestReport,
    SwitchEngine(String),
    OpenJsConsole,
    OpenPerformanceMonitor,
    OpenDocumentation,
    ReportIssue,
    About,
}

#[derive(Clone)]
pub struct MenuBar {
    container: Arc<Mutex<WebView>>,
    height: u32,
}

impl MenuBar {
    pub fn new(window: &Window, command_handler: impl Fn(MenuCommand) + Send + 'static) -> Result<Self, Box<dyn std::error::Error>> {
        let height = 30;

        let webview = WebViewBuilder::new(window)
            .with_html(MENU_BAR_HTML)
            .with_initialization_script("window.addEventListener('DOMContentLoaded', () => { document.body.style.backgroundColor = '#ffffff'; });")
            .with_ipc_handler(move |window, message| {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&message) {
                    if let Some(cmd_type) = data.get("type").and_then(|v| v.as_str()) {
                        if cmd_type == "MenuCommand" {
                            if let Some(cmd) = data.get("command").and_then(|v| v.as_str()) {
                                let command = match cmd {
                                    "NewTab" => Some(MenuCommand::NewTab),
                                    "NewWindow" => Some(MenuCommand::NewWindow),
                                    "CloseTab" => Some(MenuCommand::CloseTab),
                                    "CloseWindow" => Some(MenuCommand::CloseWindow),
                                    "ZoomIn" => Some(MenuCommand::ZoomIn),
                                    "ZoomOut" => Some(MenuCommand::ZoomOut),
                                    "ZoomReset" => Some(MenuCommand::ZoomReset),
                                    "ToggleDevTools" => Some(MenuCommand::ToggleDevTools),
                                    "StartRecording" => Some(MenuCommand::StartRecording),
                                    "StopRecording" => Some(MenuCommand::StopRecording),
                                    "RunTests" => Some(MenuCommand::RunTests),
                                    "ViewTestReport" => Some(MenuCommand::ViewTestReport),
                                    "SwitchEngine" => {
                                        data.get("param")
                                            .and_then(|v| v.as_str())
                                            .map(|engine| MenuCommand::SwitchEngine(engine.to_string()))
                                    },
                                    "OpenJsConsole" => Some(MenuCommand::OpenJsConsole),
                                    "OpenPerformanceMonitor" => Some(MenuCommand::OpenPerformanceMonitor),
                                    "OpenDocumentation" => Some(MenuCommand::OpenDocumentation),
                                    "ReportIssue" => Some(MenuCommand::ReportIssue),
                                    "About" => Some(MenuCommand::About),
                                    _ => None,
                                };

                                if let Some(command) = command {
                                    command_handler(command);
                                }
                            }
                        }
                    }
                }
            })
            .build()?;

        Ok(MenuBar {
            container: Arc::new(Mutex::new(webview)),
            height,
        })
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
} 
