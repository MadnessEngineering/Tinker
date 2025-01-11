use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserEvent {
    Navigation {
        url: String,
    },
    TabCreated {
        id: usize,
        url: String,
    },
    TabUrlChanged {
        id: usize,
        url: String,
    },
    Error {
        message: String,
    },
}

pub struct EventSystem {
    client_id: String,
    host: String,
    client: Option<Arc<Mutex<rumqttc::Client>>>,
    test_mode: bool,
}

impl EventSystem {
    pub fn new(host: &str, client_id: &str) -> Self {
        // Check if we're in test mode
        let test_mode = env::var("TINKER_TEST_MODE").unwrap_or_default() == "1";
        
        // Allow overriding host/port via env vars
        let host = env::var("TINKER_MQTT_HOST").unwrap_or(host.to_string());
        
        Self {
            host,
            client_id: client_id.to_string(),
            client: None,
            test_mode,
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        if self.test_mode {
            return Ok(());
        }

        let port = env::var("TINKER_MQTT_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(1883);

        let options = rumqttc::MqttOptions::new(&self.client_id, &self.host, port);
        let (client, _eventloop) = rumqttc::Client::new(options, 10);
        self.client = Some(Arc::new(Mutex::new(client)));
        Ok(())
    }

    pub fn publish(&self, event: BrowserEvent) -> Result<()> {
        if self.test_mode {
            return Ok(());
        }

        if let Some(client) = &self.client {
            let json = serde_json::to_string(&event)?;
            let topic = match &event {
                BrowserEvent::Navigation { .. } => "browser/navigation",
                BrowserEvent::TabCreated { .. } => "browser/tabs/created",
                BrowserEvent::TabUrlChanged { .. } => "browser/tabs/url",
                BrowserEvent::Error { .. } => "browser/error",
            };
            client.lock().unwrap().publish(topic, rumqttc::QoS::AtLeastOnce, false, json)?;
        }
        Ok(())
    }
}
