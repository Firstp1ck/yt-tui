//! YouTube Data API v3 models and data structures.
//!
//! Contains structures for representing videos, channels, and API responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a YouTube video.
///
/// Contains all relevant information about a video for display and playback.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Video {
    /// YouTube video ID
    pub id: String,
    /// Video title
    pub title: String,
    /// Channel name
    pub channel: String,
    /// Channel ID
    pub channel_id: String,
    /// Video description (truncated)
    pub description: String,
    /// Video duration in seconds
    pub duration: u64,
    /// Video published date
    pub published_at: DateTime<Utc>,
    /// Video thumbnail URL
    pub thumbnail_url: String,
    /// View count
    pub view_count: u64,
    /// YouTube video URL
    pub url: String,
}

impl Video {
    /// Create a new Video instance.
    ///
    /// # Arguments
    /// * `id` - YouTube video ID
    /// * `title` - Video title
    /// * `channel` - Channel name
    /// * `channel_id` - Channel ID
    /// * `description` - Video description
    /// * `duration` - Duration in seconds
    /// * `published_at` - Published date
    /// * `thumbnail_url` - Thumbnail URL
    /// * `view_count` - View count
    ///
    /// # Returns
    /// * `Video` - New video instance
    ///
    /// # Details
    /// Automatically generates the YouTube URL from the video ID.
    #[allow(clippy::too_many_arguments)] // Constructor requires all video fields
    pub fn new(
        id: String,
        title: String,
        channel: String,
        channel_id: String,
        description: String,
        duration: u64,
        published_at: DateTime<Utc>,
        thumbnail_url: String,
        view_count: u64,
    ) -> Self {
        let url = format!("https://www.youtube.com/watch?v={}", id);
        Self {
            id,
            title,
            channel,
            channel_id,
            description,
            duration,
            published_at,
            thumbnail_url,
            view_count,
            url,
        }
    }

    /// Format duration as HH:MM:SS or MM:SS.
    ///
    /// # Returns
    /// * `String` - Formatted duration string
    pub fn format_duration(&self) -> String {
        let hours = self.duration / 3600;
        let minutes = (self.duration % 3600) / 60;
        let seconds = self.duration % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }

    /// Format view count with K/M suffixes.
    ///
    /// # Returns
    /// * `String` - Formatted view count
    pub fn format_views(&self) -> String {
        if self.view_count >= 1_000_000 {
            format!("{:.1}M", self.view_count as f64 / 1_000_000.0)
        } else if self.view_count >= 1_000 {
            format!("{:.1}K", self.view_count as f64 / 1_000.0)
        } else {
            self.view_count.to_string()
        }
    }

    /// Format published date as a readable string.
    ///
    /// # Returns
    /// * `String` - Formatted date string (e.g., "Mo. 15.01.2024")
    pub fn format_date(&self) -> String {
        self.published_at.format("%a. %d.%m.%Y").to_string()
    }
}

/// YouTube API search/list response wrapper.
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    /// Response items
    pub items: Vec<T>,
    /// Next page token for pagination
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

/// YouTube API video item (from activities.list or search.list).
#[derive(Debug, Deserialize)]
pub struct ApiVideoItem {
    /// Video ID
    pub id: String,
    /// Snippet containing video details
    pub snippet: ApiSnippet,
    /// Content details (duration, etc.)
    #[serde(rename = "contentDetails")]
    pub content_details: Option<ApiContentDetails>,
    /// Statistics (view count, etc.)
    pub statistics: Option<ApiStatistics>,
}

/// Video snippet from API response.
#[derive(Debug, Deserialize)]
pub struct ApiSnippet {
    /// Video title
    pub title: String,
    /// Channel title
    #[serde(rename = "channelTitle")]
    pub channel_title: String,
    /// Channel ID
    #[serde(rename = "channelId")]
    pub channel_id: String,
    /// Video description
    pub description: String,
    /// Published date
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    /// Thumbnails
    pub thumbnails: ApiThumbnails,
}

/// Thumbnail information.
#[derive(Debug, Deserialize)]
pub struct ApiThumbnails {
    /// Default thumbnail
    #[serde(default)]
    pub default: Option<ApiThumbnail>,
    /// Medium thumbnail
    #[serde(default)]
    pub medium: Option<ApiThumbnail>,
    /// High thumbnail
    #[serde(default)]
    pub high: Option<ApiThumbnail>,
}

/// Single thumbnail.
#[derive(Debug, Deserialize)]
pub struct ApiThumbnail {
    /// Thumbnail URL
    pub url: String,
}

/// Content details (duration, etc.).
#[derive(Debug, Deserialize)]
pub struct ApiContentDetails {
    /// Video duration in ISO 8601 format (PT4M13S)
    pub duration: Option<String>,
}

/// Video statistics.
#[derive(Debug, Deserialize)]
pub struct ApiStatistics {
    /// View count
    #[serde(rename = "viewCount")]
    pub view_count: Option<String>,
}

/// Activity item from activities.list (for recommendations).
#[derive(Debug, Deserialize)]
pub struct ApiActivityItem {
    /// Activity ID
    #[allow(dead_code)] // Part of API response structure, may be used for debugging
    pub id: String,
    /// Snippet
    pub snippet: ApiActivitySnippet,
}

