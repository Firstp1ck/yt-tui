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
/// Highlights when in search mode or on Search tab.
/// Shows search_query_global when on Search tab, otherwise shows search_query.
pub fn render_search(app: &App, area: Rect, buf: &mut Buffer) {
    let is_active = app.mode == crate::app::UiMode::Search;
    let is_search_tab = app.active_tab() == crate::app::Tab::Search;
    let prompt = if is_active || is_search_tab {
        "Search: "
    } else {
        "Search (press '/'): "
    };

    // Show search_query_global when on Search tab, otherwise show search_query
    let query = if is_search_tab {
        &app.search_query_global
    } else {
        &app.search_query
    };

    let line = Line::from(vec![
        Span::styled(prompt, Style::default().fg(Color::Yellow)),
        Span::styled(
            query,
            Style::default().fg(if is_active || is_search_tab {
                Color::White
            } else {
                Color::Gray
            }),
        ),
        Span::styled(
            if is_active || is_search_tab { "_" } else { "" },
            Style::default().fg(Color::Yellow),
        ),
    ]);

    let paragraph = Paragraph::new(line).block(
        Block::default()
            .title("Search")
            .borders(Borders::ALL)
            .style(if is_active || is_search_tab {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }),
    );

    Widget::render(paragraph, area, buf);
}
