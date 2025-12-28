//! Tabs widget rendering.
//!
//! Displays tab headers for switching between different video views.

use crate::app::{App, Tab};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// Render the tabs widget.
///
/// # Arguments
/// * `app` - Application state
/// * `area` - Area to render in
/// * `buf` - Buffer to render to
///
/// # Details
/// Displays three tabs horizontally:
/// - Current View
/// - Search
/// - History
///
/// Highlights the active tab with different styling.
pub fn render_tabs(app: &App, area: Rect, buf: &mut Buffer) {
    let active_tab = app.active_tab();

    // Create tab labels
    let tabs = [
        ("Current View", Tab::CurrentView),
        ("Search", Tab::Search),
        ("History", Tab::History),
    ];

    // Build tab line with separators
    let mut spans = Vec::new();
    for (i, (label, tab)) in tabs.iter().enumerate() {
        let is_active = *tab == active_tab;
        let style = if is_active {
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        // Add separator before each tab except the first
        if i > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        }

        // Add tab label
        let tab_text = if is_active {
            format!("▶ {} ◀", label)
        } else {
            format!("  {}  ", label)
        };
        spans.push(Span::styled(tab_text, style));
    }

    let line = Line::from(spans);

    let paragraph = Paragraph::new(line)
        .block(Block::default().title("Tabs").borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);

    Widget::render(paragraph, area, buf);
}
