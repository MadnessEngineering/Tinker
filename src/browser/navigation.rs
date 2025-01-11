use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use url::Url;
use crate::browser::error::{NavigationError, BrowserResult};

/// Represents a parsed and validated URL with additional metadata
#[derive(Debug, Clone)]
pub struct ParsedUrl {
    url: Url,
    is_secure: bool,
    is_search: bool,
    original_input: String,
}

impl ParsedUrl {
    /// Create a new ParsedUrl from a string input
    pub fn new(input: &str) -> BrowserResult<Self> {
        // First, try to parse as-is
        let url = match Url::parse(input) {
            Ok(url) => url,
            Err(_) => {
                // If that fails, try adding https://
                if !input.starts_with("http://") && !input.starts_with("https://") {
                    match Url::parse(&format!("https://{}", input)) {
                        Ok(url) => url,
                        Err(_) => {
                            // If still fails, treat as search query
                            let search_url = format!(
                                "https://www.google.com/search?q={}",
                                urlencoding::encode(input)
                            );
                            Url::parse(&search_url)
                                .map_err(|e| NavigationError::InvalidUrl(e.to_string()))?
                        }
                    }
                } else {
                    return Err(NavigationError::InvalidUrl(format!("Invalid URL: {}", input)).into());
                }
            }
        };

        Ok(Self {
            is_secure: url.scheme() == "https",
            is_search: url.host_str() == Some("www.google.com") && url.path() == "/search",
            original_input: input.to_string(),
            url,
        })
    }

    /// Get the final URL
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the original input
    pub fn original_input(&self) -> &str {
        &self.original_input
    }

    /// Check if the URL is secure (https)
    pub fn is_secure(&self) -> bool {
        self.is_secure
    }

    /// Check if this is a search query
    pub fn is_search(&self) -> bool {
        self.is_search
    }

    /// Get the display URL (with appropriate formatting)
    pub fn display_url(&self) -> String {
        if self.is_search {
            format!("🔍 {}", self.original_input)
        } else {
            self.url.to_string()
        }
    }
}

/// Manages URL parsing and validation
#[derive(Clone)]
pub struct UrlManager {
    search_engine: Arc<String>,
}

impl UrlManager {
    pub fn new() -> Self {
        Self {
            search_engine: Arc::new("https://www.google.com/search?q=".to_string()),
        }
    }

    /// Set a custom search engine URL
    pub fn set_search_engine(&mut self, url: String) -> BrowserResult<()> {
        // Validate the search engine URL
        if !url.contains("{}") {
            return Err(NavigationError::InvalidUrl(
                "Search engine URL must contain {} placeholder for query".to_string()
            ).into());
        }

        // Test the URL with a dummy query
        let test_url = url.replace("{}", "test");
        Url::parse(&test_url)
            .map_err(|e| NavigationError::InvalidUrl(format!("Invalid search engine URL: {}", e)))?;

        self.search_engine = Arc::new(url);
        Ok(())
    }

    /// Parse and validate a URL or search query
    pub fn parse_input(&self, input: &str) -> BrowserResult<ParsedUrl> {
        ParsedUrl::new(input)
    }

    /// Convert a string to a search URL if needed
    pub fn to_search_url(&self, query: &str) -> String {
        self.search_engine
            .replace("{}", &urlencoding::encode(query))
    }
}

/// Represents a single entry in the navigation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    url: String,
    title: String,
    timestamp: SystemTime,
    favicon: Option<String>,
}

impl HistoryEntry {
    pub fn new(url: String, title: String) -> Self {
        Self {
            url,
            title,
            timestamp: SystemTime::now(),
            favicon: None,
        }
    }

