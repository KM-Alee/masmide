use crate::theme::Theme;
use ratatui::{
    layout::Position,
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render(frame: &mut Frame, area: Rect, title: &str, value: &str, theme: &Theme) {
    let popup_width = 60;
    let popup_height = 3;

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(
        area.x + popup_x,
        area.y + popup_y,
        popup_width,
        popup_height,
    );

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.ui.border_focused.to_color()))
        .style(Style::default().bg(theme.ui.background.to_color()));

    let paragraph = Paragraph::new(value)
        .block(block)
        .style(Style::default().fg(theme.ui.foreground.to_color()));

    frame.render_widget(paragraph, popup_area);

    // Set cursor at the end of the input
    frame.set_cursor_position(Position::new(
        popup_area.x + 1 + value.len() as u16,
        popup_area.y + 1,
    ));
}
