//! Video list widget rendering.
//!
//! Displays a scrollable list of videos with selection highlighting.

use crate::app::App;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

/// Render the video list widget.
///
/// # Arguments
/// * `app` - Application state
/// * `area` - Area to render in
/// * `buf` - Buffer to render to
///
/// # Details
/// Displays a scrollable list of videos with each video on multiple lines:
/// - Line 1: Video title and watched indicator (bold)
/// - Line 2: Creator/channel name
/// - Line 3: Video duration
/// - Line 4: Upload date
/// - Line 5: View count
///
/// Highlights the selected video.
pub fn render_list(app: &App, area: Rect, buf: &mut Buffer) {
    // Get the current video list based on active tab
    let current_list = app.get_current_video_list();
    let total_count = match app.active_tab() {
        crate::app::Tab::CurrentView => app.all_videos.len(),
        crate::app::Tab::Search => current_list.len(),
        crate::app::Tab::History => current_list.len(),
    };

    // Handle empty list
    if current_list.is_empty() {
        let title = format!("Videos ({}/{})", current_list.len(), total_count);
        let list = List::new(vec![ListItem::new("No videos to display")])
            .block(Block::default().title(title).borders(Borders::ALL));
        Widget::render(list, area, buf);
        return;
    }

    // Ensure selected_index is valid
    let selected_index = app.selected_index.min(current_list.len().saturating_sub(1));

    // Calculate separator width (accounting for borders)
    let separator_width = area.width.saturating_sub(2).max(10) as usize;
    let separator_line = "─".repeat(separator_width);

    // Calculate scroll offset to keep selection centered
    // Each video takes 6 lines (1 for title + 4 for info + 1 separator)
    let lines_per_video = 6;
    let available_height = area.height.saturating_sub(2); // Account for borders
    let visible_videos = (available_height / lines_per_video).max(1) as usize;
    let center_offset = (visible_videos / 2).max(0);

    // Calculate scroll offset to center the selected item
    let scroll_offset = if selected_index >= center_offset {
        selected_index.saturating_sub(center_offset)
    } else {
        0
    };

    // Ensure we don't scroll past the end
    let max_scroll = current_list.len().saturating_sub(visible_videos);
    let scroll_offset = scroll_offset.min(max_scroll);

    // Only render visible items based on scroll offset
    let start_idx = scroll_offset;
    let end_idx = (scroll_offset + visible_videos).min(current_list.len());

    let items: Vec<ListItem> = current_list
        .iter()
        .enumerate()
        .skip(start_idx)
        .take(end_idx - start_idx)
        .map(|(idx, video)| {
            // idx is the absolute index in filtered_videos (enumerate preserves original index)
            let is_selected = idx == selected_index;
            let is_watched = app.history.is_watched(&video.id);

            let base_style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let title_style = Style::default()
                .fg(if is_selected {
                    Color::Yellow
                } else {
                    Color::White
                })
                .add_modifier(Modifier::BOLD); // Always bold for title

            // Line 1: Video title (bold, single line)
            let mut line1_spans = vec![Span::styled(&video.title, title_style)];
            if is_watched {
                line1_spans.push(Span::styled(
                    " [WATCHED]",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            let line1 = Line::from(line1_spans);

            // Line 2: Creator/channel
            let line2 = Line::from(vec![Span::styled(
                format!("Creator: {}", video.channel),
                Style::default().fg(Color::Cyan),
            )]);

            // Line 3: Video duration
            let line3 = Line::from(vec![Span::styled(
                format!("Duration: {}", video.format_duration()),
                Style::default().fg(Color::Magenta),
            )]);

            // Line 4: Upload date
            let line4 = Line::from(vec![Span::styled(
                format!("Uploaded: {}", video.format_date()),
                Style::default().fg(Color::Yellow),
            )]);

            // Line 5: Views
            let line5 = Line::from(vec![Span::styled(
                format!("Views: {}", video.format_views()),
                Style::default().fg(Color::Gray),
            )]);

            // Line 6: Separator (dashed line)
            let separator_style = if is_selected {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let separator = Line::from(vec![Span::styled(separator_line.clone(), separator_style)]);

            // Create ListItem with 6 lines (5 content + 1 separator: title takes 1 line)
            ListItem::new(vec![line1, line2, line3, line4, line5, separator]).style(base_style)
        })
        .collect();

    let title = format!("Videos ({}/{})", current_list.len(), total_count);

    // Calculate relative selected index for visible items
    let relative_selected = if selected_index >= scroll_offset
        && selected_index < scroll_offset + items.len()
        && !items.is_empty()
    {
        Some(selected_index - scroll_offset)
    } else {
        None
    };

    let mut list_state = ListState::default();
    list_state.select(relative_selected);

    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );

    StatefulWidget::render(list, area, buf, &mut list_state);
}