    pub fn with_favicon(mut self, favicon: String) -> Self {
        self.favicon = Some(favicon);
        self
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    pub fn favicon(&self) -> Option<&str> {
        self.favicon.as_deref()
    }
}

/// Manages navigation history for a single tab
#[derive(Clone)]
pub struct NavigationHistory {
    entries: Arc<RwLock<Vec<HistoryEntry>>>,
    current_index: Arc<RwLock<usize>>,
    max_entries: usize,
}

impl NavigationHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            current_index: Arc::new(RwLock::new(0)),
            max_entries,
        }
    }

    /// Add a new entry to the history
    pub fn push(&self, url: String, title: String) -> BrowserResult<()> {
        let mut entries = self.entries
            .write()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire write lock: {}", e)))?;
        
        let mut current_index = self.current_index
            .write()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire index lock: {}", e)))?;

        // Remove all entries after the current index (when navigating back and then to a new page)
        if *current_index < entries.len() - 1 {
            entries.truncate(*current_index + 1);
        }

        // Add new entry
        entries.push(HistoryEntry::new(url, title));
        
        // Update current index
        *current_index = entries.len() - 1;

        // Trim history if it exceeds max entries
        if entries.len() > self.max_entries {
            entries.remove(0);
            *current_index -= 1;
        }

        Ok(())
    }

    /// Navigate back in history
    pub fn back(&self) -> BrowserResult<Option<HistoryEntry>> {
        let entries = self.entries
            .read()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire read lock: {}", e)))?;
        
        let mut current_index = self.current_index
            .write()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire index lock: {}", e)))?;

        if *current_index > 0 {
            *current_index -= 1;
            Ok(Some(entries[*current_index].clone()))
        } else {
            Ok(None)
        }
    }

    /// Navigate forward in history
    pub fn forward(&self) -> BrowserResult<Option<HistoryEntry>> {
        let entries = self.entries
            .read()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire read lock: {}", e)))?;
        
        let mut current_index = self.current_index
            .write()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire index lock: {}", e)))?;

        if *current_index < entries.len() - 1 {
            *current_index += 1;
            Ok(Some(entries[*current_index].clone()))
        } else {
            Ok(None)
        }
    }

    /// Check if we can navigate back
    pub fn can_go_back(&self) -> BrowserResult<bool> {
        let current_index = self.current_index
            .read()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire read lock: {}", e)))?;
        
        Ok(*current_index > 0)
    }

    /// Check if we can navigate forward
    pub fn can_go_forward(&self) -> BrowserResult<bool> {
        let entries = self.entries
            .read()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire read lock: {}", e)))?;
        
        let current_index = self.current_index
            .read()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire read lock: {}", e)))?;
        
        Ok(*current_index < entries.len() - 1)
    }

    /// Get the current entry
    pub fn current(&self) -> BrowserResult<Option<HistoryEntry>> {
        let entries = self.entries
            .read()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire read lock: {}", e)))?;
        
        let current_index = self.current_index
            .read()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire read lock: {}", e)))?;

        Ok(entries.get(*current_index).cloned())
    }

    /// Get all history entries
    pub fn entries(&self) -> BrowserResult<Vec<HistoryEntry>> {
        self.entries
            .read()
            .map(|entries| entries.clone())
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire read lock: {}", e)).into())
    }

    /// Clear the history
    pub fn clear(&self) -> BrowserResult<()> {
        let mut entries = self.entries
            .write()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire write lock: {}", e)))?;
        
        let mut current_index = self.current_index
            .write()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire index lock: {}", e)))?;

        entries.clear();
        *current_index = 0;
        Ok(())
    }
}

/// Navigation state for a tab
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationState {
    Idle,
    Loading,
    Error(String),
}

/// Manages navigation for a single tab
#[derive(Clone)]
pub struct NavigationManager {
    url_manager: UrlManager,
    history: NavigationHistory,
    state: Arc<RwLock<NavigationState>>,
}

impl NavigationManager {
    pub fn new() -> Self {
        Self {
            url_manager: UrlManager::new(),
            history: NavigationHistory::new(100), // Store last 100 entries
            state: Arc::new(RwLock::new(NavigationState::Idle)),
        }
    }

    /// Navigate to a URL or search query
    pub fn navigate(&self, input: &str) -> BrowserResult<ParsedUrl> {
        // Set loading state
        self.set_state(NavigationState::Loading)?;

        // Parse the input
        let parsed = self.url_manager.parse_input(input)?;

        // Add to history (use display URL for search queries)
        self.history.push(parsed.display_url(), input.to_string())?;

        // Return to idle state
        self.set_state(NavigationState::Idle)?;

        Ok(parsed)
    }

    /// Navigate back in history
    pub fn go_back(&self) -> BrowserResult<Option<ParsedUrl>> {
        if !self.can_go_back()? {
            return Ok(None);
        }

        self.set_state(NavigationState::Loading)?;

        // Get previous entry
        if let Some(entry) = self.history.back()? {
            // Parse the URL again to get a ParsedUrl
            let parsed = self.url_manager.parse_input(&entry.url())?;
            self.set_state(NavigationState::Idle)?;
            Ok(Some(parsed))
        } else {
            self.set_state(NavigationState::Idle)?;
            Ok(None)
        }
    }

