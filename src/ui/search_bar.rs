use crate::theme::Theme;
use crate::ui::editor::EditorState;
use ratatui::{prelude::*, widgets::Paragraph};

pub fn render(frame: &mut Frame, area: Rect, input: &str, editor: &EditorState, theme: &Theme) {
    let status = editor.search_status().unwrap_or_default();
    let text = if status.is_empty() {
        format!("/{}", input)
    } else {
        format!("/{}  [{}]", input, status)
    };

    let paragraph = Paragraph::new(text).style(
        Style::default()
            .fg(theme.ui.foreground.to_color())
            .bg(theme.ui.status_bar_bg.to_color()),
    );
    frame.render_widget(paragraph, area);

    // Position cursor after the slash and input
    frame.set_cursor_position(Position::new(area.x + 1 + input.len() as u16, area.y));
}
