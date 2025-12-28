//! History tracking for watched YouTube videos.
//!
//! Persists watched video IDs to a JSON file and provides query functionality.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// History of watched videos.
///
/// Maintains a set of watched video IDs with timestamps.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct History {
    /// Set of watched video IDs
    #[serde(default)]
    watched_videos: HashSet<String>,
    /// Map of video ID to watch timestamp (for future use)
    #[serde(default)]
    watch_timestamps: std::collections::HashMap<String, String>,
}

impl History {
    /// Load history from file.
    ///
    /// # Arguments
    /// * `path` - Path to history JSON file
    ///
    /// # Returns
    /// * `Result<History>` - Loaded history or error
    ///
    /// # Details
    /// If the file doesn't exist, returns an empty history.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read history file: {}", path.display()))?;

        let history: History =
            serde_json::from_str(&content).with_context(|| "Failed to parse history file")?;

        Ok(history)
    }

    /// Save history to file.
    ///
    /// # Arguments
    /// * `path` - Path to history JSON file
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Details
    /// Creates parent directory if it doesn't exist.
    pub fn save(&self, path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create history directory: {}", parent.display())
            })?;
        }

        let json = serde_json::to_string_pretty(self).context("Failed to serialize history")?;

        fs::write(path, json)
            .with_context(|| format!("Failed to write history file: {}", path.display()))?;

        Ok(())
    }

    /// Mark a video as watched.
    ///
    /// # Arguments
    /// * `video_id` - YouTube video ID
    ///
    /// # Details
    /// Adds the video ID to the watched set and records the current timestamp.
    pub fn mark_watched(&mut self, video_id: &str) {
        self.watched_videos.insert(video_id.to_string());
        self.watch_timestamps
            .insert(video_id.to_string(), chrono::Utc::now().to_rfc3339());
    }

    /// Check if a video is watched.
    ///
    /// # Arguments
    /// * `video_id` - YouTube video ID
    ///
    /// # Returns
    /// * `bool` - True if video is watched
    pub fn is_watched(&self, video_id: &str) -> bool {
        self.watched_videos.contains(video_id)
    }

    /// Get count of watched videos.
    ///
    /// # Returns
    /// * `usize` - Number of watched videos
    #[allow(dead_code)] // Useful for displaying statistics in UI
    pub fn watched_count(&self) -> usize {
        self.watched_videos.len()
    }

    /// Clear all history.
    ///
    /// # Details
    /// Removes all watched video entries.
    #[allow(dead_code)] // Useful for future "clear history" feature
    pub fn clear(&mut self) {
        self.watched_videos.clear();
        self.watch_timestamps.clear();
    }

    /// Remove a video from history.
    ///
    /// # Arguments
    /// * `video_id` - YouTube video ID to remove
    #[allow(dead_code)] // Useful for "unwatch" feature
    pub fn remove(&mut self, video_id: &str) {
        self.watched_videos.remove(video_id);
        self.watch_timestamps.remove(video_id);
    }

    /// Get watched videos sorted by timestamp (newest first).
    ///
    /// # Returns
    /// * `Vec<(String, String)>` - Vector of (video_id, timestamp) tuples sorted by timestamp
    ///
    /// # Details
    /// Returns all watched video IDs with their timestamps, sorted by watch time (newest first).
    pub fn get_watched_videos_sorted(&self) -> Vec<(String, String)> {
        let mut videos: Vec<(String, String)> = self
            .watch_timestamps
            .iter()
            .map(|(id, timestamp)| (id.clone(), timestamp.clone()))
            .collect();

        // Sort by timestamp (newest first)
        videos.sort_by(|a, b| {
            // Parse timestamps and compare
            let time_a = chrono::DateTime::parse_from_rfc3339(&a.1).unwrap_or_else(|_| {
                chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap()
            });
            let time_b = chrono::DateTime::parse_from_rfc3339(&b.1).unwrap_or_else(|_| {
                chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap()
            });
            time_b.cmp(&time_a) // Reverse order for newest first
        });

        videos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_history_new() {
        let history = History::default();
        assert_eq!(history.watched_count(), 0);
        assert!(!history.is_watched("test_id"));
    }

    #[test]
    fn test_history_mark_watched() {
        let mut history = History::default();
        history.mark_watched("test_id");
        assert!(history.is_watched("test_id"));
        assert_eq!(history.watched_count(), 1);
    }

    #[test]
    fn test_history_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.json");

        let mut history = History::default();
        history.mark_watched("video1");
        history.mark_watched("video2");

        history.save(&history_path).unwrap();
        assert!(history_path.exists());

        let loaded = History::load(&history_path).unwrap();
        assert_eq!(loaded.watched_count(), 2);
        assert!(loaded.is_watched("video1"));
        assert!(loaded.is_watched("video2"));
    }

    #[test]
    fn test_history_clear() {
        let mut history = History::default();
        history.mark_watched("video1");
        history.mark_watched("video2");
        assert_eq!(history.watched_count(), 2);

        history.clear();
        assert_eq!(history.watched_count(), 0);
        assert!(!history.is_watched("video1"));
    }

    #[test]
    fn test_history_remove() {
        let mut history = History::default();
        history.mark_watched("video1");
        history.mark_watched("video2");
        assert_eq!(history.watched_count(), 2);

        history.remove("video1");
        assert_eq!(history.watched_count(), 1);
        assert!(!history.is_watched("video1"));
        assert!(history.is_watched("video2"));
    }
}
