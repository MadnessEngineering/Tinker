use std::fs;
use std::time::Instant;
use serde::{Serialize, Deserialize};
use crate::event::BrowserEvent;

#[derive(Default)]
pub struct EventRecorder {
    events: Vec<BrowserEvent>,
    recording: bool,
    start_time: Option<Instant>,
    save_path: Option<String>,
}

impl EventRecorder {
    pub fn start(&mut self) {
        self.recording = true;
        self.start_time = Some(Instant::now());
        if let Some(path) = &self.save_path {
            if let Err(e) = self.save(path) {
                eprintln!("Failed to save initial recording: {}", e);
            }
        }
    }

    pub fn stop(&mut self) {
        self.recording = false;
        if let Some(path) = &self.save_path {
            if let Err(e) = self.save(path) {
                eprintln!("Failed to save final recording: {}", e);
            }
        }
    }

    pub fn record_event(&mut self, event: BrowserEvent) {
        if self.recording {
            self.events.push(event);
            if let Some(path) = &self.save_path {
                if let Err(e) = self.save(path) {
                    eprintln!("Failed to save recording: {}", e);
                }
            }
        }
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(&self.events)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn set_save_path(&mut self, path: String) {
        let path_clone = path.clone();
        self.save_path = Some(path);
        if let Err(e) = self.save(&path_clone) {
            eprintln!("Failed to save initial recording: {}", e);
        }
    }
}

#[derive(Default)]
pub struct EventPlayer {
    events: Vec<BrowserEvent>,
    current_index: usize,
    playing: bool,
    speed: f32,
}

impl EventPlayer {
    pub fn load(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        self.events = serde_json::from_str(&json)?;
        self.current_index = 0;
        Ok(())
    }

    pub fn start(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn next_event(&mut self) -> Option<BrowserEvent> {
        if !self.playing || self.current_index >= self.events.len() {
            return None;
        }

        let event = self.events[self.current_index].clone();
        self.current_index += 1;
        Some(event)
    }
} 
