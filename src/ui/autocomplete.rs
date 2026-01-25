//! Autocomplete popup rendering

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::autocomplete::{AutocompleteState, SuggestionKind};
use crate::theme::Theme;

/// Render the autocomplete popup
pub fn render(
    frame: &mut Frame,
    autocomplete: &AutocompleteState,
    cursor_screen_pos: (u16, u16), // (x, y) screen position of cursor
    theme: &Theme,
) {
    if !autocomplete.visible || autocomplete.suggestions.is_empty() {
        return;
    }

    let visible = autocomplete.visible_suggestions();
    let selected_idx = autocomplete.visible_selected();

    // Calculate popup dimensions
    let max_text_width = visible
        .iter()
        .map(|s| s.text.len() + 4) // +4 for kind icon and padding
        .max()
        .unwrap_or(20) as u16;

    let popup_width = max_text_width.max(15).min(40);
    let popup_height = (visible.len() as u16 + 2).min(12); // +2 for borders

    // Position popup below cursor, or above if not enough space
    let (cursor_x, cursor_y) = cursor_screen_pos;
    let area = frame.area();

    let popup_x = cursor_x.min(area.width.saturating_sub(popup_width));
    let popup_y = if cursor_y + popup_height + 1 < area.height {
        cursor_y + 1 // Below cursor
    } else {
        cursor_y.saturating_sub(popup_height) // Above cursor
    };

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Build popup content
    let mut lines: Vec<Line> = Vec::new();

    for (i, suggestion) in visible.iter().enumerate() {
        let is_selected = i == selected_idx;

        let kind_style = match suggestion.kind {
            SuggestionKind::Keyword => Style::default().fg(theme.syntax.keyword.to_color()),
            SuggestionKind::Register => Style::default().fg(theme.syntax.register.to_color()),
            SuggestionKind::Directive => Style::default().fg(theme.syntax.directive.to_color()),
            SuggestionKind::TypeKeyword => Style::default().fg(theme.syntax.type_kw.to_color()),
            SuggestionKind::Label => Style::default().fg(theme.syntax.label.to_color()),
            SuggestionKind::Procedure => Style::default().fg(theme.syntax.label.to_color()),
            SuggestionKind::Macro => Style::default().fg(theme.syntax.macro_call.to_color()),
        };

        let base_style = if is_selected {
            Style::default()
                .bg(theme.ui.selection.to_color())
                .fg(theme.ui.foreground.to_color())
        } else {
            Style::default().fg(theme.ui.foreground.to_color())
        };

        let icon = Span::styled(
            format!("{} ", suggestion.kind.icon()),
            if is_selected { base_style } else { kind_style },
        );

        let text = Span::styled(&suggestion.text, base_style);

        // Pad to fill width
        let padding_len = (popup_width as usize).saturating_sub(suggestion.text.len() + 4);
        let padding = Span::styled(" ".repeat(padding_len), base_style);

        lines.push(Line::from(vec![icon, text, padding]));
    }

    // Show scroll indicator if there are more items
    let total = autocomplete.suggestions.len();
    let title = if total > 10 {
        format!(" {}/{} ", autocomplete.selected + 1, total)
    } else {
        String::new()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.ui.border.to_color()))
        .title(title)
        .style(Style::default().bg(theme.ui.background.to_color()));

    let paragraph = Paragraph::new(lines).block(block);

    frame.render_widget(paragraph, popup_area);
}
