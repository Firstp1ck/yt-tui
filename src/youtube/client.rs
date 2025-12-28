//! YouTube Data API v3 client implementation.
//!
//! Handles authentication and fetching of recommended videos.

use crate::config::Config;
use crate::youtube::models::{ApiActivityItem, ApiResponse, ApiVideoItem, Video};
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

/// YouTube Data API v3 client.
///
/// Handles API requests for fetching recommended videos.
#[derive(Debug, Clone)]
pub struct YouTubeClient {
    /// HTTP client for API requests
    client: Client,
    /// API key for authentication
    api_key: String,
    /// OAuth2 access token (if available)
    access_token: Option<String>,
    /// Base URL for YouTube Data API
    base_url: String,
}

impl YouTubeClient {
    /// Create a new YouTube client from configuration.
    ///
    /// # Arguments
    /// * `config` - Application configuration
    ///
    /// # Returns
    /// * `Result<YouTubeClient>` - New client or error
    ///
    /// # Details
    /// Requires at least an API key. OAuth2 tokens are optional but needed
    /// for personalized recommendations.
    pub fn new(config: &Config) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(anyhow::anyhow!(
                "YouTube API key is required. Please set it in config.jsonc"
            ));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_key: config.api_key.clone(),
            access_token: config.oauth_access_token.clone(),
            base_url: "https://www.googleapis.com/youtube/v3".to_string(),
        })
    }

    /// Fetch recommended videos from YouTube.
    ///
    /// # Arguments
    /// * `max_results` - Maximum number of videos to fetch (default: 50)
    ///
    /// # Returns
    /// * `Result<Vec<Video>>` - List of recommended videos or error
    ///
    /// # Details
    /// Uses the activities.list endpoint with home channel to get recommendations.
    /// If OAuth2 is available, uses authenticated requests for personalized recommendations.
    /// Otherwise, falls back to search.list with trending videos.
    pub async fn fetch_recommended_videos(&self, max_results: u32) -> Result<Vec<Video>> {
        // Try to get personalized recommendations if OAuth is available
        if self.access_token.is_some()
            && let Ok(videos) = self.fetch_personalized_recommendations(max_results).await
        {
            return Ok(videos);
        }

        // Fallback to trending videos
        self.fetch_trending_videos(max_results).await
    }

    /// Fetch personalized recommendations using activities.list.
    ///
    /// # Arguments
    /// * `max_results` - Maximum number of videos to fetch
    ///
    /// # Returns
    /// * `Result<Vec<Video>>` - List of recommended videos or error
    ///
    /// # Details
    /// Requires OAuth2 authentication. Fetches activities from "home" channel.
    async fn fetch_personalized_recommendations(&self, max_results: u32) -> Result<Vec<Video>> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("OAuth access token required for personalized recommendations")
        })?;

        let url = format!("{}/activities", self.base_url);
        let mut videos = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut params = vec![
                ("part", "snippet,contentDetails"),
                ("home", "true"),
                ("maxResults", "50"),
            ];

            if let Some(token) = &page_token {
                params.push(("pageToken", token));
            }

            let response = self
                .client
                .get(&url)
                .bearer_auth(access_token)
                .query(&params)
                .send()
                .await
                .context("Failed to fetch activities from YouTube API")?;

            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "YouTube API error ({}): {}",
                    status,
                    error_text
                ));
            }

            let api_response: ApiResponse<ApiActivityItem> = response
                .json()
                .await
                .context("Failed to parse activities response")?;

            // Extract video IDs from recommendations
            let video_ids: Vec<String> = api_response
                .items
                .iter()
                .filter_map(|activity| {
                    activity
                        .snippet
                        .content_details
                        .as_ref()?
                        .recommendation
                        .as_ref()?
                        .resource_id
                        .video_id
                        .clone()
                        .into()
                })
                .collect();

            if video_ids.is_empty() {
                break;
            }

            // Fetch video details
            let video_details = self.fetch_video_details(&video_ids).await?;
            videos.extend(video_details);

            if videos.len() >= max_results as usize {
                videos.truncate(max_results as usize);
                break;
            }

            page_token = api_response.next_page_token;
            if page_token.is_none() {
                break;
            }
        }

        Ok(videos)
    }

    /// Fetch trending videos using videos.list with chart parameter.
    ///
    /// # Arguments
    /// * `max_results` - Maximum number of videos to fetch
    ///
    /// # Returns
    /// * `Result<Vec<Video>>` - List of trending videos or error
    ///
    /// # Details
    /// Uses public API key. Fetches trending videos from YouTube using the videos.list
    /// endpoint with chart=mostPopular. This directly returns video details, so no
    /// separate fetch_video_details call is needed.
    async fn fetch_trending_videos(&self, max_results: u32) -> Result<Vec<Video>> {
        let url = format!("{}/videos", self.base_url);
        let params = [
            ("part", "snippet,contentDetails,statistics"),
            ("chart", "mostPopular"),
            ("maxResults", &max_results.to_string()),
            ("key", &self.api_key),
        ];

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to fetch trending videos from YouTube API")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "YouTube API error ({}): {}",
                status,
                error_text
            ));
        }

        let api_response: ApiResponse<ApiVideoItem> = response
            .json()
            .await
            .context("Failed to parse trending videos response")?;

        let mut videos = Vec::new();
        for item in api_response.items {
            match Video::try_from(item) {
                Ok(video) => videos.push(video),
                Err(e) => {
                    eprintln!("Failed to parse video: {}", e);
                    // Continue with other videos
                }
            }
        }

        Ok(videos)
    }

    /// Fetch detailed information for a list of video IDs.
    ///
    /// # Arguments
    /// * `video_ids` - List of YouTube video IDs
    ///
    /// # Returns
    /// * `Result<Vec<Video>>` - List of video details or error
    ///
    /// # Details
    /// Uses videos.list endpoint to get full video details including duration and statistics.
    async fn fetch_video_details(&self, video_ids: &[String]) -> Result<Vec<Video>> {
        // YouTube API limits to 50 IDs per request
        let chunk_size = 50;
        let mut all_videos = Vec::new();

        for chunk in video_ids.chunks(chunk_size) {
            let ids = chunk.join(",");
            let url = format!("{}/videos", self.base_url);
            let params = [
                ("part", "snippet,contentDetails,statistics"),
                ("id", &ids),
                ("key", &self.api_key),
            ];

            let response = self
                .client
                .get(&url)
                .query(&params)
                .send()
                .await
                .context("Failed to fetch video details from YouTube API")?;

            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "YouTube API error ({}): {}",
                    status,
                    error_text
                ));
            }

            let api_response: ApiResponse<ApiVideoItem> = response
                .json()
                .await
                .context("Failed to parse video details response")?;

            for item in api_response.items {
                match Video::try_from(item) {
                    Ok(video) => all_videos.push(video),
                    Err(e) => {
                        eprintln!("Failed to parse video: {}", e);
                        // Continue with other videos
                    }
                }
            }
        }

        Ok(all_videos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_client_new_requires_api_key() {
        let config = Config::default();
        assert!(YouTubeClient::new(&config).is_err());
    }

    #[test]
    fn test_client_new_with_api_key() {
        let config = Config {
            api_key: "test_key".to_string(),
            ..Config::default()
        };
        assert!(YouTubeClient::new(&config).is_ok());
    }
}
