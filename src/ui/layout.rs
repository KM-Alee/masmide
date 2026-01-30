use crate::app::{App, FocusedPanel, Mode};
use ratatui::prelude::*;

pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Output-only fullscreen mode - clone theme to avoid borrow conflict
    if app.output_only_mode {
        let theme = app.config.theme.clone();
        render_output_only(frame, app, size, &theme);
        return;
    }

    // For normal rendering, we can use a reference since we handle borrows carefully
    let theme = app.config.theme.clone();

    // Main vertical layout: content area + status bar + (optional) command/search bar
    let bottom_bar_height = match app.mode {
        Mode::Command | Mode::Search => 1,
        _ => 0,
    };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(bottom_bar_height),
        ])
        .split(size);

    let content_area = main_chunks[0];
    let status_area = main_chunks[1];

    // Content area: file tree (optional) | editor/output
    let mut h_constraints = Vec::new();

    if app.show_file_tree {
        h_constraints.push(Constraint::Length(app.file_tree_width));
    }

    h_constraints.push(Constraint::Min(30)); // Main area takes all remaining space

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(h_constraints)
        .split(content_area);

    let mut chunk_idx = 0;

    // Render file tree
    if app.show_file_tree {
        let mut file_tree_state = app.file_tree.clone();
        super::file_tree::render(
            frame,
            h_chunks[chunk_idx],
            &mut file_tree_state,
            app.focus == FocusedPanel::FileTree,
            &theme,
        );
        chunk_idx += 1;
    }

    // Main editor/output area
    let main_area = h_chunks[chunk_idx];

    // Check if we need tab bar (multiple buffers)
    let show_tabs = app.editor.buffers.len() > 1;

    // Split for tabs if needed
    let editor_area = if show_tabs {
        let tab_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Tab bar
                Constraint::Min(5),    // Editor content
            ])
            .split(main_area);

        super::tabs::render(frame, tab_chunks[0], &app.editor, &theme);
        tab_chunks[1]
    } else {
        main_area
    };

    // Split vertically: editor on top, output on bottom
    if app.show_output {
        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(app.output_height)])
            .split(editor_area);

        super::editor::render(
            frame,
            v_chunks[0],
            &app.editor,
            app.focus == FocusedPanel::Editor
                && app.mode != Mode::Command
                && app.mode != Mode::Search,
            &theme,
            &app.diagnostics,
            app.editor.current_file(),
        );

        super::output::render(
            frame,
            v_chunks[1],
            &mut app.output,
            app.focus == FocusedPanel::Output,
            &theme,
        );
    } else {
        super::editor::render(
            frame,
            editor_area,
            &app.editor,
            app.focus == FocusedPanel::Editor
                && app.mode != Mode::Command
                && app.mode != Mode::Search,
            &theme,
            &app.diagnostics,
            app.editor.current_file(),
        );
    }

    // Render status bar
    super::status_bar::render(frame, status_area, app);

    // Render command bar if in command mode
    if app.mode == Mode::Command {
        super::command_bar::render(frame, main_chunks[2], &app.command_input, &theme);
    }

    // Render search bar if in search mode
    if app.mode == Mode::Search {
        super::search_bar::render(
            frame,
            main_chunks[2],
            &app.search_input,
            &app.editor,
            &theme,
        );
    }

    // Render help popup if visible
    if app.show_help {
        super::help::render(frame, size, &theme, app.help_scroll);
    }

    // Render input popup if in that mode
    if app.mode == Mode::InputPopup {
        super::input_popup::render(
            frame,
            size,
            &app.input_popup_title,
            &app.input_popup_value,
            &theme,
        );
    }

    // Render autocomplete popup if visible
    if app.autocomplete.visible && app.mode == Mode::Insert {
        // Calculate cursor screen position
        let buf = &app.editor.buffers[app.editor.active_buffer];
        let line_number_width = format!("{}", buf.lines.len()).len() + 2;

        // Account for file tree width and editor position
        let editor_x = if app.show_file_tree {
            app.file_tree_width
        } else {
            0
        };
        let editor_y = if app.editor.buffers.len() > 1 { 1 } else { 0 }; // Tab bar

        let cursor_screen_x = editor_x + line_number_width as u16 + 1 + buf.cursor_x as u16;
        let cursor_screen_y =
            editor_y + 1 + (buf.cursor_y.saturating_sub(buf.scroll_offset)) as u16;

        super::autocomplete::render(
            frame,
            &app.autocomplete,
            (cursor_screen_x, cursor_screen_y),
            &theme,
        );
    }

    // Render hover documentation popup if visible
    if app.show_hover {
        if let Some(doc) = app.hover_doc {
            let buf = &app.editor.buffers[app.editor.active_buffer];
            let line_number_width = format!("{}", buf.lines.len()).len() + 2;

            let editor_x = if app.show_file_tree {
                app.file_tree_width
            } else {
                0
            };
            let editor_y = if app.editor.buffers.len() > 1 { 1 } else { 0 };

            let cursor_screen_x = editor_x + line_number_width as u16 + 1 + buf.cursor_x as u16;
            let cursor_screen_y =
                editor_y + 1 + (buf.cursor_y.saturating_sub(buf.scroll_offset)) as u16;

            super::hover::render(frame, doc, (cursor_screen_x, cursor_screen_y), &theme);
        }
    }
}

/// Render fullscreen output-only view (for screenshots)
fn render_output_only(frame: &mut Frame, app: &mut App, size: Rect, theme: &crate::theme::Theme) {
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;

    // Layout: output panel + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(size);

    let output_area = chunks[0];
    let status_area = chunks[1];

    // Render output panel fullscreen
    super::output::render(
        frame,
        output_area,
        &mut app.output,
        true, // Always focused in this mode
        theme,
    );

    // Render minimal status bar
    let status_text = Line::from(vec![
        Span::styled(
            " OUTPUT ",
            ratatui::style::Style::default()
                .fg(theme.ui.mode_normal_fg.to_color())
                .bg(theme.ui.mode_normal_bg.to_color())
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            "F8/Esc",
            ratatui::style::Style::default()
                .fg(theme.ui.title_focused.to_color())
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        Span::styled(
            " exit  ",
            ratatui::style::Style::default().fg(theme.ui.foreground.to_color()),
        ),
        Span::styled(
            "F9",
            ratatui::style::Style::default()
                .fg(theme.ui.title_focused.to_color())
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        Span::styled(
            " save  ",
            ratatui::style::Style::default().fg(theme.ui.foreground.to_color()),
        ),
        Span::styled(
            "Ctrl+C",
            ratatui::style::Style::default()
                .fg(theme.ui.title_focused.to_color())
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        Span::styled(
            " copy  ",
            ratatui::style::Style::default().fg(theme.ui.foreground.to_color()),
        ),
        Span::styled(
            "j/k",
            ratatui::style::Style::default()
                .fg(theme.ui.title_focused.to_color())
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        Span::styled(
            " scroll",
            ratatui::style::Style::default().fg(theme.ui.foreground.to_color()),
        ),
    ]);

    let status_bar = Paragraph::new(status_text)
        .style(ratatui::style::Style::default().bg(theme.ui.status_bar_bg.to_color()));

    frame.render_widget(status_bar, status_area);
}
