//! Event recording and replay system for browser automation and testing
//!
//! This module provides comprehensive recording and playback capabilities:
//! - Record browser events with precise timing
//! - Replay sessions with speed control
//! - Pause, resume, seek, and step through recordings
//! - Session management for multiple recordings
//! - Test generation from recordings
//! - Recording editing (trim, splice, insert)

use std::fs::File;
use std::io::{self, BufWriter, BufReader};
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use crate::event::BrowserEvent;
use tracing::{debug, info, warn, error};

/// Recording format version for compatibility
const RECORDING_VERSION: &str = "1.0";

/// Individual event record with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    /// Relative timestamp from recording start (milliseconds)
    pub timestamp_ms: u64,
    /// The browser event
    pub event: BrowserEvent,
    /// Optional metadata for this event
    pub metadata: Option<EventMetadata>,
}

/// Metadata for individual events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Event description
    pub description: Option<String>,
    /// Whether this is an assertion point for testing
    pub is_assertion: bool,
    /// Expected state for assertion
    pub expected_state: Option<serde_json::Value>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Complete recording session with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    /// Recording format version
    pub version: String,
    /// Recording metadata
    pub metadata: RecordingMetadata,
    /// Recorded events
    pub events: Vec<EventRecord>,
    /// Snapshots for deterministic replay
    pub snapshots: Vec<StateSnapshot>,
}

/// Recording session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    /// Recording ID
    pub id: String,
    /// Recording name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Number of events
    pub event_count: usize,
    /// Starting URL
    pub start_url: String,
    /// Browser configuration at recording time
    pub browser_config: BrowserConfig,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Custom metadata
    pub custom: serde_json::Value,
}

/// Browser configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Viewport width
    pub viewport_width: u32,
    /// Viewport height
    pub viewport_height: u32,
    /// User agent string
    pub user_agent: Option<String>,
    /// Whether JavaScript is enabled
    pub javascript_enabled: bool,
    /// Additional configuration
    pub custom: serde_json::Value,
}

/// State snapshot for deterministic replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Timestamp when snapshot was taken
    pub timestamp_ms: u64,
    /// DOM snapshot (simplified)
    pub dom_state: Option<String>,
    /// Network state
    pub network_state: Option<serde_json::Value>,
    /// Browser state
    pub browser_state: serde_json::Value,
}

/// Event recorder with enhanced features
#[derive(Default)]
pub struct EventRecorder {
    /// Current recording session
    recording: Option<Recording>,
    /// Recording start time
    start_time: Option<Instant>,
    /// Whether currently recording
    is_recording: bool,
    /// Whether to capture snapshots
    capture_snapshots: bool,
    /// Snapshot interval (milliseconds)
    snapshot_interval_ms: u64,
    /// Last snapshot time
    last_snapshot: Option<Instant>,
    /// Event filters (types to exclude)
    filters: Vec<String>,
}

impl EventRecorder {
    pub fn new() -> Self {
        Self {
            recording: None,
            start_time: None,
            is_recording: false,
            capture_snapshots: false,
            snapshot_interval_ms: 5000, // 5 seconds
            last_snapshot: None,
            filters: Vec::new(),
        }
    }

    /// Start recording with metadata
    pub fn start(&mut self, name: String, start_url: String) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let recording_id = format!("rec_{}_{}", timestamp, name.replace(" ", "_"));

        self.recording = Some(Recording {
            version: RECORDING_VERSION.to_string(),
            metadata: RecordingMetadata {
                id: recording_id.clone(),
                name,
                description: None,
                created_at: timestamp,
                modified_at: timestamp,
                duration_ms: 0,
                event_count: 0,
                start_url,
                browser_config: BrowserConfig {
                    viewport_width: 1920,
                    viewport_height: 1080,
                    user_agent: None,
                    javascript_enabled: true,
                    custom: serde_json::json!({}),
                },
                tags: Vec::new(),
                custom: serde_json::json!({}),
            },
            events: Vec::new(),
            snapshots: Vec::new(),
        });

