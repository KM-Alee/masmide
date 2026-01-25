use crate::app::{App, Mode};
use crate::diagnostics::{count_by_severity, DiagnosticSeverity};
use ratatui::{prelude::*, text::Span, widgets::Paragraph};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();

    let mode_str = match app.mode {
        Mode::Normal => " NORMAL ",
        Mode::Insert => " INSERT ",
        Mode::Command => " COMMAND ",
        Mode::FileTree => " FILES ",
        Mode::Search => " SEARCH ",
        Mode::InputPopup => " INPUT ",
        Mode::Visual => " VISUAL ",
        Mode::VisualLine => " V-LINE ",
    };

    let mode_style = match app.mode {
        Mode::Normal => Style::default()
            .bg(theme.ui.mode_normal_bg.to_color())
            .fg(theme.ui.mode_normal_fg.to_color())
            .add_modifier(Modifier::BOLD),
        Mode::Insert => Style::default()
            .bg(theme.ui.mode_insert_bg.to_color())
            .fg(theme.ui.mode_insert_fg.to_color())
            .add_modifier(Modifier::BOLD),
        Mode::Command => Style::default()
            .bg(theme.ui.mode_command_bg.to_color())
            .fg(theme.ui.mode_command_fg.to_color())
            .add_modifier(Modifier::BOLD),
        Mode::FileTree => Style::default()
            .bg(theme.ui.mode_filetree_bg.to_color())
            .fg(theme.ui.mode_filetree_fg.to_color())
            .add_modifier(Modifier::BOLD),
        Mode::Search => Style::default()
            .bg(theme.ui.mode_search_bg.to_color())
            .fg(theme.ui.mode_search_fg.to_color())
            .add_modifier(Modifier::BOLD),
        Mode::InputPopup => Style::default()
            .bg(theme.ui.mode_command_bg.to_color())
            .fg(theme.ui.mode_command_fg.to_color())
            .add_modifier(Modifier::BOLD),
        Mode::Visual | Mode::VisualLine => Style::default()
            .bg(theme.ui.selection.to_color())
            .fg(theme.ui.selection_fg.to_color())
            .add_modifier(Modifier::BOLD),
    };

    let file_info = match app.editor.current_file() {
        Some(path) => {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let modified = if app.editor.modified() { " ●" } else { "" };
            format!(" {}{} ", name, modified)
        }
        None => String::from(" [No File] "),
    };

    // Buffer indicator
    let buffer_info = if app.editor.buffers.len() > 1 {
        format!(
            " [{}/{}] ",
            app.editor.active_buffer + 1,
            app.editor.buffers.len()
        )
    } else {
        String::new()
    };

    let cursor_pos = format!(
        " Ln {}, Col {} ",
        app.editor.cursor_y() + 1,
        app.editor.cursor_x() + 1
    );

    // Check for diagnostic at cursor position
    let cursor_diagnostic = app.diagnostic_at_cursor();

    // Use diagnostic message if cursor is on an error line, otherwise use status message
    let status_msg = if let Some(diag) = cursor_diagnostic {
        let severity = match diag.severity {
            DiagnosticSeverity::Error => "Error",
            DiagnosticSeverity::Warning => "Warning",
        };
        format!(" {}: {} ", severity, diag.message)
    } else {
        format!(" {} ", app.status_message)
    };

    // Diagnostic count indicator
    let (errors, warnings) = count_by_severity(&app.diagnostics);
    let diag_indicator = if errors > 0 || warnings > 0 {
        let mut parts = Vec::new();
        if errors > 0 {
            parts.push(format!("✗ {}", errors));
        }
        if warnings > 0 {
            parts.push(format!("⚠ {}", warnings));
        }
        format!(" {} ", parts.join(" "))
    } else {
        String::new()
    };

    let mode_span = Span::styled(mode_str, mode_style);
    let file_span = Span::styled(
        file_info,
        Style::default()
            .bg(theme.ui.tab_active_bg.to_color())
            .fg(theme.ui.tab_active_fg.to_color()),
    );
    let buffer_span = Span::styled(
        buffer_info,
        Style::default()
            .bg(theme.ui.tab_inactive_bg.to_color())
            .fg(theme.ui.tab_inactive_fg.to_color()),
    );

    // Diagnostic indicator span (styled based on whether there are errors)
    let diag_span = if !diag_indicator.is_empty() {
        if errors > 0 {
            Span::styled(
                diag_indicator.clone(),
                Style::default()
                    .bg(theme.ui.diagnostic_error.to_color())
                    .fg(theme.ui.background.to_color())
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                diag_indicator.clone(),
                Style::default()
                    .bg(theme.ui.diagnostic_warning.to_color())
                    .fg(theme.ui.background.to_color())
                    .add_modifier(Modifier::BOLD),
            )
        }
    } else {
        Span::raw("")
    };

    let msg_span = Span::styled(
        status_msg.clone(),
        Style::default().fg(if cursor_diagnostic.is_some() {
            match cursor_diagnostic.unwrap().severity {
                DiagnosticSeverity::Error => theme.ui.diagnostic_error.to_color(),
                DiagnosticSeverity::Warning => theme.ui.diagnostic_warning.to_color(),
            }
        } else {
            theme.ui.status_bar_fg.to_color()
        }),
    );

    // Calculate remaining space for right-aligned cursor position
    let left_len = mode_str.len()
        + file_span.content.len()
        + buffer_span.content.len()
        + diag_indicator.len()
        + status_msg.len();
    let right_len = cursor_pos.len();
    let padding = if area.width as usize > left_len + right_len {
        area.width as usize - left_len - right_len
    } else {
        1
    };

    let padding_span = Span::raw(" ".repeat(padding));
    let cursor_span = Span::styled(
        cursor_pos,
        Style::default()
            .bg(theme.ui.tab_active_bg.to_color())
            .fg(theme.ui.tab_active_fg.to_color()),
    );

    let line = Line::from(vec![
        mode_span,
        file_span,
        buffer_span,
        diag_span,
        msg_span,
        padding_span,
        cursor_span,
    ]);
    let paragraph =
        Paragraph::new(line).style(Style::default().bg(theme.ui.status_bar_bg.to_color()));

    frame.render_widget(paragraph, area);
}
