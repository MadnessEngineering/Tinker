pub enum BrowserMessage {
    CreateTab { id: u32, url: Option<String> },
    SwitchTab { id: u32 },
    CloseTab { id: u32 },
    UpdateUrl { id: u32, url: String },
    UpdateTitle { id: u32, title: String },
}

impl BrowserMessage {
    pub fn handle(&self, window: &BrowserWindow) -> Result<(), Error> {
        match self {
            BrowserMessage::CreateTab { id, url } => {
                window.create_tab(*id, url.clone())?;
            }
            BrowserMessage::SwitchTab { id } => {
                window.switch_to_tab(*id)?;
            }
            // ... other message handlers
        }
        Ok(())
    }
} 