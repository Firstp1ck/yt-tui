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
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton, MouseEvent, MouseEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::time::Duration;
use history::History;
use player::open_in_mpv;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
};
use std::io;
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
                app.set_status("Warning: No videos found. Check your API key permissions.".to_string());
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
            ratatui::layout::Constraint::Min(0),    // Video list
            ratatui::layout::Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    // Render search
    ui::render_search(app, chunks[0], f.buffer_mut());

    // Render filters
    ui::render_filters(app, chunks[1], f.buffer_mut());

    // Render video list
    ui::render_list(app, chunks[2], f.buffer_mut());

    // Render status bar
    let status_text = app
        .status_message
        .as_deref()
        .unwrap_or("Press 'q' to quit, '/' to search, 'f' for filters, 'h' to toggle hide watched, 's' to change sort");
    let status = ratatui::widgets::Paragraph::new(ratatui::text::Line::from(status_text))
        .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL));
    f.render_widget(status, chunks[3]);
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

    loop {
        terminal.draw(|f| {
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Length(3), // Search bar
                    ratatui::layout::Constraint::Length(6), // Filters
                    ratatui::layout::Constraint::Min(0),    // Video list
                    ratatui::layout::Constraint::Length(1), // Status bar
                ])
                .split(f.area());
            list_area = chunks[2]; // Store list area for mouse click detection
            render_ui(f, app);
        })?;

        // Use non-blocking event polling with timeout to keep UI responsive
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match app.mode {
                UiMode::List => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('Q') => break,
                    KeyCode::Esc => break,
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::Enter => {
                        if let Some(video) = app.selected_video() {
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
                    KeyCode::Char('/') => {
                        app.mode = UiMode::Search;
                    }
                    KeyCode::Char('f') => {
                        app.mode = UiMode::Filters;
                    }
                    KeyCode::Char('h') => {
                        app.toggle_hide_watched();
                    }
                    KeyCode::Char('s') => {
                        app.cycle_sort_mode();
                        app.set_status(format!("Sort: {}", app.sort_mode_name()));
                    }
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        break;
                    }
                    _ => {}
                },
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
                handle_mouse_event(mouse, app, config, list_area)?;
            }
            _ => {}
            }
        }
        // If no event, continue loop to redraw UI (keeps it responsive)
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
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Details
/// Handles mouse scroll for navigation and left click to play videos.
fn handle_mouse_event(
    mouse: MouseEvent,
    app: &mut App,
    config: &Config,
    list_area: ratatui::layout::Rect,
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
            // Check if click is within the video list area
            // Account for list widget borders (1 line for top border)
            if app.mode == UiMode::List
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

                if video_index < app.filtered_videos.len() {
                    // Set selection to clicked video
                    app.selected_index = video_index;

                    // Play the video
                    if let Some(video) = app.selected_video() {
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