        self.start_time = Some(Instant::now());
        self.last_snapshot = Some(Instant::now());
        self.is_recording = true;

        info!("Started recording: {}", recording_id);
    }

    /// Stop recording and finalize metadata
    pub fn stop(&mut self) -> Option<Recording> {
        if let Some(ref mut recording) = self.recording {
            if let Some(start) = self.start_time {
                let duration = start.elapsed();
                recording.metadata.duration_ms = duration.as_millis() as u64;
                recording.metadata.event_count = recording.events.len();
                recording.metadata.modified_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
            }
        }

        self.is_recording = false;
        info!("Stopped recording");

        self.recording.take()
    }

    /// Pause recording
    pub fn pause(&mut self) {
        self.is_recording = false;
        debug!("Paused recording");
    }

    /// Resume recording
    pub fn resume(&mut self) {
        self.is_recording = true;
        debug!("Resumed recording");
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Set recording description
    pub fn set_description(&mut self, description: String) {
        if let Some(ref mut recording) = self.recording {
            recording.metadata.description = Some(description);
        }
    }

    /// Add tag to recording
    pub fn add_tag(&mut self, tag: String) {
        if let Some(ref mut recording) = self.recording {
            if !recording.metadata.tags.contains(&tag) {
                recording.metadata.tags.push(tag);
            }
        }
    }

    /// Add event filter (exclude certain event types)
    pub fn add_filter(&mut self, event_type: String) {
        if !self.filters.contains(&event_type) {
            self.filters.push(event_type);
        }
    }

    /// Clear all filters
    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }

    /// Enable snapshot capturing
    pub fn enable_snapshots(&mut self, interval_ms: u64) {
        self.capture_snapshots = true;
        self.snapshot_interval_ms = interval_ms;
    }

    /// Disable snapshot capturing
    pub fn disable_snapshots(&mut self) {
        self.capture_snapshots = false;
    }

    /// Record an event
    pub fn record_event(&mut self, event: BrowserEvent) {
        if !self.is_recording {
            return;
        }

        // Check filters
        let event_type = format!("{:?}", event).split(' ').next().unwrap_or("").to_string();
        if self.filters.contains(&event_type) {
            return;
        }

        if let (Some(ref mut recording), Some(start)) = (&mut self.recording, self.start_time) {
            let elapsed = start.elapsed();

            recording.events.push(EventRecord {
                timestamp_ms: elapsed.as_millis() as u64,
                event,
                metadata: None,
            });

            debug!("Recorded event at {:?}", elapsed);

            // Check if we should capture a snapshot
            if self.capture_snapshots {
                if let Some(last) = self.last_snapshot {
                    if last.elapsed().as_millis() as u64 >= self.snapshot_interval_ms {
                        self.capture_snapshot();
                    }
                }
            }
        }
    }

    /// Add assertion point at current time
    pub fn add_assertion(&mut self, expected_state: serde_json::Value, description: Option<String>) {
        if let Some(ref mut recording) = self.recording {
            if let Some(last_event) = recording.events.last_mut() {
                last_event.metadata = Some(EventMetadata {
                    description,
                    is_assertion: true,
                    expected_state: Some(expected_state),
                    tags: vec!["assertion".to_string()],
                });
                info!("Added assertion point");
            }
        }
    }

    /// Capture state snapshot
    fn capture_snapshot(&mut self) {
        if let (Some(ref mut recording), Some(start)) = (&mut self.recording, self.start_time) {
            let elapsed = start.elapsed();

            recording.snapshots.push(StateSnapshot {
                timestamp_ms: elapsed.as_millis() as u64,
                dom_state: None, // TODO: Capture actual DOM state
                network_state: None,
                browser_state: serde_json::json!({}),
            });

            self.last_snapshot = Some(Instant::now());
            debug!("Captured snapshot at {:?}", elapsed);
        }
    }

    /// Save recording to file
    pub fn save(&self, path: &str) -> io::Result<()> {
        if let Some(ref recording) = self.recording {
            let file = File::create(path)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, recording)?;
            info!("Saved recording to {} ({} events)", path, recording.events.len());
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No recording to save"))
        }
    }

    /// Get current recording
    pub fn get_recording(&self) -> Option<&Recording> {
        self.recording.as_ref()
    }

    /// Get event count
    pub fn get_event_count(&self) -> usize {
        self.recording.as_ref().map_or(0, |r| r.events.len())
    }

    /// Get recording duration
    pub fn get_duration(&self) -> Duration {
        if let Some(start) = self.start_time {
            start.elapsed()
        } else if let Some(ref recording) = self.recording {
            Duration::from_millis(recording.metadata.duration_ms)
        } else {
            Duration::from_secs(0)
        }
    }
}

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Completed,
}

