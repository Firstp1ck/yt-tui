//! Application state management.
//!
//! Manages video list, selection, search, filters, and UI mode.

use crate::config::FilterSettings;
use crate::history::History;
use crate::youtube::Video;
use std::cmp;

/// Application state and UI mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    /// Normal list view
    List,
    /// Search mode
    Search,
    /// Filters mode
    Filters,
}

/// Sort mode for video list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    /// Sort by upload date (newest first)
    Date,
    /// Sort by view count (highest first)
    Views,
    /// Sort by upload date (oldest first)
    UploadDate,
    /// Sort by creator/channel name (alphabetical)
    Creator,
}

/// Tab mode for different video views.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// Current view - shows recommendations/trending videos
    CurrentView,
    /// Search tab - search YouTube platform
    Search,
    /// History tab - show watched videos
    History,
}

/// Main application state.
///
/// Manages all application data including videos, selection, search, and filters.
#[derive(Debug)]
pub struct App {
    /// All videos fetched from YouTube
    pub all_videos: Vec<Video>,
    /// Filtered videos (based on search and filters)
    pub filtered_videos: Vec<Video>,
    /// Currently selected video index (in filtered_videos)
    pub selected_index: usize,
    /// Search query string
    pub search_query: String,
    /// Current filter settings
    pub filters: FilterSettings,
    /// Current UI mode
    pub mode: UiMode,
    /// Whether to hide watched videos
    pub hide_watched: bool,
    /// History tracker
    pub history: History,
    /// Status message to display
    pub status_message: Option<String>,
    /// Current sort mode
    pub sort_mode: SortMode,
    /// Active tab
    pub active_tab: Tab,
    /// Videos from platform search
    pub search_results: Vec<Video>,
    /// Videos from watch history
    pub history_videos: Vec<Video>,
    /// Search query for platform search (separate from filter search)
    pub search_query_global: String,
    /// Pending search task handle (for non-blocking search)
    pub search_task: Option<tokio::task::JoinHandle<anyhow::Result<Vec<Video>>>>,
}

impl App {
    /// Create a new application state.
    ///
    /// # Arguments
    /// * `history` - History tracker instance
    /// * `hide_watched` - Whether to hide watched videos by default
    ///
    /// # Returns
    /// * `App` - New application state
    pub fn new(history: History, hide_watched: bool) -> Self {
        Self {
            all_videos: Vec::new(),
            filtered_videos: Vec::new(),
            selected_index: 0,
            search_query: String::new(),
            filters: FilterSettings::default(),
            mode: UiMode::List,
            hide_watched,
            history,
            status_message: None,
            sort_mode: SortMode::Date,
            active_tab: Tab::CurrentView,
            search_results: Vec::new(),
            history_videos: Vec::new(),
            search_query_global: String::new(),
            search_task: None,
        }
    }

    /// Set the list of videos and apply current filters.
    ///
    /// # Arguments
    /// * `videos` - List of videos to set
    ///
    /// # Details
    /// Replaces all videos and immediately applies current search and filters.
    pub fn set_videos(&mut self, videos: Vec<Video>) {
        self.all_videos = videos;
        self.apply_filters();
    }

    /// Apply current search query and filters to video list.
    ///
    /// # Details
    /// Filters videos based on:
    /// - Search query (title, channel, description)
    /// - Channel filter
    /// - Duration filters
    /// - Date filter
    /// - Hide watched option
    ///
    /// Only applies when on CurrentView tab.
    pub fn apply_filters(&mut self) {
        // Only apply filters when on CurrentView tab
        if self.active_tab != Tab::CurrentView {
            return;
        }
        let mut filtered: Vec<Video> = self.all_videos.clone();

        // Apply search query
        if !self.search_query.is_empty() {
            let query_lower = self.search_query.to_lowercase();
            filtered.retain(|video| {
                video.title.to_lowercase().contains(&query_lower)
                    || video.channel.to_lowercase().contains(&query_lower)
                    || video.description.to_lowercase().contains(&query_lower)
            });
        }

        // Apply channel filter
        if let Some(ref channel) = self.filters.channel {
            let channel_lower = channel.to_lowercase();
            filtered.retain(|video| video.channel.to_lowercase().contains(&channel_lower));
        }

        // Apply duration filters
        if let Some(min_duration) = self.filters.min_duration {
            filtered.retain(|video| video.duration >= min_duration);
        }
        if let Some(max_duration) = self.filters.max_duration {
            filtered.retain(|video| video.duration <= max_duration);
        }

        // Apply date filter
        if let Some(ref after_date) = self.filters.after_date
            && let Ok(filter_date) = chrono::DateTime::parse_from_rfc3339(after_date)
        {
            let filter_date_utc = filter_date.with_timezone(&chrono::Utc);
            filtered.retain(|video| video.published_at >= filter_date_utc);
        }

        // Apply hide watched filter
        if self.hide_watched {
            filtered.retain(|video| !self.history.is_watched(&video.id));
        }

        // Apply sorting
        self.apply_sorting(&mut filtered);

        self.filtered_videos = filtered;
        self.selected_index = cmp::min(
            self.selected_index,
            self.filtered_videos.len().saturating_sub(1),
        );
    }

