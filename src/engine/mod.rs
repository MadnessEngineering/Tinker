use std::path::PathBuf;

/// Content resource
#[derive(Debug, Clone)]
pub enum Resource {
    /// URL resource
    Url(String),
    /// File resource
    File(PathBuf),
    /// Raw data
    Data(Vec<u8>),
}

/// Content type
#[derive(Debug, Clone)]
pub enum Content {
    /// Text content
    Text(String),
    /// HTML content
    Html(String),
    /// Binary content
    Binary(Vec<u8>),
}

/// Content processor
pub trait ContentProcessor {
    /// Check if this processor can handle the content type
    fn can_handle(&self, content_type: &str) -> bool;
    
    /// Process the content
    fn process(&self, content: &[u8]) -> Result<Content, String>;
}

/// Resource loader
#[async_trait::async_trait]
pub trait ResourceLoader {
    /// Load a resource
    async fn load(&self, resource: Resource) -> Result<Content, String>;
    
    /// Cache a resource
    fn cache(&mut self, resource: &Resource, content: &Content);
}

/// HTML processor using html5ever
pub struct HtmlProcessor {
    parser: html5ever::Parser,
}

impl HtmlProcessor {
    pub fn new() -> Self {
        Self {
            parser: html5ever::Parser::new(),
        }
    }
}

impl ContentProcessor for HtmlProcessor {
    fn can_handle(&self, content_type: &str) -> bool {
        content_type.contains("text/html")
    }
    
    fn process(&self, content: &[u8]) -> Result<Content, String> {
        // TODO: Parse HTML using html5ever
        Ok(Content::Html(String::from_utf8_lossy(content).into()))
    }
}

/// Default resource loader
pub struct DefaultLoader {
    processors: Vec<Box<dyn ContentProcessor>>,
}

impl DefaultLoader {
    pub fn new() -> Self {
        let mut processors: Vec<Box<dyn ContentProcessor>> = Vec::new();
        processors.push(Box::new(HtmlProcessor::new()));
        Self { processors }
    }
}

#[async_trait::async_trait]
impl ResourceLoader for DefaultLoader {
    async fn load(&self, resource: Resource) -> Result<Content, String> {
        match resource {
            Resource::Url(url) => {
                // TODO: Fetch URL content
                Ok(Content::Text(format!("Content from {}", url)))
            }
            Resource::File(path) => {
                // TODO: Load file content
                Ok(Content::Text(format!("Content from {:?}", path)))
            }
            Resource::Data(data) => {
                // TODO: Process data with appropriate processor
                Ok(Content::Binary(data))
            }
        }
    }
    
    fn cache(&mut self, _resource: &Resource, _content: &Content) {
        // TODO: Implement caching
    }
} 