/// Event player with advanced controls
pub struct EventPlayer {
    /// Loaded recording
    recording: Option<Recording>,
    /// Playback state
    state: PlaybackState,
    /// Playback start time
    start_time: Option<Instant>,
    /// Pause time (for calculating offsets)
    pause_time: Option<Instant>,
    /// Total paused duration
    paused_duration: Duration,
    /// Current event index
    current_index: usize,
    /// Playback speed multiplier
    speed: f32,
    /// Whether to loop playback
    loop_playback: bool,
    /// Event callback for when events are played
    on_event: Option<Box<dyn Fn(&EventRecord) + Send>>,
}

impl Default for EventPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl EventPlayer {
    pub fn new() -> Self {
        Self {
            recording: None,
            state: PlaybackState::Stopped,
            start_time: None,
            pause_time: None,
            paused_duration: Duration::from_secs(0),
            current_index: 0,
            speed: 1.0,
            loop_playback: false,
            on_event: None,
        }
    }

    /// Load recording from file
    pub fn load(&mut self, path: &str) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let recording: Recording = serde_json::from_reader(reader)?;

        info!("Loaded recording: {} ({} events)",
              recording.metadata.name,
              recording.events.len());

        self.recording = Some(recording);
        self.current_index = 0;
        self.state = PlaybackState::Stopped;

