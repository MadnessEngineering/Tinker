//! Browser console monitoring and logging
//!
//! This module provides comprehensive console tracking including:
//! - Console output capture (log, info, warn, error, debug)
//! - JavaScript error and unhandled rejection tracking
//! - Real-time console message streaming
//! - Message filtering by level
//! - Circular buffer for memory management

use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Console log levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleLevel {
    Log,
    Info,
    Warn,
    Error,
    Debug,
}

impl ConsoleLevel {
    /// Parse console level from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "log" => Some(ConsoleLevel::Log),
            "info" => Some(ConsoleLevel::Info),
            "warn" => Some(ConsoleLevel::Warn),
            "error" => Some(ConsoleLevel::Error),
            "debug" => Some(ConsoleLevel::Debug),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            ConsoleLevel::Log => "log",
            ConsoleLevel::Info => "info",
            ConsoleLevel::Warn => "warn",
            ConsoleLevel::Error => "error",
            ConsoleLevel::Debug => "debug",
        }
    }
}

/// Source location information for console messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleSource {
    pub url: String,
    pub line: u32,
    pub column: u32,
}

/// Console message with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    /// Log level (log, info, warn, error, debug)
    pub level: ConsoleLevel,

    /// The formatted message string
    pub message: String,

    /// Timestamp in milliseconds since epoch
    pub timestamp: u64,

    /// Stack trace (for errors)
    pub stack_trace: Option<String>,

    /// Source location where console was called
    pub source: Option<ConsoleSource>,

    /// Raw arguments passed to console method
    pub args: Vec<serde_json::Value>,
}

impl ConsoleMessage {
    pub fn new(
        level: ConsoleLevel,
        message: String,
        args: Vec<serde_json::Value>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            level,
            message,
            timestamp,
            stack_trace: None,
            source: None,
            args,
        }
    }
}

/// JavaScript error captured from window.onerror or unhandledrejection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaScriptError {
    /// Error message
    pub message: String,

    /// Full stack trace
    pub stack: String,

    /// Source filename where error occurred
    pub filename: String,

    /// Line number
    pub line: u32,

    /// Column number
    pub column: u32,

    /// Timestamp in milliseconds since epoch
    pub timestamp: u64,

    /// Error type (Error, TypeError, UnhandledRejection, etc.)
    pub error_type: String,
}

impl JavaScriptError {
    pub fn new(
        message: String,
        stack: String,
        filename: String,
        line: u32,
        column: u32,
        error_type: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            message,
            stack,
            filename,
            line,
            column,
            timestamp,
            error_type,
        }
    }
}

/// Console monitor with circular buffer management
pub struct ConsoleMonitor {
    /// Whether monitoring is currently active
    monitoring: bool,

    /// Circular buffer of console messages
    messages: Vec<ConsoleMessage>,

    /// Circular buffer of JavaScript errors
    errors: Vec<JavaScriptError>,

    /// Maximum messages to keep in buffer
    max_messages: usize,

    /// Optional filter level (only keep messages at or above this level)
    filter_level: Option<ConsoleLevel>,
}

impl ConsoleMonitor {
    /// Create a new console monitor with default buffer size (1000 messages)
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create a new console monitor with custom buffer size
    pub fn with_capacity(max_messages: usize) -> Self {
        info!("Creating console monitor with capacity: {}", max_messages);
        Self {
            monitoring: false,
            messages: Vec::with_capacity(max_messages),
            errors: Vec::new(),
            max_messages,
            filter_level: None,
        }
    }

    /// Start console monitoring
    pub fn start_monitoring(&mut self) {
        info!("Starting console monitoring");
        self.monitoring = true;
    }

    /// Stop console monitoring
    pub fn stop_monitoring(&mut self) {
        info!("Stopping console monitoring");
        self.monitoring = false;
    }

    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        self.monitoring
    }

    /// Add a console message to the buffer
    pub fn add_message(&mut self, message: ConsoleMessage) {
        // Check filter level
        if let Some(ref filter_level) = self.filter_level {
            // Skip messages below filter level (simple ordinal comparison)
            match (&message.level, filter_level) {
                (ConsoleLevel::Debug, ConsoleLevel::Log | ConsoleLevel::Info | ConsoleLevel::Warn | ConsoleLevel::Error) => return,
                (ConsoleLevel::Log, ConsoleLevel::Info | ConsoleLevel::Warn | ConsoleLevel::Error) => return,
                (ConsoleLevel::Info, ConsoleLevel::Warn | ConsoleLevel::Error) => return,
                (ConsoleLevel::Warn, ConsoleLevel::Error) => return,
                _ => {}
            }
        }

        debug!("Adding console message: {} - {}", message.level.as_str(), message.message);

        // Implement circular buffer
        if self.messages.len() >= self.max_messages {
            self.messages.remove(0);
        }

        self.messages.push(message);
    }

    /// Add a JavaScript error to the buffer
    pub fn add_error(&mut self, error: JavaScriptError) {
        warn!("Adding JavaScript error: {} at {}:{}:{}",
              error.message, error.filename, error.line, error.column);

        // Implement circular buffer
        if self.errors.len() >= self.max_messages {
            self.errors.remove(0);
        }

        self.errors.push(error);
    }

    /// Get all console messages, optionally filtered by level
    pub fn get_messages(&self, level: Option<ConsoleLevel>) -> Vec<ConsoleMessage> {
        if let Some(filter_level) = level {
            self.messages
                .iter()
                .filter(|msg| msg.level == filter_level)
                .cloned()
                .collect()
        } else {
            self.messages.clone()
        }
    }

    /// Get all JavaScript errors
    pub fn get_errors(&self) -> Vec<JavaScriptError> {
        self.errors.clone()
    }

    /// Clear all console messages
    pub fn clear(&mut self) {
        info!("Clearing console buffer");
        self.messages.clear();
        self.errors.clear();
    }

    /// Set filter level (messages below this level won't be stored)
    pub fn set_filter(&mut self, level: Option<ConsoleLevel>) {
        match &level {
            Some(l) => info!("Setting console filter level to: {}", l.as_str()),
            None => info!("Removing console filter level"),
        }
        self.filter_level = level;
    }

    /// Get current filter level
    pub fn get_filter(&self) -> Option<ConsoleLevel> {
        self.filter_level.clone()
    }

    /// Get current message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get current error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Generate JavaScript injection script to intercept console methods
    pub fn generate_injection_script(&self) -> String {
        // Include the console injection template
        include_str!("../../templates/console_inject.js").to_string()
    }
}

