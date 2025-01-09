use std::sync::{Arc, Mutex};
use tinker::{
    browser::BrowserEngine,
    event::{EventSystem, BrowserEvent},
};

#[test]
fn test_browser_events() {
    // Create event system
    let mut events = EventSystem::new("localhost", "test-browser");
    events.connect().unwrap();
    let events = Arc::new(Mutex::new(events));

    // Create browser with events enabled
    let mut browser = BrowserEngine::new(true, Some(events.clone()));

    // Create a tab and verify the event
    let tab_id = browser.create_tab("about:blank").unwrap();
    
    // Navigate to a URL and verify the event
    browser.navigate("https://example.com").unwrap();

    // Switch tabs and verify the event
    browser.switch_to_tab(tab_id).unwrap();

    // Close tab and verify the event
    browser.close_tab(tab_id).unwrap();
}

#[test]
fn test_event_subscription() {
    // Create event system
    let mut events = EventSystem::new("localhost", "test-subscriber");
    events.connect().unwrap();

    // Subscribe to all browser events
    events.subscribe("browser/#").unwrap();
}

#[test]
fn test_event_publishing() {
    // Create event system
    let mut events = EventSystem::new("localhost", "test-publisher");
    events.connect().unwrap();

    // Test publishing different types of events
    let events_to_test = vec![
        BrowserEvent::Navigation {
            url: "https://example.com".to_string(),
        },
        BrowserEvent::TabCreated { id: 1 },
        BrowserEvent::TabClosed { id: 1 },
        BrowserEvent::TabSwitched { id: 1 },
        BrowserEvent::TabUrlChanged {
            id: 1,
            url: "https://example.com".to_string(),
        },
        BrowserEvent::TabTitleChanged {
            id: 1,
            title: "Test Page".to_string(),
        },
        BrowserEvent::PageLoaded {
            url: "https://example.com".to_string(),
        },
        BrowserEvent::Error {
            message: "Test error".to_string(),
        },
    ];

    for event in events_to_test {
        events.publish(event).unwrap();
    }
} 