/// Activity snippet.
#[derive(Debug, Deserialize)]
pub struct ApiActivitySnippet {
    /// Published date
    #[serde(rename = "publishedAt")]
    #[allow(dead_code)] // Part of API response structure, may be used for filtering/sorting
    pub published_at: String,
    /// Activity type
    #[serde(rename = "type")]
    #[allow(dead_code)]
    // Part of API response structure, may be used for filtering activity types
    pub activity_type: String,
    /// Video details (if type is "recommendation")
    pub content_details: Option<ApiActivityContentDetails>,
}

/// Activity content details.
#[derive(Debug, Deserialize)]
pub struct ApiActivityContentDetails {
    /// Recommendation details
    pub recommendation: Option<ApiRecommendationDetails>,
}

/// Recommendation details.
#[derive(Debug, Deserialize)]
pub struct ApiRecommendationDetails {
    /// Resource ID of recommended video
    #[serde(rename = "resourceId")]
    pub resource_id: ApiResourceId,
    /// Reason for recommendation
    #[allow(dead_code)]
    // Part of API response structure, may be used for displaying recommendation reasons
    pub reason: Option<String>,
}

/// Resource ID (video ID).
#[derive(Debug, Deserialize)]
pub struct ApiResourceId {
    /// Video ID
    #[serde(rename = "videoId")]
    pub video_id: String,
}

impl TryFrom<ApiVideoItem> for Video {
    type Error = anyhow::Error;

    /// Convert API video item to Video.
    ///
    /// # Arguments
    /// * `item` - API video item
    ///
    /// # Returns
    /// * `Result<Video>` - Converted video or error
    ///
    /// # Details
    /// Parses duration from ISO 8601 format (PT4M13S) to seconds.
    fn try_from(item: ApiVideoItem) -> Result<Self, Self::Error> {
        let duration = item
            .content_details
            .and_then(|cd| cd.duration)
            .map(parse_duration)
            .transpose()?
            .unwrap_or(0);

        let view_count = item
            .statistics
            .and_then(|s| s.view_count)
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let published_at = DateTime::parse_from_rfc3339(&item.snippet.published_at)
            .map_err(|e| anyhow::anyhow!("Failed to parse published date: {}", e))?
            .with_timezone(&Utc);

        let thumbnail_url = item
            .snippet
            .thumbnails
            .high
            .or(item.snippet.thumbnails.medium)
            .or(item.snippet.thumbnails.default)
            .map(|t| t.url)
            .unwrap_or_default();

        Ok(Video::new(
            item.id,
            item.snippet.title,
            item.snippet.channel_title,
            item.snippet.channel_id,
            item.snippet.description,
            duration,
            published_at,
            thumbnail_url,
            view_count,
        ))
    }
}

/// Parse ISO 8601 duration (PT4M13S) to seconds.
///
/// # Arguments
/// * `duration` - ISO 8601 duration string
///
/// # Returns
/// * `Result<u64>` - Duration in seconds or error
fn parse_duration(duration: String) -> anyhow::Result<u64> {
    // Format: PT4M13S (Period Time, 4 Minutes, 13 Seconds)
    let mut seconds = 0u64;
    let mut current_num = String::new();

    for ch in duration.chars() {
        match ch {
            'P' | 'T' => continue,
            'H' => {
                seconds += current_num.parse::<u64>()? * 3600;
                current_num.clear();
            }
            'M' => {
                seconds += current_num.parse::<u64>()? * 60;
                current_num.clear();
            }
            'S' => {
                seconds += current_num.parse::<u64>()?;
                current_num.clear();
            }
            c if c.is_ascii_digit() => current_num.push(c),
            _ => return Err(anyhow::anyhow!("Invalid duration format: {}", duration)),
        }
    }

    Ok(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("PT4M13S".to_string()).unwrap(), 253);
        assert_eq!(parse_duration("PT1H30M".to_string()).unwrap(), 5400);
        assert_eq!(parse_duration("PT30S".to_string()).unwrap(), 30);
        assert_eq!(parse_duration("PT2H15M30S".to_string()).unwrap(), 8130);
    }

    #[test]
    fn test_video_format_duration() {
        let video = Video::new(
            "test".to_string(),
            "Test".to_string(),
            "Channel".to_string(),
            "channel_id".to_string(),
            "Description".to_string(),
            253,
            Utc::now(),
            "thumb".to_string(),
            1000,
        );
        assert_eq!(video.format_duration(), "04:13");

        let video_long = Video::new(
            "test".to_string(),
            "Test".to_string(),
            "Channel".to_string(),
            "channel_id".to_string(),
            "Description".to_string(),
            8130,
            Utc::now(),
            "thumb".to_string(),
            1000,
        );
        assert_eq!(video_long.format_duration(), "02:15:30");
    }

    #[test]
    fn test_video_format_views() {
        let video = Video::new(
            "test".to_string(),
            "Test".to_string(),
            "Channel".to_string(),
            "channel_id".to_string(),
            "Description".to_string(),
            100,
            Utc::now(),
            "thumb".to_string(),
            1500,
        );
        assert_eq!(video.format_views(), "1.5K");

        let video_m = Video::new(
            "test".to_string(),
            "Test".to_string(),
            "Channel".to_string(),
            "channel_id".to_string(),
            "Description".to_string(),
            100,
            Utc::now(),
            "thumb".to_string(),
            2_500_000,
        );
        assert_eq!(video_m.format_views(), "2.5M");
    }
}
