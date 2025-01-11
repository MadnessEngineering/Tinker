use std::sync::{Arc, Mutex};
use tinker::{
    browser::BrowserEngine,
    event::EventSystem,
};

#[test]
fn test_browser_creation() {
    let browser = BrowserEngine::new(false, None, Some("about:blank".to_string()));
    assert!(browser.is_ok());
}

#[test]
fn test_event_system_creation() {
    // Create event system
    let mut events = EventSystem::new("localhost", "test-browser");
    assert!(events.connect().is_ok());
    let events = Arc::new(Mutex::new(events));

    // Create browser with events enabled
    let browser = BrowserEngine::new(false, Some(events), Some("about:blank".to_string()));
    assert!(browser.is_ok());
} 
