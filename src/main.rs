//! YouTube TUI - Terminal User Interface for browsing YouTube recommendations.
//!
//! Main entry point and event loop for the application.

mod app;
mod config;
mod history;
mod player;
mod ui;
mod youtube;

use app::{App, UiMode};
use config::Config;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use history::History;
use player::open_in_mpv;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;
use youtube::YouTubeClient;

/// Main application entry point.
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Details
/// Initializes terminal, loads configuration, fetches videos, and runs the event loop.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load(None)?;

    if config.api_key.is_empty() {
        eprintln!("Error: YouTube API key is required.");
        eprintln!(
            "Please create a config file at: {}",
            Config::default_config_path()?.display()
        );
        eprintln!("See config.jsonc.example for template.");
        return Err(anyhow::anyhow!("API key not configured"));
    }

    // Load history
    let history_path = config.history_file_path()?;
    let history = History::load(&history_path)?;

    // Create YouTube client
    let youtube_client = YouTubeClient::new(&config)?;

    // Create application state
    let mut app = App::new(history, config.hide_watched);

    // Fetch videos
    app.set_status("Fetching recommended videos...".to_string());
    match youtube_client.fetch_recommended_videos(50).await {
        Ok(videos) => {
            if videos.is_empty() {
                app.set_status(
                    "Warning: No videos found. Check your API key permissions.".to_string(),
                );
            } else {
                app.set_videos(videos);
                app.set_status(format!("Loaded {} videos", app.all_videos.len()));
            }
        }
        Err(e) => {
            let error_msg = format!("Error fetching videos: {}", e);
            eprintln!("{}", error_msg);
            app.set_status(error_msg);
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run event loop
    let result = run_app(&mut terminal, &mut app, &config).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Render the complete UI.
///
/// # Arguments
/// * `f` - Frame to render to
/// * `app` - Application state
///
/// # Details
/// Lays out and renders all UI components including list, search, filters, and status.
fn render_ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(3), // Search bar
            ratatui::layout::Constraint::Length(6), // Filters
            ratatui::layout::Constraint::Length(3), // Tabs
            ratatui::layout::Constraint::Min(0),    // Video list
            ratatui::layout::Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    // Render search
    ui::render_search(app, chunks[0], f.buffer_mut());

    // Render filters
    ui::render_filters(app, chunks[1], f.buffer_mut());

    // Render tabs
    ui::render_tabs(app, chunks[2], f.buffer_mut());

    // Render video list
    ui::render_list(app, chunks[3], f.buffer_mut());

    // Render status bar
    let status_text = app
        .status_message
        .as_deref()
        .unwrap_or("Press 'q' to quit, '/' to search, 'f' for filters, 'h' to toggle hide watched, 's' to change sort, '1/2/3' or Tab to switch tabs");
    let status = ratatui::widgets::Paragraph::new(ratatui::text::Line::from(status_text))
        .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL));
    f.render_widget(status, chunks[4]);
}

