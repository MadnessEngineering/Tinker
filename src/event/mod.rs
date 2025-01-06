//! MQTT event system

use rumqttc::Client;
use tracing::info;

pub struct EventSystem {
    client: Client,
}

impl EventSystem {
    pub fn new() -> Self {
        info!("Creating new event system");
        unimplemented!("Event system not implemented yet")
    }
} 