impl Default for ConsoleMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_monitor_creation() {
        let monitor = ConsoleMonitor::new();
        assert!(!monitor.is_monitoring());
        assert_eq!(monitor.message_count(), 0);
        assert_eq!(monitor.error_count(), 0);
    }

    #[test]
    fn test_start_stop_monitoring() {
        let mut monitor = ConsoleMonitor::new();
        assert!(!monitor.is_monitoring());

        monitor.start_monitoring();
        assert!(monitor.is_monitoring());

        monitor.stop_monitoring();
        assert!(!monitor.is_monitoring());
    }

    #[test]
    fn test_add_message() {
        let mut monitor = ConsoleMonitor::new();
        let message = ConsoleMessage::new(
            ConsoleLevel::Log,
            "Test message".to_string(),
            vec![],
        );

        monitor.add_message(message);
        assert_eq!(monitor.message_count(), 1);
    }

    #[test]
    fn test_circular_buffer() {
        let mut monitor = ConsoleMonitor::with_capacity(3);

        // Add 4 messages
        for i in 0..4 {
            let message = ConsoleMessage::new(
                ConsoleLevel::Log,
                format!("Message {}", i),
                vec![],
            );
            monitor.add_message(message);
        }

        // Should only have 3 messages (capacity)
        assert_eq!(monitor.message_count(), 3);

        // First message should be removed
        let messages = monitor.get_messages(None);
        assert_eq!(messages[0].message, "Message 1");
    }

    #[test]
    fn test_filter_by_level() {
        let mut monitor = ConsoleMonitor::new();

        monitor.add_message(ConsoleMessage::new(
            ConsoleLevel::Log,
            "Log message".to_string(),
            vec![],
        ));
        monitor.add_message(ConsoleMessage::new(
            ConsoleLevel::Error,
            "Error message".to_string(),
            vec![],
        ));

        let errors = monitor.get_messages(Some(ConsoleLevel::Error));
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Error message");
    }

    #[test]
    fn test_set_filter_level() {
        let mut monitor = ConsoleMonitor::new();
        monitor.set_filter(Some(ConsoleLevel::Warn));

        // Add messages at different levels
        monitor.add_message(ConsoleMessage::new(
            ConsoleLevel::Log,
            "Log message".to_string(),
            vec![],
        ));
        monitor.add_message(ConsoleMessage::new(
            ConsoleLevel::Warn,
            "Warn message".to_string(),
            vec![],
        ));
        monitor.add_message(ConsoleMessage::new(
            ConsoleLevel::Error,
            "Error message".to_string(),
            vec![],
        ));

        // Only warn and error should be stored
        assert_eq!(monitor.message_count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut monitor = ConsoleMonitor::new();

        monitor.add_message(ConsoleMessage::new(
            ConsoleLevel::Log,
            "Message".to_string(),
            vec![],
        ));
        monitor.add_error(JavaScriptError::new(
            "Error".to_string(),
            "Stack".to_string(),
            "file.js".to_string(),
            10,
            5,
            "Error".to_string(),
        ));

        assert_eq!(monitor.message_count(), 1);
        assert_eq!(monitor.error_count(), 1);

        monitor.clear();

        assert_eq!(monitor.message_count(), 0);
        assert_eq!(monitor.error_count(), 0);
    }

    #[test]
    fn test_console_level_from_str() {
        assert_eq!(ConsoleLevel::from_str("log"), Some(ConsoleLevel::Log));
        assert_eq!(ConsoleLevel::from_str("ERROR"), Some(ConsoleLevel::Error));
        assert_eq!(ConsoleLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_add_javascript_error() {
        let mut monitor = ConsoleMonitor::new();

        let error = JavaScriptError::new(
            "Test error".to_string(),
            "Stack trace".to_string(),
            "app.js".to_string(),
            42,
            10,
            "TypeError".to_string(),
        );

        monitor.add_error(error);
        assert_eq!(monitor.error_count(), 1);

        let errors = monitor.get_errors();
        assert_eq!(errors[0].message, "Test error");
        assert_eq!(errors[0].filename, "app.js");
        assert_eq!(errors[0].line, 42);
    }
}
