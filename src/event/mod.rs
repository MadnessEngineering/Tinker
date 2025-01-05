//! MQTT event system

use rumqttc::{Client, MqttOptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum BrowserEvent {
    Navigation { url: String },
    Click { x: i32, y: i32 },
    KeyPress { key: String },
    // Add more events as needed
}

pub struct EventSystem {
    mqtt_client: Option<Client>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self { mqtt_client: None }
    }
} 