    /// Navigate forward in history
    pub fn go_forward(&self) -> BrowserResult<Option<ParsedUrl>> {
        if !self.can_go_forward()? {
            return Ok(None);
        }

        self.set_state(NavigationState::Loading)?;

        // Get next entry
        if let Some(entry) = self.history.forward()? {
            // Parse the URL again to get a ParsedUrl
            let parsed = self.url_manager.parse_input(&entry.url())?;
            self.set_state(NavigationState::Idle)?;
            Ok(Some(parsed))
        } else {
            self.set_state(NavigationState::Idle)?;
            Ok(None)
        }
    }

    /// Check if we can navigate back
    pub fn can_go_back(&self) -> BrowserResult<bool> {
        self.history.can_go_back()
    }

    /// Check if we can navigate forward
    pub fn can_go_forward(&self) -> BrowserResult<bool> {
        self.history.can_go_forward()
    }

    /// Get the current URL
    pub fn current_url(&self) -> BrowserResult<Option<ParsedUrl>> {
        if let Some(entry) = self.history.current()? {
            self.url_manager.parse_input(&entry.url())
        } else {
            Ok(None)
        }
    }

    /// Get the current navigation state
    pub fn state(&self) -> BrowserResult<NavigationState> {
        self.state
            .read()
            .map(|state| state.clone())
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire state lock: {}", e)).into())
    }

    /// Set the navigation state
    fn set_state(&self, state: NavigationState) -> BrowserResult<()> {
        let mut current_state = self.state
            .write()
            .map_err(|e| NavigationError::Failed(format!("Failed to acquire state lock: {}", e)))?;
        *current_state = state;
        Ok(())
    }

    /// Set a custom search engine
    pub fn set_search_engine(&mut self, url: String) -> BrowserResult<()> {
        self.url_manager.set_search_engine(url)
    }

    /// Handle a navigation error
    pub fn handle_error(&self, error: &str) -> BrowserResult<()> {
        self.set_state(NavigationState::Error(error.to_string()))
    }

    /// Clear navigation history
    pub fn clear_history(&self) -> BrowserResult<()> {
        self.history.clear()
    }

    /// Reload the current page
    pub fn reload(&self) -> BrowserResult<Option<ParsedUrl>> {
        self.set_state(NavigationState::Loading)?;
        
        if let Some(entry) = self.history.current()? {
            let parsed = self.url_manager.parse_input(&entry.url())?;
            self.set_state(NavigationState::Idle)?;
            Ok(Some(parsed))
        } else {
            self.set_state(NavigationState::Idle)?;
            Ok(None)
        }
    }

    /// Stop the current page load
    pub fn stop_loading(&self) -> BrowserResult<()> {
        if matches!(self.state()?, NavigationState::Loading) {
            self.set_state(NavigationState::Idle)?;
        }
        Ok(())
    }