    /// Apply current sort mode to video list.
    ///
    /// # Arguments
    /// * `videos` - Mutable reference to video list to sort
    ///
    /// # Details
    /// Sorts videos in-place based on current sort_mode.
    fn apply_sorting(&self, videos: &mut [Video]) {
        match self.sort_mode {
            SortMode::Date => {
                // Sort by upload date (newest first)
                videos.sort_by(|a, b| b.published_at.cmp(&a.published_at));
            }
            SortMode::Views => {
                // Sort by view count (highest first)
                videos.sort_by(|a, b| b.view_count.cmp(&a.view_count));
            }
            SortMode::UploadDate => {
                // Sort by upload date (oldest first)
                videos.sort_by(|a, b| a.published_at.cmp(&b.published_at));
            }
            SortMode::Creator => {
                // Sort by creator/channel name (alphabetical)
                videos.sort_by(|a, b| a.channel.cmp(&b.channel));
            }
        }
    }

    /// Cycle to next sort mode.
    ///
    /// # Details
    /// Cycles through sort modes: Date -> Views -> UploadDate -> Creator -> Date
    /// Reapplies filters after changing sort mode.
    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Date => SortMode::Views,
            SortMode::Views => SortMode::UploadDate,
            SortMode::UploadDate => SortMode::Creator,
            SortMode::Creator => SortMode::Date,
        };
        self.apply_filters();
    }

    /// Get current sort mode as a string.
    ///
    /// # Returns
    /// * `&str` - Sort mode name
    pub fn sort_mode_name(&self) -> &str {
        match self.sort_mode {
            SortMode::Date => "Date (newest)",
            SortMode::Views => "Views (highest)",
            SortMode::UploadDate => "Upload Date (oldest)",
            SortMode::Creator => "Creator (A-Z)",
        }
    }

    /// Get the currently selected video.
    ///
    /// # Returns
    /// * `Option<&Video>` - Selected video or None if list is empty
    ///
    /// # Deprecated
    /// Use `selected_video_from_tab()` instead for tab-aware selection.
    /// Move selection up.
    ///
    /// # Details
    /// Decrements selected index, wrapping to bottom if at top.
    /// Updates scroll offset to keep selection centered.
    /// Works with the current tab's video list.
    pub fn move_up(&mut self) {
        let list = self.get_current_video_list();
        if list.is_empty() {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = list.len() - 1;
        } else {
            self.selected_index -= 1;
        }
        self.update_scroll_offset();
    }

    /// Move selection down.
    ///
    /// # Details
    /// Increments selected index, wrapping to top if at bottom.
    /// Updates scroll offset to keep selection centered.
    /// Works with the current tab's video list.
    pub fn move_down(&mut self) {
        let list = self.get_current_video_list();
        if list.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % list.len();
        self.update_scroll_offset();
    }

    /// Update scroll offset to keep selection centered in the view.
    ///
    /// # Details
    /// Calculates the scroll offset needed to center the selected item.
    /// This is called automatically when selection changes.
    /// The actual scroll offset is calculated in the render function based on available height.
    #[allow(dead_code)] // Scroll offset is calculated in render function, not stored
    pub fn update_scroll_offset(&mut self) {
        // Scroll offset is calculated dynamically in the render function
        // based on the actual available height, so we don't need to store it
    }

    /// Add a character to the search query.
    ///
    /// # Arguments
    /// * `ch` - Character to add
    ///
    /// # Details
    /// Only works in Search mode. Applies filters after adding character.
    pub fn add_search_char(&mut self, ch: char) {
        if self.mode == UiMode::Search {
            self.search_query.push(ch);
            self.apply_filters();
        }
    }

    /// Remove last character from search query.
    ///
    /// # Details
    /// Only works in Search mode. Applies filters after removing character.
    pub fn remove_search_char(&mut self) {
        if self.mode == UiMode::Search {
            self.search_query.pop();
            self.apply_filters();
        }
    }

    /// Clear search query.
    ///
    /// # Details
    /// Clears search and applies filters.
    #[allow(dead_code)] // Useful for future UI features (e.g., clear button)
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.apply_filters();
    }

    /// Toggle hide watched filter.
    ///
    /// # Details
    /// Toggles the hide_watched flag and reapplies filters.
    pub fn toggle_hide_watched(&mut self) {
        self.hide_watched = !self.hide_watched;
        self.apply_filters();
    }

    /// Set status message.
    ///
    /// # Arguments
    /// * `message` - Status message to display
    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Clear status message.
    #[allow(dead_code)] // Useful for auto-clearing status messages after timeout
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Mark selected video as watched.
    ///
    /// # Details
    /// Marks the currently selected video as watched in history.
    /// Works with videos from any tab (CurrentView, Search, History).
    /// If hide_watched is enabled, the video will be removed from the list.
    pub fn mark_selected_watched(&mut self) {
        if let Some(video) = self.selected_video_from_tab() {
            let video_id = video.id.clone();
            self.history.mark_watched(&video_id);
            if self.hide_watched && self.active_tab == Tab::CurrentView {
                self.apply_filters();
            }
        }
    }

    /// Switch to a different tab.
    ///
    /// # Arguments
    /// * `tab` - Tab to switch to
    ///
    /// # Details
    /// Switches the active tab and resets selected index.
    pub fn switch_tab(&mut self, tab: Tab) {
        self.active_tab = tab;
        self.selected_index = 0;
    }

    /// Get the currently active tab.
    ///
    /// # Returns
    /// * `Tab` - Current active tab
    pub fn active_tab(&self) -> Tab {
        self.active_tab
    }

    /// Get the current video list based on active tab.
    ///
    /// # Returns
    /// * `&Vec<Video>` - Reference to the appropriate video list
    ///
    /// # Details
    /// Returns the video list for the currently active tab:
    /// - CurrentView: filtered_videos
    /// - Search: search_results
    /// - History: history_videos
    pub fn get_current_video_list(&self) -> &Vec<Video> {
        match self.active_tab {
            Tab::CurrentView => &self.filtered_videos,
            Tab::Search => &self.search_results,
            Tab::History => &self.history_videos,
        }
    }

    /// Set search results from platform search.
    ///
    /// # Arguments
    /// * `videos` - Videos from search
    ///
    /// # Details
    /// Stores search results and resets selected index.
    pub fn set_search_results(&mut self, videos: Vec<Video>) {
        self.search_results = videos;
        self.selected_index = 0;
    }

    /// Set history videos.
    ///
    /// # Arguments
    /// * `videos` - Videos from history
    ///
    /// # Details
    /// Stores history videos and resets selected index.
    pub fn set_history_videos(&mut self, videos: Vec<Video>) {
        self.history_videos = videos;
        self.selected_index = 0;
    }

    /// Get the currently selected video from the active tab's list.
    ///
    /// # Returns
    /// * `Option<&Video>` - Selected video or None if list is empty
    ///
    /// # Details
    /// Returns the selected video from the appropriate list based on active tab.
    pub fn selected_video_from_tab(&self) -> Option<&Video> {
        let list = self.get_current_video_list();
        list.get(self.selected_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::youtube::Video;
    use chrono::Utc;

    fn create_test_video(id: &str, title: &str, channel: &str) -> Video {
        Video::new(
            id.to_string(),
            title.to_string(),
            channel.to_string(),
            "channel_id".to_string(),
            "Description".to_string(),
            100,
            Utc::now(),
            "thumb".to_string(),
            1000,
        )
    }

    #[test]
    fn test_app_new() {
        let history = History::default();
        let app = App::new(history, false);
        assert_eq!(app.all_videos.len(), 0);
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.mode, UiMode::List);
    }

    #[test]
    fn test_app_set_videos() {
        let history = History::default();
        let mut app = App::new(history, false);
        let videos = vec![
            create_test_video("1", "Video 1", "Channel 1"),
            create_test_video("2", "Video 2", "Channel 2"),
        ];
        app.set_videos(videos);
        assert_eq!(app.all_videos.len(), 2);
        assert_eq!(app.filtered_videos.len(), 2);
    }

    #[test]
    fn test_app_search_filter() {
        let history = History::default();
        let mut app = App::new(history, false);
        let videos = vec![
            create_test_video("1", "Rust Tutorial", "Channel 1"),
            create_test_video("2", "Python Guide", "Channel 2"),
        ];
        app.set_videos(videos);
        app.mode = UiMode::Search;
        app.search_query = "Rust".to_string();
        app.apply_filters();
        assert_eq!(app.filtered_videos.len(), 1);
        assert_eq!(app.filtered_videos[0].title, "Rust Tutorial");
    }

    #[test]
    fn test_app_move_selection() {
        let history = History::default();
        let mut app = App::new(history, false);
        let videos = vec![
            create_test_video("1", "Video 1", "Channel 1"),
            create_test_video("2", "Video 2", "Channel 2"),
            create_test_video("3", "Video 3", "Channel 3"),
        ];
        app.set_videos(videos);
        assert_eq!(app.selected_index, 0);

        app.move_down();
        assert_eq!(app.selected_index, 1);

        app.move_up();
        assert_eq!(app.selected_index, 0);

        app.move_up(); // Should wrap to end
        assert_eq!(app.selected_index, 2);
    }

    #[test]
    fn test_app_hide_watched() {
        let mut history = History::default();
        history.mark_watched("1");
        let mut app = App::new(history, true);
        let videos = vec![
            create_test_video("1", "Video 1", "Channel 1"),
            create_test_video("2", "Video 2", "Channel 2"),
        ];
        app.set_videos(videos);
        assert_eq!(app.filtered_videos.len(), 1);
        assert_eq!(app.filtered_videos[0].id, "2");
    }
}
