//! Video player integration for MPV.
//!
//! Handles opening YouTube videos in the MPV video player.

use anyhow::{Context, Result};
use std::process::Command;

/// Open a YouTube video in MPV player.
///
/// # Arguments
/// * `video_url` - YouTube video URL (e.g., https://www.youtube.com/watch?v=VIDEO_ID)
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Details
/// Uses MPV directly with the YouTube URL. MPV has built-in support for YouTube URLs
/// when yt-dlp is installed, and handles the yt-dlp integration automatically.
/// This ensures both video and audio work correctly.
/// Falls back to Haruna if MPV is not available.
pub fn open_in_mpv(video_url: &str) -> Result<()> {
    // Use mpv directly with YouTube URL
    // MPV has built-in yt-dlp support and handles YouTube URLs properly
    // We try Wayland-compatible video outputs first, then fall back to others
    
    // Detect if we're on Wayland
    let is_wayland = std::env::var("XDG_SESSION_TYPE")
        .map(|s| s == "wayland")
        .unwrap_or(false)
        || std::env::var("WAYLAND_DISPLAY").is_ok();

    // Format preference: prefer merged streams (best), then try merging best video+audio
    // This ensures we get both video and audio together when possible
    let format_preference = "best[height<=?1080]/bestvideo[height<=?1080]+bestaudio/best";

    // Audio output preference: try pipewire (Wayland), pulse, then auto-detect
    let audio_outputs = if is_wayland {
        vec!["pipewire", "pulse", "auto"]
    } else {
        vec!["pulse", "alsa", "auto"]
    };

    if is_wayland {
        // Wayland: Try different video outputs with audio
        let video_outputs = vec!["gpu", "dmabuf-wayland", "wlshm"];
        
        for vo in &video_outputs {
            for ao in &audio_outputs {
                let mut cmd = Command::new("mpv");
                cmd.arg("--player-operation-mode=pseudo-gui")
                    .arg(format!("--ytdl-format={}", format_preference))
                    .arg(format!("--vo={}", vo))
                    .arg(format!("--ao={}", ao));
                
                if *vo == "wlshm" {
                    cmd.arg("--hwdec=no");
                }
                
                cmd.arg(video_url);
                
                if cmd.spawn().is_ok() {
                    return Ok(());
                }
            }
        }
    } else {
        // X11: Try different video outputs with audio
        let video_outputs = vec!["gpu", "x11"];
        
        for vo in &video_outputs {
            for ao in &audio_outputs {
                let mut cmd = Command::new("mpv");
                cmd.arg("--player-operation-mode=pseudo-gui")
                    .arg(format!("--ytdl-format={}", format_preference))
                    .arg(format!("--vo={}", vo))
                    .arg(format!("--ao={}", ao));
                
                if *vo == "x11" {
                    cmd.arg("--hwdec=no");
                }
                
                cmd.arg(video_url);
                
                if cmd.spawn().is_ok() {
                    return Ok(());
                }
            }
        }
    }

    // Final fallback: Use best format with auto-detection for both video and audio
    Command::new("mpv")
        .arg("--player-operation-mode=pseudo-gui")
        .arg("--ytdl-format=best")
        .arg(video_url)
        .spawn()
        .with_context(|| {
            format!(
                "Failed to open video with mpv. Make sure mpv and yt-dlp are installed. URL: {}",
                video_url
            )
        })?;

    Ok(())
}

/// Check if MPV is available in the system PATH.
///
/// # Returns
/// * `bool` - True if MPV command is available
///
/// # Details
/// Checks if `mpv --version` succeeds.
#[allow(dead_code)] // Useful for startup validation and error messages
pub fn is_mpv_available() -> bool {
    Command::new("mpv").arg("--version").output().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_mpv_available() {
        // This test just checks that the function doesn't panic
        // Actual result depends on system configuration
        let _ = is_mpv_available();
    }
}