    /// Get the current page for splitting
    pub fn get_current_for_split(&self) -> BrowserResult<Option<ParsedUrl>> {
        if let Some(entry) = self.history.current()? {
            self.url_manager.parse_input(&entry.url())
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_url_parsing() {
        let url = ParsedUrl::new("https://example.com").unwrap();
        assert!(url.is_secure());
        assert!(!url.is_search());
        assert_eq!(url.original_input(), "https://example.com");
    }

    #[test]
    fn test_url_without_scheme() {
        let url = ParsedUrl::new("example.com").unwrap();
        assert!(url.is_secure());
        assert!(!url.is_search());
        assert_eq!(url.url().scheme(), "https");
    }

    #[test]
    fn test_search_query() {
        let url = ParsedUrl::new("rust programming").unwrap();
        assert!(url.is_secure());
        assert!(url.is_search());
        assert_eq!(url.original_input(), "rust programming");
        assert!(url.display_url().starts_with("🔍"));
    }

    #[test]
    fn test_invalid_url() {
        let result = ParsedUrl::new("http:invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_search_engine() {
        let mut manager = UrlManager::new();
        
        // Valid search engine URL
        assert!(manager.set_search_engine("https://duckduckgo.com/?q={}".to_string()).is_ok());
        
        // Invalid search engine URL (missing placeholder)
        assert!(manager.set_search_engine("https://invalid.com".to_string()).is_err());
    }

    #[test]
    fn test_history_navigation() {
        let history = NavigationHistory::new(5);
        
        // Add some entries
        history.push("https://example.com".to_string(), "Example".to_string()).unwrap();
        history.push("https://example.com/page1".to_string(), "Page 1".to_string()).unwrap();
        history.push("https://example.com/page2".to_string(), "Page 2".to_string()).unwrap();

        // Test current entry
        let current = history.current().unwrap().unwrap();
        assert_eq!(current.url(), "https://example.com/page2");
        assert_eq!(current.title(), "Page 2");

        // Test back navigation
        assert!(history.can_go_back().unwrap());
        let previous = history.back().unwrap().unwrap();
        assert_eq!(previous.url(), "https://example.com/page1");

        // Test forward navigation
        assert!(history.can_go_forward().unwrap());
        let next = history.forward().unwrap().unwrap();
        assert_eq!(next.url(), "https://example.com/page2");
    }

    #[test]
    fn test_history_limits() {
        let history = NavigationHistory::new(3);
        
        // Add more entries than the limit
        history.push("https://example.com/1".to_string(), "Page 1".to_string()).unwrap();
        history.push("https://example.com/2".to_string(), "Page 2".to_string()).unwrap();
        history.push("https://example.com/3".to_string(), "Page 3".to_string()).unwrap();
        history.push("https://example.com/4".to_string(), "Page 4".to_string()).unwrap();

        // Check that oldest entry was removed
        let entries = history.entries().unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].url(), "https://example.com/2");
    }

    #[test]
    fn test_history_clear() {
        let history = NavigationHistory::new(5);
        
        history.push("https://example.com".to_string(), "Example".to_string()).unwrap();
        history.push("https://example.com/page1".to_string(), "Page 1".to_string()).unwrap();
        
        history.clear().unwrap();
        
        assert!(!history.can_go_back().unwrap());
        assert!(!history.can_go_forward().unwrap());
        assert!(history.current().unwrap().is_none());
        assert_eq!(history.entries().unwrap().len(), 0);
    }

    #[test]
    fn test_navigation_manager() {
        let manager = NavigationManager::new();
        
        // Test initial state
        assert_eq!(manager.state().unwrap(), NavigationState::Idle);
        assert!(!manager.can_go_back().unwrap());
        assert!(!manager.can_go_forward().unwrap());
        
        // Test navigation
        let url1 = manager.navigate("https://example.com").unwrap();
        assert_eq!(url1.url().as_str(), "https://example.com/");
        
        let url2 = manager.navigate("https://example.com/page1").unwrap();
        assert_eq!(url2.url().as_str(), "https://example.com/page1");
        
        // Test back navigation
        assert!(manager.can_go_back().unwrap());
        let previous = manager.go_back().unwrap().unwrap();
        assert_eq!(previous.url().as_str(), "https://example.com/");
        
        // Test forward navigation
        assert!(manager.can_go_forward().unwrap());
        let next = manager.go_forward().unwrap().unwrap();
        assert_eq!(next.url().as_str(), "https://example.com/page1");
    }

    #[test]
    fn test_navigation_error_handling() {
        let manager = NavigationManager::new();
        
        // Test error state
        manager.handle_error("Failed to load page").unwrap();
        assert_eq!(
            manager.state().unwrap(),
            NavigationState::Error("Failed to load page".to_string())
        );
        
        // Test recovery
        manager.navigate("https://example.com").unwrap();
        assert_eq!(manager.state().unwrap(), NavigationState::Idle);
    }

    #[test]
    fn test_search_navigation() {
        let manager = NavigationManager::new();
        
        // Test search query navigation
        let search = manager.navigate("rust programming").unwrap();
        assert!(search.is_search());
        assert!(search.display_url().starts_with("🔍"));
        
        // Verify it's in history
        let current = manager.current_url().unwrap().unwrap();
        assert!(current.is_search());
    }

    #[test]
    fn test_reload() {
        let manager = NavigationManager::new();
        
        // Navigate to a page first
        let url = "https://example.com";
        manager.navigate(url).unwrap();
        
        // Test reload
        let reloaded = manager.reload().unwrap().unwrap();
        assert_eq!(reloaded.url().as_str(), "https://example.com/");
        assert_eq!(manager.state().unwrap(), NavigationState::Idle);
    }

    #[test]
    fn test_stop_loading() {
        let manager = NavigationManager::new();
        
        // Set loading state
        manager.set_state(NavigationState::Loading).unwrap();
        
        // Stop loading
        manager.stop_loading().unwrap();
        assert_eq!(manager.state().unwrap(), NavigationState::Idle);
    }

    #[test]
    fn test_split_tab() {
        let manager = NavigationManager::new();
        
        // Navigate to a page
        let url = "https://example.com";
        manager.navigate(url).unwrap();
        
        // Get URL for splitting
        let split_url = manager.get_current_for_split().unwrap().unwrap();
        assert_eq!(split_url.url().as_str(), "https://example.com/");
    }
} 
