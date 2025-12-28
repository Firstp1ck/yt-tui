//! Configuration management for YouTube TUI application.
//!
//! Handles loading and saving configuration from JSONC files.
//! Manages API keys, OAuth credentials, and user preferences.

use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Application configuration structure.
///
/// Contains API credentials and user preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// YouTube Data API v3 key
    pub api_key: String,
    /// OAuth2 client ID for personalized recommendations
    pub oauth_client_id: Option<String>,
    /// OAuth2 client secret
    pub oauth_client_secret: Option<String>,
    /// OAuth2 access token (stored after authentication)
    pub oauth_access_token: Option<String>,
    /// OAuth2 refresh token
    pub oauth_refresh_token: Option<String>,
    /// Default filter settings
    pub default_filters: FilterSettings,
    /// Whether to hide watched videos by default
    pub hide_watched: bool,
    /// History file path (relative to config dir or absolute)
    pub history_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            oauth_client_id: None,
            oauth_client_secret: None,
            oauth_access_token: None,
            oauth_refresh_token: None,
            default_filters: FilterSettings::default(),
            hide_watched: false,
            history_path: "history.json".to_string(),
        }
    }
}

/// Filter settings for video filtering.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterSettings {
    /// Filter by channel name (partial match)
    pub channel: Option<String>,
    /// Filter by minimum duration in seconds
    pub min_duration: Option<u64>,
    /// Filter by maximum duration in seconds
    pub max_duration: Option<u64>,
    /// Filter by date (videos after this date)
    pub after_date: Option<String>,
}

impl Config {
    /// Load configuration from file.
    ///
    /// # Arguments
    /// * `path` - Optional path to config file. If None, uses default location.
    ///
    /// # Returns
    /// * `Result<Config>` - Loaded configuration or error
    ///
    /// # Details
    /// Searches for config file in:
    /// 1. Provided path (if given)
    /// 2. `$XDG_CONFIG_HOME/yt-tui/config.jsonc`
    /// 3. `~/.config/yt-tui/config.jsonc`
    ///
    /// If no config file exists, returns default configuration.
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let config_path = if let Some(p) = path {
            p.to_path_buf()
        } else {
            Self::default_config_path()?
        };

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        // Parse JSONC (JSON with comments)
        // Strip // style comments manually
        let json_content: String = content
            .lines()
            .map(|line| {
                // Remove // comments (but preserve // in strings)
                if let Some(comment_pos) = line.find("//") {
                    // Check if // is inside a string (simplified - doesn't handle escaped quotes)
                    let before_comment = &line[..comment_pos];
                    let quote_count = before_comment.matches('"').count();
                    if quote_count % 2 == 0 {
                        // Not inside a string, remove comment
                        line[..comment_pos].trim_end()
                    } else {
                        // Inside a string, keep as is
                        line
                    }
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let config: Config =
            serde_json::from_str(&json_content).with_context(|| "Failed to deserialize config")?;

        Ok(config)
    }

    /// Save configuration to file.
    ///
    /// # Arguments
    /// * `path` - Optional path to config file. If None, uses default location.
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Details
    /// Creates config directory if it doesn't exist.
    #[allow(dead_code)] // Useful for saving config changes from within the app
    pub fn save(&self, path: Option<&Path>) -> Result<()> {
        let config_path = if let Some(p) = path {
            p.to_path_buf()
        } else {
            Self::default_config_path()?
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let json = serde_json::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&config_path, json)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(())
    }

    /// Get default configuration file path.
    ///
    /// # Returns
    /// * `Result<PathBuf>` - Path to config file or error
    ///
    /// # Details
    /// Returns `$XDG_CONFIG_HOME/yt-tui/config.jsonc` or `~/.config/yt-tui/config.jsonc`.
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir =
            config_dir().ok_or_else(|| anyhow::anyhow!("Failed to determine config directory"))?;
        Ok(config_dir.join("yt-tui").join("config.jsonc"))
    }

    /// Get history file path.
    ///
    /// # Returns
    /// * `Result<PathBuf>` - Path to history file or error
    ///
    /// # Details
    /// If history_path is absolute, returns it as-is.
    /// Otherwise, returns path relative to config directory.
    pub fn history_file_path(&self) -> Result<PathBuf> {
        let history_path = Path::new(&self.history_path);
        if history_path.is_absolute() {
            Ok(history_path.to_path_buf())
        } else {
            let config_dir = config_dir()
                .ok_or_else(|| anyhow::anyhow!("Failed to determine config directory"))?;
            Ok(config_dir.join("yt-tui").join(&self.history_path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.api_key.is_empty());
        assert!(config.oauth_client_id.is_none());
        assert!(!config.hide_watched);
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.jsonc");

        let config = Config {
            api_key: "test_key".to_string(),
            hide_watched: true,
            ..Config::default()
        };

        config.save(Some(&config_path)).unwrap();
        assert!(config_path.exists());

        let loaded = Config::load(Some(&config_path)).unwrap();
        assert_eq!(loaded.api_key, "test_key");
        assert!(loaded.hide_watched);
    }

    #[test]
    fn test_config_jsonc_with_comments() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.jsonc");

        let jsonc_content = r#"{
            // API key for YouTube Data API v3
            "api_key": "test_key",
            "hide_watched": true
        }"#;

        fs::write(&config_path, jsonc_content).unwrap();

        let loaded = Config::load(Some(&config_path)).unwrap();
        assert_eq!(loaded.api_key, "test_key");
        assert!(loaded.hide_watched);
    }
}
