//! YouTube Data API v3 integration module.
//!
//! Provides client for fetching recommended videos and other YouTube data.

pub mod client;
pub mod models;

pub use client::YouTubeClient;
pub use models::Video;