/// Main event loop.
///
/// # Arguments
/// * `terminal` - Terminal instance
/// * `app` - Application state
/// * `config` - Configuration
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Details
/// Handles keyboard and mouse events, updates state, and renders UI.
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    config: &Config,
) -> anyhow::Result<()> {
    // Calculate video list area boundaries (will be updated on each render)
    let mut list_area = ratatui::layout::Rect::default();
    let mut tabs_area = ratatui::layout::Rect::default();

    // Create YouTube client for async operations
    let youtube_client = YouTubeClient::new(config)?;

    loop {
        terminal.draw(|f| {
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Length(3), // Search bar
                    ratatui::layout::Constraint::Length(6), // Filters
                    ratatui::layout::Constraint::Length(3), // Tabs
                    ratatui::layout::Constraint::Min(0),    // Video list
                    ratatui::layout::Constraint::Length(1), // Status bar
                ])
                .split(f.area());
            list_area = chunks[3]; // Store list area for mouse click detection (updated index)
            tabs_area = chunks[2]; // Store tabs area for mouse click detection
            render_ui(f, app);
        })?;

        // Check for completed search task after rendering (non-blocking)
        if let Some(task) = &mut app.search_task
            && task.is_finished()
            && let Some(handle) = app.search_task.take()
        {
            match handle.await {
                Ok(Ok(videos)) => {
                    app.set_search_results(videos);
                    app.set_status(format!("Found {} videos", app.search_results.len()));
                }
                Ok(Err(e)) => {
                    app.set_status(format!("Search failed: {}", e));
                }
                Err(e) => {
                    app.set_status(format!("Search task error: {}", e));
                }
            }
        }

        // Use non-blocking event polling with timeout to keep UI responsive
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match app.mode {
                        UiMode::List => {
                            // Handle Search tab input when on Search tab
                            if app.active_tab() == crate::app::Tab::Search {
                                match key.code {
                                    KeyCode::Enter => {
                                        // Alt+Enter: Play selected video
                                        // Note: Ctrl+Enter is not reliably detected by most terminals
                                        if key.modifiers.contains(event::KeyModifiers::ALT) {
                                            if let Some(video) = app.selected_video_from_tab() {
                                                let video_url = video.url.clone();
                                                let video_title = video.title.clone();
                                                match open_in_mpv(&video_url) {
                                                    Ok(()) => {
                                                        app.mark_selected_watched();
                                                        let history_path =
                                                            config.history_file_path()?;
                                                        if let Err(e) =
                                                            app.history.save(&history_path)
                                                        {
                                                            app.set_status(format!(
                                                                "Failed to save history: {}",
                                                                e
                                                            ));
                                                        } else {
                                                            app.set_status(format!(
                                                                "Opened: {}",
                                                                video_title
                                                            ));
                                                        }
                                                    }
                                                    Err(e) => {
                                                        app.set_status(format!(
                                                            "Failed to open video: {}",
                                                            e
                                                        ));
                                                    }
                                                }
                                            }
                                            // Skip the rest of the event processing
                                            continue;
                                        }

                                        // Regular Enter (without Alt): Perform search
                                        if !app.search_query_global.is_empty()
                                            && app.search_task.is_none()
                                        {
                                            app.set_status("Searching YouTube...".to_string());
                                            let query = app.search_query_global.clone();
                                            let client = youtube_client.clone();
                                            app.search_task = Some(tokio::spawn(async move {
                                                client.search_videos(&query, 50).await
                                            }));
                                        }
                                        // Skip the rest of the event processing for regular Enter too
                                        continue;
                                    }
                                    KeyCode::Backspace => {
                                        app.search_query_global.pop();
                                    }
                                    KeyCode::Char(c) => {
                                        app.search_query_global.push(c);
                                    }
                                    _ => {}
                                }
                            }
                            // Handle normal list navigation
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Char('Q') => break,
                                KeyCode::Esc => break,
                                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                                KeyCode::Enter => {
                                    // Only handle Enter if not on Search tab (Search tab handles it above)
                                    if app.active_tab() != crate::app::Tab::Search
                                        && let Some(video) = app.selected_video_from_tab()
                                    {
                                        let video_url = video.url.clone();
                                        let video_title = video.title.clone();
                                        match open_in_mpv(&video_url) {
                                            Ok(()) => {
                                                app.mark_selected_watched();
                                                let history_path = config.history_file_path()?;
                                                if let Err(e) = app.history.save(&history_path) {
                                                    app.set_status(format!(
                                                        "Failed to save history: {}",
                                                        e
                                                    ));
                                                } else {
                                                    app.set_status(format!(
                                                        "Opened: {}",
                                                        video_title
                                                    ));
                                                }
                                            }
                                            Err(e) => {
                                                app.set_status(format!(
                                                    "Failed to open video: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char('/') => {
                                    app.mode = UiMode::Search;
                                }
                                KeyCode::Char('f')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.mode = UiMode::Filters;
                                }
                                KeyCode::Char('h') => {
                                    app.toggle_hide_watched();
                                }
                                KeyCode::Char('s') => {
                                    app.cycle_sort_mode();
                                    app.set_status(format!("Sort: {}", app.sort_mode_name()));
                                }
                                KeyCode::Char('1') => {
                                    handle_tab_switch(
                                        app,
                                        &youtube_client,
                                        config,
                                        crate::app::Tab::CurrentView,
                                    )
                                    .await?;
                                }
                                KeyCode::Char('2') => {
                                    handle_tab_switch(
                                        app,
                                        &youtube_client,
                                        config,
                                        crate::app::Tab::Search,
                                    )
                                    .await?;
                                }
                                KeyCode::Char('3') => {
                                    handle_tab_switch(
                                        app,
                                        &youtube_client,
                                        config,
                                        crate::app::Tab::History,
                                    )
                                    .await?;
                                }
                                KeyCode::Tab => {
                                    // Cycle to next tab (forward)
                                    let next_tab = match app.active_tab() {
                                        crate::app::Tab::CurrentView => crate::app::Tab::Search,
                                        crate::app::Tab::Search => crate::app::Tab::History,
                                        crate::app::Tab::History => crate::app::Tab::CurrentView,
                                    };
                                    handle_tab_switch(app, &youtube_client, config, next_tab)
                                        .await?;
                                }
                                KeyCode::BackTab => {
                                    // Cycle to previous tab (backward, Shift+Tab)
                                    let prev_tab = match app.active_tab() {
                                        crate::app::Tab::CurrentView => crate::app::Tab::History,
                                        crate::app::Tab::Search => crate::app::Tab::CurrentView,
                                        crate::app::Tab::History => crate::app::Tab::Search,
                                    };
                                    handle_tab_switch(app, &youtube_client, config, prev_tab)
                                        .await?;
                                }
                                KeyCode::Char('c')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    break;
                                }
                                _ => {}
                            }
                        }
                        UiMode::Search => match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.mode = UiMode::List;
                            }
                            KeyCode::Backspace => {
                                app.remove_search_char();
                            }
                            KeyCode::Char(c) => {
                                app.add_search_char(c);
                            }
                            _ => {}
                        },
                        UiMode::Filters => match key.code {
                            KeyCode::Esc | KeyCode::Char('f') => {
                                app.mode = UiMode::List;
                            }
                            KeyCode::Char('h') => {
                                app.toggle_hide_watched();
                            }
                            KeyCode::Char('s') => {
                                app.cycle_sort_mode();
                                app.set_status(format!("Sort: {}", app.sort_mode_name()));
                            }
                            // Filter editing would go here in a more complete implementation
                            _ => {}
                        },
                    }
                }
                Event::Mouse(mouse) => {
                    handle_mouse_event(mouse, app, config, list_area, tabs_area, &youtube_client)
                        .await?;
                }
                _ => {}
            }
        }
        // If no event, continue loop to redraw UI (keeps it responsive)
    }

    Ok(())
}

