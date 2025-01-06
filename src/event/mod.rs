//! MQTT event system

use rumqttc::{Client, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use tracing::{info, error, debug};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize)]
pub enum BrowserEvent {
    Navigation { url: String },
    TabCreated { id: usize },
    TabClosed { id: usize },
    TabSwitched { id: usize },
    PageLoaded { url: String },
    TitleChanged { title: String },
    Error { message: String },
}

pub struct EventSystem {
    client: Option<Client>,
    options: MqttOptions,
    last_error_log: Option<Instant>,
    error_count: usize,
}

impl EventSystem {
    pub fn new(broker_url: &str, client_id: &str) -> Self {
        info!("Creating new event system with broker: {}", broker_url);

        let mut options = MqttOptions::new(client_id, broker_url, 1883);
        options.set_keep_alive(Duration::from_secs(5));
        options.set_clean_session(true);

        Self {
            client: None,
            options,
            last_error_log: None,
            error_count: 0,
        }
    }

    pub fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Connecting to MQTT broker...");
        let (client, mut connection) = Client::new(self.options.clone(), 10);

        // Spawn a thread to handle incoming messages
        std::thread::spawn(move || {
            debug!("Starting MQTT event loop");
            let mut last_error_log = None;
            let mut error_count = 0;
            
            for notification in connection.iter() {
                match notification {
                    Ok(event) => debug!("Received MQTT event: {:?}", event),
                    Err(e) => {
                        let now = Instant::now();
                        if let Some(last) = last_error_log {
                            if now.duration_since(last) > Duration::from_secs(5) {
                                error!("MQTT error occurred {} times: {:?}", error_count + 1, e);
                                last_error_log = Some(now);
                                error_count = 0;
                            } else {
                                error_count += 1;
                            }
                        } else {
                            error!("MQTT error: {:?}", e);
                            last_error_log = Some(now);
                        }
                    }
                }
            }
        });

        self.client = Some(client);
        Ok(())
    }

    pub fn publish(&mut self, event: BrowserEvent) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut client) = self.client {
            let topic = match event {
                BrowserEvent::Navigation { .. } => "browser/navigation",
                BrowserEvent::TabCreated { .. } => "browser/tabs/created",
                BrowserEvent::TabClosed { .. } => "browser/tabs/closed",
                BrowserEvent::TabSwitched { .. } => "browser/tabs/switched",
                BrowserEvent::PageLoaded { .. } => "browser/page/loaded",
                BrowserEvent::TitleChanged { .. } => "browser/page/title",
                BrowserEvent::Error { .. } => "browser/error",
            };

            let payload = serde_json::to_string(&event)?;
            debug!("Publishing event to {}: {}", topic, payload);
            client.publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())?;
            Ok(())
        } else {
            let now = Instant::now();
            if let Some(last) = self.last_error_log {
                if now.duration_since(last) > Duration::from_secs(5) {
                    error!("Cannot publish event: MQTT client not connected (occurred {} times)", self.error_count + 1);
                    self.last_error_log = Some(now);
                    self.error_count = 0;
                } else {
                    self.error_count += 1;
                }
            } else {
                error!("Cannot publish event: MQTT client not connected");
                self.last_error_log = Some(now);
            }
            Err("MQTT client not connected".into())
        }
    }

    pub fn subscribe(&mut self, topic: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut client) = self.client {
            debug!("Subscribing to topic: {}", topic);
            client.subscribe(topic, QoS::AtLeastOnce)?;
            Ok(())
        } else {
            let now = Instant::now();
            if let Some(last) = self.last_error_log {
                if now.duration_since(last) > Duration::from_secs(5) {
                    error!("Cannot subscribe: MQTT client not connected (occurred {} times)", self.error_count + 1);
                    self.last_error_log = Some(now);
                    self.error_count = 0;
                } else {
                    self.error_count += 1;
                }
            } else {
                error!("Cannot subscribe: MQTT client not connected");
                self.last_error_log = Some(now);
            }
            Err("MQTT client not connected".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = BrowserEvent::Navigation {
            url: "https://example.com".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Navigation"));
        assert!(json.contains("example.com"));
    }

    #[test]
    fn test_event_system_creation() {
        let system = EventSystem::new("localhost", "test-client");
        assert!(system.client.is_none());
    }
} 
