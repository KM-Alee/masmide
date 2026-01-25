use crate::theme::Theme;
use crate::ui::editor::EditorState;
use ratatui::{prelude::*, text::Span, widgets::Paragraph};

pub fn render(frame: &mut Frame, area: Rect, editor: &EditorState, theme: &Theme) {
    let mut spans = Vec::new();

    for (idx, buffer) in editor.buffers.iter().enumerate() {
        let is_active = idx == editor.active_buffer;
        let modified = if buffer.modified { " ●" } else { "" };
        let name = buffer.filename();

        let tab_text = format!(" {}{} ", name, modified);

        let style = if is_active {
            Style::default()
                .fg(theme.ui.tab_active_fg.to_color())
                .bg(theme.ui.tab_active_bg.to_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.ui.tab_inactive_fg.to_color())
                .bg(theme.ui.tab_inactive_bg.to_color())
        };

        spans.push(Span::styled(tab_text, style));

        // Add separator
        if idx < editor.buffers.len() - 1 {
            spans.push(Span::styled(
                "│",
                Style::default().fg(theme.ui.border.to_color()),
            ));
        }
    }

    // Fill remaining space with background
    let line = Line::from(spans);
    let paragraph =
        Paragraph::new(line).style(Style::default().bg(theme.ui.tab_inactive_bg.to_color()));

    frame.render_widget(paragraph, area);
}
