use super::TabId;
use crate::engine::Resource;

#[derive(Debug, Clone)]
pub enum Message {
    /// Message for a specific tab
    TabMessage(TabId, TabMessage),
    /// Create a new tab
    NewTab,
    /// Close a tab
    CloseTab(TabId),
    /// Activate a tab
    ActivateTab(TabId),
    /// Navigate to URL
    Navigate(String),
}

#[derive(Debug, Clone)]
pub enum TabMessage {
    /// Load content
    Load(Resource),
    /// Content loaded
    Loaded(String),
    /// Loading failed
    LoadError(String),
    /// Update progress
    Progress(f32),
    /// Update title
    Title(String),
    /// Go back
    Back,
    /// Go forward 
    Forward,
    /// Reload page
    Reload,
} 