        Ok(())
    }

    /// Load recording from Recording struct
    pub fn load_recording(&mut self, recording: Recording) {
        info!("Loaded recording: {} ({} events)",
              recording.metadata.name,
              recording.events.len());

        self.recording = Some(recording);
        self.current_index = 0;
        self.state = PlaybackState::Stopped;
    }

    /// Start playback
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.current_index = 0;
        self.paused_duration = Duration::from_secs(0);
        self.state = PlaybackState::Playing;
        info!("Started playback");
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.current_index = 0;
        self.start_time = None;
        self.pause_time = None;
        self.paused_duration = Duration::from_secs(0);
        info!("Stopped playback");
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
            self.pause_time = Some(Instant::now());
            debug!("Paused playback");
        }
    }

    /// Resume playback
    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused {
            if let Some(pause) = self.pause_time {
                self.paused_duration += pause.elapsed();
                self.pause_time = None;
            }
            self.state = PlaybackState::Playing;
            debug!("Resumed playback");
        }
    }

    /// Step forward one event
    pub fn step_forward(&mut self) -> Option<BrowserEvent> {
        if let Some(ref recording) = self.recording {
            if self.current_index < recording.events.len() {
                let event = recording.events[self.current_index].event.clone();
                self.current_index += 1;
                debug!("Stepped forward to event {}", self.current_index);
                Some(event)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Step backward one event
    pub fn step_backward(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            debug!("Stepped backward to event {}", self.current_index);
        }
    }

    /// Seek to specific timestamp
    pub fn seek(&mut self, timestamp_ms: u64) {
        if let Some(ref recording) = self.recording {
            // Find the event closest to the timestamp
            for (i, event) in recording.events.iter().enumerate() {
                if event.timestamp_ms >= timestamp_ms {
                    self.current_index = i;
                    debug!("Seeked to timestamp {}ms (event {})", timestamp_ms, i);
                    return;
                }
            }
            // If we didn't find anything, go to the end
            self.current_index = recording.events.len();
        }
    }

    /// Seek to specific event index
    pub fn seek_to_index(&mut self, index: usize) {
        if let Some(ref recording) = self.recording {
            self.current_index = index.min(recording.events.len());
            debug!("Seeked to event index {}", self.current_index);
        }
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.1).min(10.0);
        debug!("Set playback speed to {}", self.speed);
    }

    /// Enable/disable looping
    pub fn set_loop(&mut self, enable: bool) {
        self.loop_playback = enable;
        debug!("Playback looping: {}", enable);
    }

    /// Get next event to play (based on timing)
    pub fn next_event(&mut self) -> Option<BrowserEvent> {
        if self.state != PlaybackState::Playing {
            return None;
        }

        if let (Some(ref recording), Some(start)) = (&self.recording, self.start_time) {
            if self.current_index < recording.events.len() {
                let record = &recording.events[self.current_index];
                let elapsed = start.elapsed() - self.paused_duration;
                let target_time = Duration::from_millis(
                    (record.timestamp_ms as f32 / self.speed) as u64
                );

                if elapsed >= target_time {
                    let event = record.event.clone();
                    self.current_index += 1;
                    debug!("Playing event {} at {:?}", self.current_index - 1, elapsed);

                    // Call callback if set
                    if let Some(ref callback) = self.on_event {
                        callback(record);
                    }

                    return Some(event);
                }
            } else {
                // Reached end of recording
                if self.loop_playback {
                    self.start();
                } else {
                    self.state = PlaybackState::Completed;
                    info!("Playback completed");
                }
            }
        }

        None
    }

    /// Get playback state
    pub fn get_state(&self) -> PlaybackState {
        self.state
    }

    /// Get current position (milliseconds)
    pub fn get_position(&self) -> u64 {
        if let Some(start) = self.start_time {
            ((start.elapsed() - self.paused_duration).as_millis() as f64 * self.speed as f64) as u64
        } else {
            0
        }
    }

    /// Get total duration (milliseconds)
    pub fn get_duration(&self) -> u64 {
        self.recording.as_ref()
            .map_or(0, |r| r.metadata.duration_ms)
    }

    /// Get current event index
    pub fn get_current_index(&self) -> usize {
        self.current_index
    }

    /// Get total event count
    pub fn get_event_count(&self) -> usize {
        self.recording.as_ref()
            .map_or(0, |r| r.events.len())
    }

    /// Get recording metadata
    pub fn get_metadata(&self) -> Option<&RecordingMetadata> {
        self.recording.as_ref().map(|r| &r.metadata)
    }

    /// Get events in time range
    pub fn get_events_in_range(&self, start_ms: u64, end_ms: u64) -> Vec<&EventRecord> {
        if let Some(ref recording) = self.recording {
            recording.events.iter()
                .filter(|e| e.timestamp_ms >= start_ms && e.timestamp_ms <= end_ms)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Play event (for manual control)
    pub fn play_event(&mut self, event: BrowserEvent) {
        if self.state == PlaybackState::Playing {
            if let Some(start) = self.start_time {
                let elapsed = start.elapsed();
                debug!("Playing event at {:?}: {:?}", elapsed, event);
            }
        }
    }
}

/// Recording editor for trimming, splicing, and modifying recordings
pub struct RecordingEditor {
    recording: Recording,
}

impl RecordingEditor {
    /// Create editor from recording
    pub fn new(recording: Recording) -> Self {
        Self { recording }
    }

    /// Trim recording to time range
    pub fn trim(&mut self, start_ms: u64, end_ms: u64) {
        self.recording.events.retain(|e| e.timestamp_ms >= start_ms && e.timestamp_ms <= end_ms);

        // Adjust timestamps to start from 0
        if let Some(first) = self.recording.events.first() {
            let offset = first.timestamp_ms;
            for event in &mut self.recording.events {
                event.timestamp_ms -= offset;
            }
        }

        self.recording.metadata.event_count = self.recording.events.len();
        self.recording.metadata.duration_ms = end_ms - start_ms;
        info!("Trimmed recording to {}ms - {}ms", start_ms, end_ms);
    }

    /// Remove events in time range
    pub fn remove_range(&mut self, start_ms: u64, end_ms: u64) {
        let removed_duration = end_ms - start_ms;
        self.recording.events.retain(|e| e.timestamp_ms < start_ms || e.timestamp_ms > end_ms);

        // Adjust timestamps after the removed section
        for event in &mut self.recording.events {
            if event.timestamp_ms > end_ms {
                event.timestamp_ms -= removed_duration;
            }
        }

        self.recording.metadata.event_count = self.recording.events.len();
        self.recording.metadata.duration_ms -= removed_duration;
        info!("Removed events in range {}ms - {}ms", start_ms, end_ms);
    }

    /// Insert events at specific timestamp
    pub fn insert_events(&mut self, at_ms: u64, events: Vec<EventRecord>) {
        let mut new_events = events;

        // Adjust timestamps of inserted events
        for event in &mut new_events {
            event.timestamp_ms += at_ms;
        }

        // Find insertion point
        let insert_idx = self.recording.events.iter()
            .position(|e| e.timestamp_ms > at_ms)
            .unwrap_or(self.recording.events.len());

        // Insert events
        self.recording.events.splice(insert_idx..insert_idx, new_events.iter().cloned());

        self.recording.metadata.event_count = self.recording.events.len();
        info!("Inserted {} events at {}ms", new_events.len(), at_ms);
    }

    /// Splice two recordings together
    pub fn splice(&mut self, other: Recording, at_ms: u64) {
        let mut other_events = other.events;

        // Adjust timestamps
        for event in &mut other_events {
            event.timestamp_ms += at_ms;
        }

        // Find insertion point
        let insert_idx = self.recording.events.iter()
            .position(|e| e.timestamp_ms > at_ms)
            .unwrap_or(self.recording.events.len());

        // Insert events
        self.recording.events.splice(insert_idx..insert_idx, other_events.iter().cloned());

        self.recording.metadata.event_count = self.recording.events.len();
        info!("Spliced recording at {}ms", at_ms);
    }

    /// Change playback speed of recording (adjusts all timestamps)
    pub fn change_speed(&mut self, speed_multiplier: f32) {
        for event in &mut self.recording.events {
            event.timestamp_ms = (event.timestamp_ms as f32 / speed_multiplier) as u64;
        }

        self.recording.metadata.duration_ms = (self.recording.metadata.duration_ms as f32 / speed_multiplier) as u64;
        info!("Changed recording speed by {}x", speed_multiplier);
    }

    /// Add delay after specific timestamp
    pub fn add_delay(&mut self, after_ms: u64, delay_ms: u64) {
        for event in &mut self.recording.events {
            if event.timestamp_ms > after_ms {
                event.timestamp_ms += delay_ms;
            }
        }

        self.recording.metadata.duration_ms += delay_ms;
        info!("Added {}ms delay after {}ms", delay_ms, after_ms);
    }

    /// Filter out specific event types
    pub fn filter_events(&mut self, event_types: Vec<String>) {
        self.recording.events.retain(|e| {
            let event_type = format!("{:?}", e.event).split(' ').next().unwrap_or("").to_string();
            !event_types.contains(&event_type)
        });

        self.recording.metadata.event_count = self.recording.events.len();
        info!("Filtered out {} event types", event_types.len());
    }

    /// Get edited recording
    pub fn finish(self) -> Recording {
        self.recording
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::BrowserEvent;

    #[test]
    fn test_recorder_lifecycle() {
        let mut recorder = EventRecorder::new();

        recorder.start("test".to_string(), "https://example.com".to_string());
        assert!(recorder.is_recording());

        recorder.pause();
        assert!(!recorder.is_recording());

        recorder.resume();
        assert!(recorder.is_recording());

        recorder.stop();
        assert!(!recorder.is_recording());
    }

    #[test]
    fn test_record_events() {
        let mut recorder = EventRecorder::new();
        recorder.start("test".to_string(), "https://example.com".to_string());

        recorder.record_event(BrowserEvent::Navigation { url: "https://example.com".to_string() });
        recorder.record_event(BrowserEvent::PageLoaded { url: "https://example.com".to_string() });

        let recording = recorder.stop();

        assert!(recording.is_some());
        assert_eq!(recording.unwrap().events.len(), 2);
    }

    #[test]
    fn test_player_lifecycle() {
        let mut player = EventPlayer::new();

        assert_eq!(player.get_state(), PlaybackState::Stopped);

        player.start();
        assert_eq!(player.get_state(), PlaybackState::Playing);

        player.pause();
        assert_eq!(player.get_state(), PlaybackState::Paused);

        player.resume();
        assert_eq!(player.get_state(), PlaybackState::Playing);

        player.stop();
        assert_eq!(player.get_state(), PlaybackState::Stopped);
    }

    #[test]
    fn test_player_seek() {
        let mut player = EventPlayer::new();
        let recording = Recording {
            version: RECORDING_VERSION.to_string(),
            metadata: RecordingMetadata {
                id: "test".to_string(),
                name: "test".to_string(),
                description: None,
                created_at: 0,
                modified_at: 0,
                duration_ms: 1000,
                event_count: 3,
                start_url: "https://example.com".to_string(),
                browser_config: BrowserConfig {
                    viewport_width: 1920,
                    viewport_height: 1080,
                    user_agent: None,
                    javascript_enabled: true,
                    custom: serde_json::json!({}),
                },
                tags: Vec::new(),
                custom: serde_json::json!({}),
            },
            events: vec![
                EventRecord {
                    timestamp_ms: 100,
                    event: BrowserEvent::Navigation { url: "https://example.com".to_string() },
                    metadata: None,
                },
                EventRecord {
                    timestamp_ms: 500,
                    event: BrowserEvent::PageLoaded { url: "https://example.com".to_string() },
                    metadata: None,
                },
                EventRecord {
                    timestamp_ms: 900,
                    event: BrowserEvent::TitleChanged { title: "Test".to_string() },
                    metadata: None,
                },
            ],
            snapshots: Vec::new(),
        };

        player.load_recording(recording);
        player.seek(500);
        assert_eq!(player.get_current_index(), 1);
    }

    #[test]
    fn test_editor_trim() {
        let recording = Recording {
            version: RECORDING_VERSION.to_string(),
            metadata: RecordingMetadata {
                id: "test".to_string(),
                name: "test".to_string(),
                description: None,
                created_at: 0,
                modified_at: 0,
                duration_ms: 1000,
                event_count: 3,
                start_url: "https://example.com".to_string(),
                browser_config: BrowserConfig {
                    viewport_width: 1920,
                    viewport_height: 1080,
                    user_agent: None,
                    javascript_enabled: true,
                    custom: serde_json::json!({}),
                },
                tags: Vec::new(),
                custom: serde_json::json!({}),
            },
            events: vec![
                EventRecord {
                    timestamp_ms: 100,
                    event: BrowserEvent::Navigation { url: "https://example.com".to_string() },
                    metadata: None,
                },
                EventRecord {
                    timestamp_ms: 500,
                    event: BrowserEvent::PageLoaded { url: "https://example.com".to_string() },
                    metadata: None,
                },
                EventRecord {
                    timestamp_ms: 900,
                    event: BrowserEvent::TitleChanged { title: "Test".to_string() },
                    metadata: None,
                },
            ],
            snapshots: Vec::new(),
        };

        let mut editor = RecordingEditor::new(recording);
        editor.trim(100, 600);

        let edited = editor.finish();
        assert_eq!(edited.events.len(), 2);
        assert_eq!(edited.events[0].timestamp_ms, 0);
        assert_eq!(edited.events[1].timestamp_ms, 400);
    }
}
