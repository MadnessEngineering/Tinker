use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::event::BrowserEvent;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    pub timestamp: Duration,
    pub event: BrowserEvent,
}

#[derive(Default)]
pub struct EventRecorder {
    events: VecDeque<RecordedEvent>,
    start_time: Option<Instant>,
    recording: bool,
}

impl EventRecorder {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            start_time: None,
            recording: false,
        }
    }

    pub fn start_recording(&mut self) {
        debug!("Starting event recording");
        self.events.clear();
        self.start_time = Some(Instant::now());
        self.recording = true;
    }

    pub fn stop_recording(&mut self) {
        debug!("Stopping event recording");
        self.recording = false;
    }

    pub fn record_event(&mut self, event: BrowserEvent) {
        if self.recording {
            if let Some(start) = self.start_time {
                let timestamp = start.elapsed();
                self.events.push_back(RecordedEvent { timestamp, event });
                debug!("Recorded event at {:?}", timestamp);
            }
        }
    }

    pub fn save_recording(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(&self.events)?;
        std::fs::write(path, json)?;
        debug!("Saved recording to {}", path);
        Ok(())
    }

    pub fn load_recording(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        self.events = serde_json::from_str(&json)?;
        debug!("Loaded recording from {}", path);
        Ok(())
    }
}

#[derive(Default)]
pub struct EventPlayer {
    events: VecDeque<RecordedEvent>,
    playback_start: Option<Instant>,
    speed: f32,
    paused: bool,
}

impl EventPlayer {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            playback_start: None,
            speed: 1.0,
            paused: false,
        }
    }

    pub fn load_events(&mut self, events: VecDeque<RecordedEvent>) {
        self.events = events;
    }

    pub fn start_playback(&mut self) {
        debug!("Starting event playback");
        self.playback_start = Some(Instant::now());
        self.paused = false;
    }

    pub fn pause_playback(&mut self) {
        debug!("Pausing event playback");
        self.paused = true;
    }

    pub fn resume_playback(&mut self) {
        debug!("Resuming event playback");
        self.paused = false;
    }

    pub fn set_speed(&mut self, speed: f32) {
        debug!("Setting playback speed to {}", speed);
        self.speed = speed;
    }

    pub fn next_event(&mut self) -> Option<BrowserEvent> {
        if self.paused {
            return None;
        }

        if let Some(start) = self.playback_start {
            if let Some(event) = self.events.front() {
                let current_time = start.elapsed();
                let target_time = Duration::from_secs_f32(event.timestamp.as_secs_f32() / self.speed);

                if current_time >= target_time {
                    let event = self.events.pop_front().unwrap();
                    debug!("Playing event at {:?}", current_time);
                    return Some(event.event);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_recording() {
        let mut recorder = EventRecorder::new();
        recorder.start_recording();
        
        let event = BrowserEvent::Navigation {
            url: "https://example.com".to_string(),
        };
        recorder.record_event(event);
        
        assert_eq!(recorder.events.len(), 1);
    }

    #[test]
    fn test_event_playback() {
        let mut player = EventPlayer::new();
        let mut events = VecDeque::new();
        
        events.push_back(RecordedEvent {
            timestamp: Duration::from_secs(1),
            event: BrowserEvent::Navigation {
                url: "https://example.com".to_string(),
            },
        });
        
        player.load_events(events);
        player.start_playback();
        
        // No event should be available immediately
        assert!(player.next_event().is_none());
        
        // Wait for 1 second
        std::thread::sleep(Duration::from_secs(1));
        
        // Now the event should be available
        assert!(player.next_event().is_some());
    }
} 
