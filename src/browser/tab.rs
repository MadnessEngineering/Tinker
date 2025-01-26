use std::sync::atomic::{AtomicU64, Ordering};

use iced::{widget::container, Command, Element};
use url::Url;

use super::message::TabMessage;
use crate::engine::{Content, Resource};

static TAB_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Unique identifier for a tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(u64);

impl TabId {
    fn new() -> Self {
        Self(TAB_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// A browser tab
#[derive(Debug)]
pub struct Tab {
    /// Unique identifier
    id: TabId,
    /// Current URL
    url: String,
    /// Page title
    title: String,
    /// Loading progress
    progress: f32,
    /// Content
    content: Option<Content>,
    /// Navigation history
    history: Vec<String>,
    /// Current position in history
    history_pos: usize,
}

impl Tab {
    pub fn new(url: String) -> (Self, Command<TabMessage>) {
        let tab = Self {
            id: TabId::new(),
            url: url.clone(),
            title: "New Tab".into(),
            progress: 0.0,
            content: None,
            history: vec![url.clone()],
            history_pos: 0,
        };

        let cmd = Command::perform(
            async move {
                // TODO: Load content
                url
            },
            |url| TabMessage::Load(Resource::Url(url)),
        );

        (tab, cmd)
    }

    pub fn id(&self) -> TabId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn navigate(&mut self, url: String) -> Command<TabMessage> {
        self.url = url.clone();
        self.progress = 0.0;
        
        // Update history
        self.history.truncate(self.history_pos + 1);
        self.history.push(url.clone());
        self.history_pos = self.history.len() - 1;

        Command::perform(
            async move {
                // TODO: Load content
                url
            },
            |url| TabMessage::Load(Resource::Url(url)),
        )
    }

    pub fn update(&mut self, message: TabMessage) -> Command<TabMessage> {
        match message {
            TabMessage::Load(resource) => {
                self.progress = 0.0;
                Command::perform(
                    async {
                        // TODO: Load content
                        "Loaded content".to_string()
                    },
                    TabMessage::Loaded,
                )
            }
            TabMessage::Loaded(content) => {
                self.progress = 1.0;
                self.content = Some(Content::Text(content));
                Command::none()
            }
            TabMessage::LoadError(error) => {
                self.progress = 1.0;
                self.content = Some(Content::Text(format!("Error: {}", error)));
                Command::none()
            }
            TabMessage::Progress(progress) => {
                self.progress = progress;
                Command::none()
            }
            TabMessage::Title(title) => {
                self.title = title;
                Command::none()
            }
            TabMessage::Back => {
                if self.history_pos > 0 {
                    self.history_pos -= 1;
                    let url = self.history[self.history_pos].clone();
                    self.navigate(url)
                } else {
                    Command::none()
                }
            }
            TabMessage::Forward => {
                if self.history_pos < self.history.len() - 1 {
                    self.history_pos += 1;
                    let url = self.history[self.history_pos].clone();
                    self.navigate(url)
                } else {
                    Command::none()
                }
            }
            TabMessage::Reload => self.navigate(self.url.clone()),
        }
    }

    pub fn view(&self) -> Element<TabMessage> {
        let content = match &self.content {
            Some(Content::Text(text)) => text.clone(),
            Some(Content::Html(html)) => html.clone(),
            Some(Content::Binary(_)) => "Binary content".into(),
            None => "Loading...".into(),
        };

        container(content).into()
    }
} 