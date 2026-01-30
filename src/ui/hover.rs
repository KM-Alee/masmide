//! Hover documentation popup rendering

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::docs::DocEntry;
use crate::theme::Theme;

/// Render the hover documentation popup
pub fn render(frame: &mut Frame, doc: &DocEntry, cursor_screen_pos: (u16, u16), theme: &Theme) {
    let area = frame.area();

    // Build content lines
    let mut lines: Vec<Line> = Vec::new();

    // Title/Name with syntax
    lines.push(Line::from(vec![Span::styled(
        doc.syntax,
        Style::default()
            .fg(theme.syntax.keyword.to_color())
            .add_modifier(Modifier::BOLD),
    )]));

    // Blank line
    lines.push(Line::from(""));

    // Description - word wrap manually
    let desc_words: Vec<&str> = doc.description.split_whitespace().collect();
    let max_line_len = 50;
    let mut current_line = String::new();

    for word in desc_words {
        if current_line.len() + word.len() + 1 > max_line_len && !current_line.is_empty() {
            lines.push(Line::from(Span::styled(
                current_line.clone(),
                Style::default().fg(theme.ui.foreground.to_color()),
            )));
            current_line = word.to_string();
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        lines.push(Line::from(Span::styled(
            current_line,
            Style::default().fg(theme.ui.foreground.to_color()),
        )));
    }

    // Example if present
    if let Some(example) = doc.example {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Example:",
            Style::default()
                .fg(theme.ui.foreground.to_color())
                .add_modifier(Modifier::BOLD),
        )));

        for example_line in example.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {}", example_line),
                Style::default().fg(theme.syntax.comment.to_color()),
            )));
        }
    }

    // Calculate popup dimensions
    let content_width = lines.iter().map(|l| l.width()).max().unwrap_or(20) as u16;

    let popup_width = (content_width + 4).clamp(30, 60); // +4 for borders and padding
    let popup_height = (lines.len() as u16 + 2).min(20); // +2 for borders

    // Position popup - try above cursor first, then below
    let (cursor_x, cursor_y) = cursor_screen_pos;

    let popup_x = if cursor_x + popup_width < area.width {
        cursor_x
    } else {
        area.width.saturating_sub(popup_width)
    };

    let popup_y = if cursor_y > popup_height {
        cursor_y - popup_height - 1 // Above cursor
    } else if cursor_y + popup_height + 2 < area.height {
        cursor_y + 1 // Below cursor
    } else {
        1 // Top of screen
    };

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Render popup
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.ui.border.to_color()))
        .title(" Documentation ")
        .title_style(Style::default().fg(theme.ui.foreground.to_color()))
        .style(Style::default().bg(theme.ui.background.to_color()));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
