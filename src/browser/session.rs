use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// A single saved tab entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedTab {
    pub url: String,
    pub title: String,
}

/// The full session state written to / read from disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Tabs in order.
    pub tabs: Vec<SavedTab>,
    /// Index into `tabs` of the active tab at save time.
    pub active_index: usize,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            tabs: Vec::new(),
            active_index: 0,
        }
    }
}

/// Returns the default path for the session file: `~/.tinker/session.json`.
/// Falls back to `./session.json` if the home directory cannot be determined.
pub fn default_session_path() -> PathBuf {
    let base = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join(".tinker").join("session.json")
}

/// Save session to the given path, creating parent directories as needed.
pub fn save_session(session: &Session, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(session)?;
    fs::write(path, json)?;
    info!("Session saved: {} tab(s) to {}", session.tabs.len(), path.display());
    Ok(())
}

/// Load session from the given path. Returns `None` if the file does not exist.
pub fn load_session(path: &Path) -> Option<Session> {
    if !path.exists() {
        debug!("No session file at {}", path.display());
        return None;
    }
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<Session>(&contents) {
            Ok(session) => {
                info!("Session loaded: {} tab(s) from {}", session.tabs.len(), path.display());
                Some(session)
            }
            Err(e) => {
                warn!("Failed to parse session file: {}", e);
                None
            }
        },
        Err(e) => {
            warn!("Failed to read session file: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = env::temp_dir().join("tinker_session_test");
        let path = dir.join("session.json");

        let session = Session {
            tabs: vec![
                SavedTab { url: "https://example.com".to_string(), title: "Example".to_string() },
                SavedTab { url: "https://rust-lang.org".to_string(), title: "Rust".to_string() },
            ],
            active_index: 1,
        };

        save_session(&session, &path).expect("save failed");
        let loaded = load_session(&path).expect("load returned None");

        assert_eq!(loaded.tabs.len(), 2);
        assert_eq!(loaded.tabs[0].url, "https://example.com");
        assert_eq!(loaded.tabs[1].title, "Rust");
        assert_eq!(loaded.active_index, 1);

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_load_missing_file_returns_none() {
        let path = PathBuf::from("/nonexistent/path/session.json");
        assert!(load_session(&path).is_none());
    }

    #[test]
    fn test_default_session_path_has_correct_suffix() {
        let path = default_session_path();
        assert!(path.ends_with(".tinker/session.json") || path.ends_with(".tinker\\session.json"));
    }
}