/// Handle tab switching with data fetching.
///
/// # Arguments
/// * `app` - Application state
/// * `youtube_client` - YouTube API client
/// * `config` - Configuration
/// * `tab` - Tab to switch to
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Details
/// Switches to the specified tab and fetches data if needed.
async fn handle_tab_switch(
    app: &mut App,
    youtube_client: &YouTubeClient,
    _config: &Config,
    tab: crate::app::Tab,
) -> anyhow::Result<()> {
    app.switch_tab(tab);

    match tab {
        crate::app::Tab::Search => {
            // If search results are empty and we have a query, start search in background
            if app.search_results.is_empty()
                && !app.search_query_global.is_empty()
                && app.search_task.is_none()
            {
                app.set_status("Searching YouTube...".to_string());
                let query = app.search_query_global.clone();
                let client = youtube_client.clone();
                app.search_task = Some(tokio::spawn(async move {
                    client.search_videos(&query, 50).await
                }));
            }
        }
        crate::app::Tab::History => {
            // Fetch history videos if not already loaded
            if app.history_videos.is_empty() {
                app.set_status("Loading watch history...".to_string());
                let watched_videos = app.history.get_watched_videos_sorted();
                if !watched_videos.is_empty() {
                    let video_ids: Vec<String> =
                        watched_videos.iter().map(|(id, _)| id.clone()).collect();
                    match youtube_client.fetch_history_videos(&video_ids).await {
                        Ok(mut videos) => {
                            // Sort by watch timestamp (newest first)
                            // Create a map for quick lookup
                            let timestamp_map: std::collections::HashMap<String, String> =
                                watched_videos.into_iter().collect();
                            videos.sort_by(|a, b| {
                                let time_a = timestamp_map
                                    .get(&a.id)
                                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                                    .unwrap_or_else(|| {
                                        chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z")
                                            .unwrap()
                                    });
                                let time_b = timestamp_map
                                    .get(&b.id)
                                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                                    .unwrap_or_else(|| {
                                        chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z")
                                            .unwrap()
                                    });
                                time_b.cmp(&time_a) // Reverse for newest first
                            });
                            app.set_history_videos(videos);
                            app.set_status(format!(
                                "Loaded {} watched videos",
                                app.history_videos.len()
                            ));
                        }
                        Err(e) => {
                            app.set_status(format!("Failed to load history: {}", e));
                        }
                    }
                } else {
                    app.set_status("No watch history".to_string());
                }
            }
        }
        crate::app::Tab::CurrentView => {
            // No action needed, already using filtered_videos
        }
    }

    Ok(())
}

