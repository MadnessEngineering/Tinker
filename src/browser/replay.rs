use std::fs::File;
use std::io::{self, BufWriter, BufReader};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use crate::event::BrowserEvent;

#[derive(Debug, Serialize, Deserialize)]
struct EventRecord {
    timestamp: u64,  // milliseconds since start
    event: BrowserEvent,
}

#[derive(Default)]
pub struct EventRecorder {
    events: Vec<EventRecord>,
    start_time: Option<Instant>,
    save_path: Option<String>,
    is_recording: bool,
}

impl EventRecorder {
    pub fn start(&mut self) {
        self.events.clear();
        self.start_time = Some(Instant::now());
        self.is_recording = true;
    }

    pub fn stop(&mut self) {
        self.is_recording = false;
    }

    pub fn set_save_path(&mut self, path: String) {
        self.save_path = Some(path);
    }

    pub fn record_event(&mut self, event: BrowserEvent) {
        if !self.is_recording {
            return;
        }

        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            let timestamp = elapsed.as_millis() as u64;
            
            self.events.push(EventRecord {
                timestamp,
                event,
            });
        }
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.events)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct EventPlayer {
    events: Vec<EventRecord>,
    start_time: Option<Instant>,
    current_index: usize,
    playback_speed: f32,
    is_playing: bool,
}

impl EventPlayer {
    pub fn load(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.events = serde_json::from_reader(reader)?;
        self.current_index = 0;
        Ok(())
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.current_index = 0;
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.1).min(10.0);
    }

    pub fn next_event(&mut self) -> Option<BrowserEvent> {
        if !self.is_playing || self.current_index >= self.events.len() {
            return None;
        }

        if let Some(start_time) = self.start_time {
            let current_time = start_time.elapsed();
            let target_time = Duration::from_millis(
                (self.events[self.current_index].timestamp as f32 / self.playback_speed) as u64
            );

            if current_time >= target_time {
                let event = self.events[self.current_index].event.clone();
                self.current_index += 1;
                Some(event)
            } else {
                None
            }
        } else {
            None
        }
    }
} 
