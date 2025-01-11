use std::env;
use tinker::browser::event::{EventSystem, BrowserEvent};

fn setup() {
    env::set_var("TINKER_TEST_MODE", "1");
}

#[test]
fn test_event_system_creation() {
    setup();
    let mut events = EventSystem::new("localhost", "test-client");
    assert!(events.connect().is_ok());
}

#[test]
fn test_event_publishing() {
    setup();
    let mut events = EventSystem::new("localhost", "test-client");
    events.connect().unwrap();

    let event = BrowserEvent::Navigation {
        url: "https://example.com".to_string(),
    };
    assert!(events.publish(event).is_ok());
} 
