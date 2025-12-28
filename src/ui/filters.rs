//! Filters widget rendering.
//!
//! Displays filter controls and current filter settings.

use crate::app::App;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// Render the filters widget.
///
/// # Arguments
/// * `app` - Application state
/// * `area` - Area to render in
/// * `buf` - Buffer to render to
///
/// # Details
/// Displays current filter settings including:
/// - Channel filter
/// - Duration filters
/// - Date filter
/// - Hide watched toggle
pub fn render_filters(app: &App, area: Rect, buf: &mut Buffer) {
    let is_active = app.mode == crate::app::UiMode::Filters;
    let mut lines = vec![];

    // Channel filter
    if let Some(ref channel) = app.filters.channel {
        lines.push(Line::from(vec![
            Span::styled("Channel: ", Style::default().fg(Color::Cyan)),
            Span::styled(channel, Style::default().fg(Color::White)),
        ]));
    }

    // Duration filters
    if app.filters.min_duration.is_some() || app.filters.max_duration.is_some() {
        let min = app
            .filters
            .min_duration
            .map(|d| format!("{}s", d))
            .unwrap_or_else(|| "0s".to_string());
        let max = app
            .filters
            .max_duration
            .map(|d| format!("{}s", d))
            .unwrap_or_else(|| "∞".to_string());
        lines.push(Line::from(vec![
            Span::styled("Duration: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{} - {}", min, max),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    // Date filter
    if let Some(ref date) = app.filters.after_date {
        lines.push(Line::from(vec![
            Span::styled("After: ", Style::default().fg(Color::Cyan)),
            Span::styled(date, Style::default().fg(Color::White)),
        ]));
    }

    // Hide watched
    lines.push(Line::from(vec![
        Span::styled("Hide Watched: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            if app.hide_watched { "Yes" } else { "No" },
            Style::default().fg(if app.hide_watched {
                Color::Green
            } else {
                Color::Gray
            }),
        ),
    ]));

    // Sort mode
    lines.push(Line::from(vec![
        Span::styled("Sort: ", Style::default().fg(Color::Cyan)),
        Span::styled(app.sort_mode_name(), Style::default().fg(Color::Magenta)),
    ]));

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No filters active",
            Style::default().fg(Color::Gray),
        )));
    }

    // Add instruction line when active
    if is_active {
        lines.push(Line::from(Span::styled(
            "Press 'h' to toggle hide watched, 's' to change sort, 'Esc' or 'f' to exit",
            Style::default().fg(Color::Yellow),
        )));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(if is_active {
                "Filters (ACTIVE - press 'Esc' or 'f' to exit)"
            } else {
                "Filters (press 'f')"
            })
            .borders(Borders::ALL)
            .style(if is_active {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }),
    );

    Widget::render(paragraph, area, buf);
}
