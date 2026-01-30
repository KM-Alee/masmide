use crate::theme::Theme;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

/// Help content organized by section - compact two-column format
const HELP_SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "GLOBAL",
        &[
            ("F1", "Help"),
            ("F5", "Build+Run"),
            ("F6", "Build"),
            ("F7", "Run"),
            ("F8", "Output view"),
            ("F9", "Save output"),
            ("Ctrl+S", "Save"),
            ("Ctrl+Q", "Quit"),
            ("Ctrl+E", "File tree"),
            ("Ctrl+O", "Output"),
            ("Tab", "Cycle focus"),
        ],
    ),
    (
        "NORMAL MODE",
        &[
            ("i/a/A", "Insert"),
            ("o/O", "New line ↓/↑"),
            ("v/V", "Visual mode"),
            ("hjkl", "←↓↑→"),
            ("w/b", "Word →/←"),
            ("0/$", "Line start/end"),
            ("g/G", "File start/end"),
            ("x/dd", "Delete"),
            ("y/p", "Yank/paste"),
            ("u/Ctrl+R", "Undo/redo"),
            ("/n/N", "Search/next/prev"),
            (":", "Command"),
        ],
    ),
    (
        "INSERT",
        &[
            ("Esc", "Normal mode"),
            ("Ctrl+C/V/X", "Copy/paste/cut"),
            ("Ctrl+Z/Y", "Undo/redo"),
        ],
    ),
    (
        "VISUAL",
        &[("y/d", "Yank/delete"), ("Ctrl+C", "Copy"), ("Esc", "Exit")],
    ),
    (
        "FILES (R=refresh)",
        &[
            ("j/k h/l", "Navigate"),
            ("Enter", "Open"),
            ("a/A", "New file/dir"),
            ("r/d", "Rename/delete"),
        ],
    ),
    (
        "OUTPUT",
        &[
            ("jk/gG", "Scroll/jump"),
            ("Ctrl+C", "Clear/copy"),
            ("y", "Copy (F8)"),
        ],
    ),
    (
        "COMMANDS",
        &[
            (":w :q :wq", "Save/quit"),
            (":e file", "Open"),
            (":bn :bp :bd", "Buffers"),
            (":theme n", "Theme"),
            (":autosave", "Toggle"),
            (":refresh", "File tree"),
        ],
    ),
    (
        "RESIZE",
        &[("Ctrl+←→", "Tree width"), ("Ctrl+↑↓", "Output height")],
    ),
];

pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, scroll: usize) {
    // Calculate centered popup area - more compact
    let popup_width = (area.width * 80 / 100).min(68);
    let popup_height = (area.height * 80 / 100).min(28);

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(
        area.x + popup_x,
        area.y + popup_y,
        popup_width,
        popup_height,
    );

    // Clear the popup area
    frame.render_widget(Clear, popup_area);

    // Build help text
    let mut lines: Vec<Line> = Vec::new();

    let key_style = Style::default()
        .fg(theme.ui.title_focused.to_color())
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(theme.ui.foreground.to_color());
    let section_style = Style::default()
        .fg(theme.syntax.keyword.to_color())
        .add_modifier(Modifier::BOLD);
    let dim_style = Style::default().fg(theme.ui.line_numbers.to_color());

    for (section_name, bindings) in HELP_SECTIONS {
        // Section header with decorative line
        lines.push(Line::from(vec![
            Span::styled("┌─ ", dim_style),
            Span::styled(*section_name, section_style),
            Span::styled(" ─", dim_style),
        ]));

        // Two-column layout for bindings
        let mut row: Vec<Span> = Vec::new();
        for (i, (key, desc)) in bindings.iter().enumerate() {
            let entry = vec![
                Span::styled(format!(" {:11}", key), key_style),
                Span::styled(format!("{:18}", desc), desc_style),
            ];
            row.extend(entry);

            if i % 2 == 1 || i == bindings.len() - 1 {
                lines.push(Line::from(row.clone()));
                row.clear();
            }
        }
    }

    let content_height = lines.len();
    let visible_height = popup_height.saturating_sub(2) as usize;
    let max_scroll = content_height.saturating_sub(visible_height);
    let scroll = scroll.min(max_scroll);

    // Title with version
    let title = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            "MASMIDE",
            Style::default()
                .fg(theme.ui.title_focused.to_color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " Help ",
            Style::default().fg(theme.ui.foreground.to_color()),
        ),
    ]);

    let block = Block::default()
        .title(title)
        .title_bottom(
            Line::from(vec![
                Span::styled(" ↑↓/jk ", key_style),
                Span::styled("scroll ", desc_style),
                Span::styled("Esc ", key_style),
                Span::styled("close ", desc_style),
            ])
            .right_aligned(),
        )
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.ui.border_focused.to_color()))
        .style(Style::default().bg(theme.ui.background.to_color()));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((scroll as u16, 0));

    frame.render_widget(paragraph, popup_area);

    // Scrollbar - only show if needed
    if content_height > visible_height {
        let scrollbar_area = Rect::new(
            popup_area.x + popup_area.width - 1,
            popup_area.y + 1,
            1,
            popup_area.height - 2,
        );

        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("▓");

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

pub fn total_lines() -> usize {
    let mut count = 2; // Header + empty line
    for (_, bindings) in HELP_SECTIONS {
        count += 1; // Section header
        count += bindings.len().div_ceil(2); // Bindings (2 per row)
        count += 1; // Empty line
    }
    count
}
