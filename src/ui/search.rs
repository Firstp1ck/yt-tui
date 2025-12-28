//! Search widget rendering.
//!
//! Displays the search input bar.

use crate::app::App;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// Render the search widget.
///
/// # Arguments
/// * `app` - Application state
/// * `area` - Area to render in
/// * `buf` - Buffer to render to
///
/// # Details
/// Displays a search input bar with the current query.
/// Highlights when in search mode.
pub fn render_search(app: &App, area: Rect, buf: &mut Buffer) {
    let is_active = app.mode == crate::app::UiMode::Search;
    let prompt = if is_active {
        "Search: "
    } else {
        "Search (press '/'): "
    };

    let line = Line::from(vec![
        Span::styled(prompt, Style::default().fg(Color::Yellow)),
        Span::styled(
            &app.search_query,
            Style::default().fg(if is_active { Color::White } else { Color::Gray }),
        ),
        Span::styled(
            if is_active { "_" } else { "" },
            Style::default().fg(Color::Yellow),
        ),
    ]);

    let paragraph = Paragraph::new(line).block(
        Block::default()
            .title("Search")
            .borders(Borders::ALL)
            .style(if is_active {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }),
    );

    Widget::render(paragraph, area, buf);
}
