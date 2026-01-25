use crate::theme::Theme;
use ratatui::{prelude::*, widgets::Paragraph};

pub fn render(frame: &mut Frame, area: Rect, input: &str, theme: &Theme) {
    let text = format!(":{}", input);
    let paragraph = Paragraph::new(text).style(
        Style::default()
            .fg(theme.ui.foreground.to_color())
            .bg(theme.ui.status_bar_bg.to_color()),
    );
    frame.render_widget(paragraph, area);

    // Position cursor after the colon and input
    frame.set_cursor_position(Position::new(area.x + 1 + input.len() as u16, area.y));
}