/// Handle mouse events (scroll and click).
///
/// # Arguments
/// * `mouse` - Mouse event
/// * `app` - Application state
/// * `config` - Configuration
/// * `list_area` - Area of the video list widget
/// * `tabs_area` - Area of the tabs widget
/// * `youtube_client` - YouTube API client
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Details
/// Handles mouse scroll for navigation, left click to play videos, and tab clicking.
async fn handle_mouse_event(
    mouse: MouseEvent,
    app: &mut App,
    config: &Config,
    list_area: ratatui::layout::Rect,
    tabs_area: ratatui::layout::Rect,
    youtube_client: &YouTubeClient,
) -> anyhow::Result<()> {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            if app.mode == UiMode::List {
                app.move_up();
            }
        }
        MouseEventKind::ScrollDown => {
            if app.mode == UiMode::List {
                app.move_down();
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            // Check if click is on tabs area
            if mouse.column >= tabs_area.x
                && mouse.column < tabs_area.x + tabs_area.width
                && mouse.row >= tabs_area.y
                && mouse.row < tabs_area.y + tabs_area.height
            {
                // Calculate which tab was clicked
                // Tabs are roughly: "Current View" (14 chars) | "Search" (6 chars) | "History" (7 chars)
                // Approximate positions
                let tab_width = tabs_area.width / 3;
                let clicked_tab = if mouse.column < tabs_area.x + tab_width {
                    crate::app::Tab::CurrentView
                } else if mouse.column < tabs_area.x + tab_width * 2 {
                    crate::app::Tab::Search
                } else {
                    crate::app::Tab::History
                };
                handle_tab_switch(app, youtube_client, config, clicked_tab).await?;
            }
            // Check if click is within the video list area
            // Account for list widget borders (1 line for top border)
            else if app.mode == UiMode::List
                && mouse.column >= list_area.x
                && mouse.column < list_area.x + list_area.width
                && mouse.row > list_area.y // Skip top border
                && mouse.row < list_area.y + list_area.height
            {
                // Calculate which video was clicked
                // Each video takes 6 lines (1 for title + 4 for info + 1 separator)
                // Account for the top border (1 line)
                let lines_per_video = 6;
                let click_y = mouse.row - list_area.y - 1; // Subtract border
                let video_index = (click_y / lines_per_video) as usize;

                let current_list = app.get_current_video_list();
                if video_index < current_list.len() {
                    // Set selection to clicked video
                    app.selected_index = video_index;

                    // Play the video
                    if let Some(video) = app.selected_video_from_tab() {
                        let video_url = video.url.clone();
                        let video_title = video.title.clone();
                        match open_in_mpv(&video_url) {
                            Ok(()) => {
                                app.mark_selected_watched();
                                let history_path = config.history_file_path()?;
                                if let Err(e) = app.history.save(&history_path) {
                                    app.set_status(format!("Failed to save history: {}", e));
                                } else {
                                    app.set_status(format!("Opened: {}", video_title));
                                }
                            }
                            Err(e) => {
                                app.set_status(format!("Failed to open video: {}", e));
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}
