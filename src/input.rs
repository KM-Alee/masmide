use crate::app::{App, FocusedPanel, Mode, PendingAction};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    Build,
    Run,
    BuildAndRun,
    Save,
}

/// Result of executing a command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandResult {
    Continue,
    Quit,
}

pub fn handle_event(app: &mut App) -> Result<Option<Action>> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(Some(Action::None));
    }

    if let Event::Key(key) = event::read()? {
        return handle_key(app, key);
    }

    Ok(Some(Action::None))
}

fn handle_key(app: &mut App, key: KeyEvent) -> Result<Option<Action>> {
    // Help popup takes priority - with scrolling support
    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('q') => {
                app.show_help = false;
                app.help_scroll = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.help_scroll = app.help_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
            }
            KeyCode::Char('g') | KeyCode::Home => {
                app.help_scroll = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                app.help_scroll = crate::ui::help::total_lines();
            }
            KeyCode::PageDown => {
                app.help_scroll = app.help_scroll.saturating_add(10);
            }
            KeyCode::PageUp => {
                app.help_scroll = app.help_scroll.saturating_sub(10);
            }
            _ => {}
        }
        return Ok(Some(Action::None));
    }

    // Global keybindings (work in any mode except when help is shown)
    match key.code {
        KeyCode::F(1) => {
            app.show_help = true;
            return Ok(Some(Action::None));
        }
        KeyCode::F(5) => return Ok(Some(Action::BuildAndRun)),
        KeyCode::F(6) => return Ok(Some(Action::Build)),
        KeyCode::F(7) => return Ok(Some(Action::Run)),
        KeyCode::F(8) => {
            app.toggle_output_only_mode();
            return Ok(Some(Action::None));
        }
        KeyCode::F(9) => {
            match app.export_output() {
                Ok(path) => {
                    app.status_message = format!("Output saved to: {}", path.display());
                }
                Err(e) => {
                    app.status_message = format!("Failed to save output: {}", e);
                }
            }
            return Ok(Some(Action::None));
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(Some(Action::Save));
        }
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(Some(Action::Quit));
        }
        _ => {}
    }

    // Handle output panel focus separately
    if app.focus == FocusedPanel::Output {
        return handle_output_panel(app, key);
    }

    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::Insert => handle_insert_mode(app, key),
        Mode::Command => handle_command_mode(app, key),
        Mode::FileTree => handle_file_tree_mode(app, key),
        Mode::Search => handle_search_mode(app, key),
        Mode::InputPopup => handle_input_popup_mode(app, key),
        Mode::Visual => handle_visual_mode(app, key),
        Mode::VisualLine => handle_visual_line_mode(app, key),
    }
}

fn handle_output_panel(app: &mut App, key: KeyEvent) -> Result<Option<Action>> {
    // Handle resize with Ctrl+arrows
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Left => {
                app.decrease_file_tree_width();
                return Ok(Some(Action::None));
            }
            KeyCode::Right => {
                app.increase_file_tree_width();
                return Ok(Some(Action::None));
            }
            KeyCode::Up => {
                app.increase_output_height();
                return Ok(Some(Action::None));
            }
            KeyCode::Down => {
                app.decrease_output_height();
                return Ok(Some(Action::None));
            }
            _ => {}
        }
    }

    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => app.output_scroll_down(1),
        KeyCode::Char('k') | KeyCode::Up => app.output_scroll_up(1),
        KeyCode::Char('g') => app.output_scroll_to_top(),
        KeyCode::Char('G') => app.output_scroll_to_bottom(),
        KeyCode::PageUp | KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.output_page_up()
        }
        KeyCode::PageDown | KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.output_page_down()
        }
        KeyCode::PageUp => app.output_page_up(),
        KeyCode::PageDown => app.output_page_down(),

        // Copy output to clipboard
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.output_only_mode {
                app.copy_output_to_clipboard();
                app.status_message = String::from("Output copied to clipboard");
            } else {
                app.output.clear();
                app.status_message = String::from("Output cleared");
            }
        }

        // Switch focus / exit output-only mode
        KeyCode::Tab => {
            if app.output_only_mode {
                app.toggle_output_only_mode();
            } else {
                app.focus = FocusedPanel::Editor;
                app.mode = Mode::Normal;
            }
        }
        KeyCode::Esc => {
            if app.output_only_mode {
                app.toggle_output_only_mode();
            } else {
                app.focus = FocusedPanel::Editor;
                app.mode = Mode::Normal;
            }
        }

        // Toggle output panel
        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_output = !app.show_output;
            if !app.show_output {
                app.focus = FocusedPanel::Editor;
            }
        }

        // Yank (copy) in output-only mode
        KeyCode::Char('y') if app.output_only_mode => {
            app.copy_output_to_clipboard();
            app.status_message = String::from("Output copied to clipboard");
        }

        _ => {}
    }
    Ok(Some(Action::None))
}

fn handle_input_popup_mode(app: &mut App, key: KeyEvent) -> Result<Option<Action>> {
    match key.code {
        KeyCode::Esc => {
            app.cancel_input_popup();
        }
        KeyCode::Enter => {
            app.execute_input_popup()?;
        }
        KeyCode::Char(c) => {
            app.input_popup_value.push(c);
        }
        KeyCode::Backspace => {
            app.input_popup_value.pop();
        }
        _ => {}
    }
    Ok(Some(Action::None))
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<Option<Action>> {
    // Hide hover on any key
    if app.show_hover {
        app.hide_hover();
    }

    // Handle pending g command (for gd - go to definition)
    if app.pending_g {
        app.pending_g = false;
        if let KeyCode::Char('d') = key.code {
            if let Some(symbol) = app.editor.go_to_definition() {
                app.editor.ensure_cursor_visible(20);
                app.status_message = format!("Jump to: {}", symbol);
            } else {
                app.status_message = String::from("No definition found");
            }
            return Ok(Some(Action::None));
        } else if let KeyCode::Char('g') = key.code {
            // gg - go to first line
            let buf = &mut app.editor.buffers[app.editor.active_buffer];
            buf.cursor_y = 0;
            buf.cursor_x = 0;
            buf.scroll_offset = 0;
            return Ok(Some(Action::None));
        }
        // Other g commands could be added here
        return Ok(Some(Action::None));
    }

    // Handle pending bracket command (for ]e - next error, [e - prev error)
    if let Some(bracket) = app.pending_bracket {
        app.pending_bracket = None;
        if let KeyCode::Char('e') = key.code {
            match bracket {
                ']' => {
                    app.next_diagnostic();
                }
                '[' => {
                    app.prev_diagnostic();
                }
                _ => {}
            }
            return Ok(Some(Action::None));
        }
        // Other bracket commands could be added here (]w - next warning, etc.)
        return Ok(Some(Action::None));
    }

    // Handle pending char for f/F/t/T commands
    if let Some(cmd) = app.pending_char {
        if let KeyCode::Char(c) = key.code {
            let count = app.pending_count.unwrap_or(1);
            for _ in 0..count {
                match cmd {
                    'f' => {
                        app.editor.find_char_forward(c);
                    }
                    'F' => {
                        app.editor.find_char_backward(c);
                    }
                    't' => {
                        app.editor.find_char_till_forward(c);
                    }
                    'T' => {
                        app.editor.find_char_till_backward(c);
                    }
                    _ => {}
                }
            }
            app.pending_char = None;
            app.pending_count = None;
            return Ok(Some(Action::None));
        }
        app.pending_char = None;
        app.pending_count = None;
        return Ok(Some(Action::None));
    }

    // Handle count prefix (1-9 for first digit, 0-9 for subsequent)
    if let KeyCode::Char(c) = key.code {
        if c.is_ascii_digit() {
            if c == '0' && app.pending_count.is_none() {
                // '0' alone is move to line start
                app.editor.move_to_line_start();
                return Ok(Some(Action::None));
            }
            let digit = c.to_digit(10).unwrap() as usize;
            app.pending_count = Some(app.pending_count.unwrap_or(0) * 10 + digit);
            return Ok(Some(Action::None));
        }
    }

    let count = app.pending_count.take().unwrap_or(1);

    match key.code {
        // Mode switching
        KeyCode::Char('i') => {
            app.mode = Mode::Insert;
        }
        KeyCode::Char('a') => {
            app.editor.move_cursor_right();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('A') => {
            app.editor.move_to_line_end();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('I') => {
            app.editor.move_to_first_non_blank();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('o') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.editor.move_to_line_end();
            app.editor.insert_newline();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('O') => {
            app.editor.move_to_line_start();
            app.editor.insert_newline();
            app.editor.move_cursor_up();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_output = !app.show_output;
        }
        KeyCode::Char(':') => {
            app.mode = Mode::Command;
            app.command_input.clear();
        }
        KeyCode::Char('/') => {
            app.start_search();
        }

        // Ctrl+V to paste (non-vim users)
        KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.editor.paste_after();
            app.status_message = String::from("Pasted");
        }

        // Visual mode
        KeyCode::Char('v') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.editor.start_visual_selection();
            app.mode = Mode::Visual;
        }
        KeyCode::Char('V') => {
            app.editor.start_visual_line_selection();
            app.mode = Mode::VisualLine;
        }

        // Undo/Redo
        KeyCode::Char('u') => {
            if app.editor.undo() {
                app.status_message = String::from("Undo");
            } else {
                app.status_message = String::from("Already at oldest change");
            }
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.editor.redo() {
                app.status_message = String::from("Redo");
            } else {
                app.status_message = String::from("Already at newest change");
            }
        }
        // Ctrl+Z for undo (non-vim users)
        KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.editor.undo() {
                app.status_message = String::from("Undo");
            } else {
                app.status_message = String::from("Already at oldest change");
            }
        }
        // Ctrl+Y for redo (non-vim users)
        KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.editor.redo() {
                app.status_message = String::from("Redo");
            } else {
                app.status_message = String::from("Already at newest change");
            }
        }

        // Ctrl+C to copy line (non-vim users)
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.editor.yank_line();
            app.status_message = String::from("Copied line");
        }
        // Ctrl+X to cut line (non-vim users)
        KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.editor.delete_line();
            app.status_message = String::from("Cut line");
        }

        // Yank and paste (vim style)
        KeyCode::Char('y') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.editor.yank_line();
            app.status_message = String::from("Yanked line");
        }
        KeyCode::Char('p') => {
            for _ in 0..count {
                app.editor.paste_after();
            }
            app.status_message = String::from("Pasted");
        }
        KeyCode::Char('P') => {
            for _ in 0..count {
                app.editor.paste_before();
            }
            app.status_message = String::from("Pasted before");
        }

        // Search navigation
        KeyCode::Char('n') => {
            for _ in 0..count {
                app.editor.find_next();
            }
            if let Some(status) = app.editor.search_status() {
                app.status_message = format!("Search: {}", status);
            }
            app.editor.ensure_cursor_visible(20);
        }
        KeyCode::Char('N') => {
            for _ in 0..count {
                app.editor.find_prev();
            }
            if let Some(status) = app.editor.search_status() {
                app.status_message = format!("Search: {}", status);
            }
            app.editor.ensure_cursor_visible(20);
        }

        // Navigation - with count support
        KeyCode::Char('h') | KeyCode::Left if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            for _ in 0..count {
                app.editor.move_cursor_left();
            }
        }
        KeyCode::Char('j') | KeyCode::Down if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            for _ in 0..count {
                app.editor.move_cursor_down();
            }
            app.editor.ensure_cursor_visible(20);
        }
        KeyCode::Char('k') | KeyCode::Up if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            for _ in 0..count {
                app.editor.move_cursor_up();
            }
            app.editor.ensure_cursor_visible(20);
        }
        KeyCode::Char('l') | KeyCode::Right if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            for _ in 0..count {
                app.editor.move_cursor_right();
            }
        }

        // Word motions
        KeyCode::Char('w') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            for _ in 0..count {
                app.editor.move_word_forward();
            }
        }
        KeyCode::Char('b') => {
            for _ in 0..count {
                app.editor.move_word_backward();
            }
        }
        KeyCode::Char('e') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            for _ in 0..count {
                app.editor.move_word_end();
            }
        }

        // Line motions
        KeyCode::Char('^') => app.editor.move_to_first_non_blank(),
        KeyCode::Char('$') => app.editor.move_to_line_end(),

        // Bracket matching
        KeyCode::Char('%') => {
            if !app.editor.find_matching_bracket() {
                app.status_message = String::from("No matching bracket");
            }
        }

        // Hover documentation
        KeyCode::Char('K') => {
            app.show_hover_docs();
        }

        // Go back (from go-to-definition)
        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.editor.go_back() {
                app.editor.ensure_cursor_visible(20);
                app.status_message = String::from("Jump back");
            } else {
                app.status_message = String::from("Jump stack empty");
            }
        }

        // Char finding
        KeyCode::Char('f') => {
            app.pending_char = Some('f');
            app.pending_count = Some(count);
        }
        KeyCode::Char('F') => {
            app.pending_char = Some('F');
            app.pending_count = Some(count);
        }
        KeyCode::Char('t') => {
            app.pending_char = Some('t');
            app.pending_count = Some(count);
        }
        KeyCode::Char('T') => {
            app.pending_char = Some('T');
            app.pending_count = Some(count);
        }

        // Go to line / go to definition
        KeyCode::Char('g') => {
            app.pending_g = true;
        }
        // Error navigation: ]e next error, [e prev error
        KeyCode::Char(']') => {
            app.pending_bracket = Some(']');
        }
        KeyCode::Char('[') => {
            app.pending_bracket = Some('[');
        }
        KeyCode::Char('G') => {
            if count > 1 {
                // nG - go to line n
                app.editor.go_to_line(count);
            } else {
                // G alone - go to end
                let len = app.editor.buffers[app.editor.active_buffer].lines.len();
                app.editor.buffers[app.editor.active_buffer].cursor_y = len.saturating_sub(1);
            }
            app.editor.ensure_cursor_visible(20);
        }

        // Editing in normal mode
        KeyCode::Char('x') => {
            for _ in 0..count {
                app.editor.delete_char();
            }
        }
        KeyCode::Char('d') => {
            for _ in 0..count {
                app.editor.delete_line();
            }
        }

        // Panel focus
        KeyCode::Tab if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.focus = match app.focus {
                FocusedPanel::Editor => {
                    if app.show_file_tree {
                        app.mode = Mode::FileTree;
                        FocusedPanel::FileTree
                    } else if app.show_output {
                        FocusedPanel::Output
                    } else {
                        FocusedPanel::Editor
                    }
                }
                FocusedPanel::FileTree => {
                    app.mode = Mode::Normal;
                    if app.show_output {
                        FocusedPanel::Output
                    } else {
                        FocusedPanel::Editor
                    }
                }
                FocusedPanel::Output => FocusedPanel::Editor,
            };
        }

        // Buffer switching with Ctrl+Tab
        KeyCode::Tab if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.editor.next_buffer();
            app.status_message = format!(
                "Buffer: {}",
                app.editor.buffers[app.editor.active_buffer].filename()
            );
        }

        // Toggle panels
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_file_tree = !app.show_file_tree;
        }

        // Panel resizing
        KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.decrease_file_tree_width();
        }
        KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.increase_file_tree_width();
        }
        KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.increase_output_height();
        }
        KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.decrease_output_height();
        }

        // Close buffer
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.editor.modified() {
                app.status_message =
                    String::from("Buffer has unsaved changes. Save first or use :bd!");
            } else if app.editor.close_buffer() {
                app.status_message = String::from("Buffer closed");
            }
        }

        KeyCode::Esc => {
            app.editor.clear_search();
            app.pending_count = None;
            app.pending_char = None;
            app.pending_bracket = None;
            app.status_message = String::from("Press F1 for help");
        }

        _ => {}
    }

    Ok(Some(Action::None))
}

fn handle_insert_mode(app: &mut App, key: KeyEvent) -> Result<Option<Action>> {
    // Handle autocomplete navigation if visible
    if app.autocomplete.visible {
        match key.code {
            KeyCode::Esc => {
                app.autocomplete.hide();
                return Ok(Some(Action::None));
            }
            KeyCode::Tab | KeyCode::Enter => {
                app.accept_autocomplete();
                return Ok(Some(Action::None));
            }
            KeyCode::Up => {
                app.autocomplete.select_prev();
                return Ok(Some(Action::None));
            }
            KeyCode::Down => {
                app.autocomplete.select_next();
                return Ok(Some(Action::None));
            }
            _ => {
                // Continue to normal handling, but hide autocomplete for non-char keys
                if !matches!(key.code, KeyCode::Char(_) | KeyCode::Backspace) {
                    app.autocomplete.hide();
                }
            }
        }
    }

    // Clipboard operations (Ctrl+C, Ctrl+V, Ctrl+X)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') => {
                // Copy current line if no selection
                app.editor.yank_line();
                app.status_message = String::from("Copied line");
                return Ok(Some(Action::None));
            }
            KeyCode::Char('v') => {
                // Paste from system clipboard in insert mode
                if let Some((text, _yank_type)) = app.editor.clipboard.paste() {
                    let buf = &mut app.editor.buffers[app.editor.active_buffer];
                    crate::ui::editor::clipboard::paste_text_inline(
                        buf,
                        &mut app.editor.undo_stack,
                        &text,
                    );
                    app.status_message = String::from("Pasted from clipboard");
                } else {
                    app.status_message = String::from("Clipboard empty");
                }
                return Ok(Some(Action::None));
            }
            KeyCode::Char('x') => {
                // Cut current line
                app.editor.delete_line();
                app.status_message = String::from("Cut line");
                return Ok(Some(Action::None));
            }
            KeyCode::Char('z') => {
                if app.editor.undo() {
                    app.status_message = String::from("Undo");
                }
                return Ok(Some(Action::None));
            }
            KeyCode::Char('y') => {
                if app.editor.redo() {
                    app.status_message = String::from("Redo");
                }
                return Ok(Some(Action::None));
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.autocomplete.hide();
        }
        KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Manual autocomplete trigger
            app.trigger_autocomplete();
        }
        KeyCode::Char(c) => {
            app.editor.insert_char(c);
            // Auto-trigger autocomplete after 2+ characters
            if c.is_alphanumeric() || c == '_' || c == '.' {
                let buf = &app.editor.buffers[app.editor.active_buffer];
                let cursor_byte = buf.cursor_x;
                if cursor_byte >= 2 {
                    let line = &buf.lines[buf.cursor_y];
                    let chars: Vec<char> = line.chars().collect();
                    
                    // Convert byte position to character index
                    let char_pos = line[..cursor_byte.min(line.len())].chars().count();
                    
                    // Check if we have at least 2 identifier chars
                    let mut word_len = 0;
                    let mut i = char_pos;
                    while i > 0 && i - 1 < chars.len() {
                        let ch = chars[i - 1];
                        if ch.is_alphanumeric() || ch == '_' || ch == '.' || ch == '@' {
                            word_len += 1;
                            i -= 1;
                        } else {
                            break;
                        }
                    }
                    if word_len >= 2 {
                        app.trigger_autocomplete();
                    }
                }
            } else {
                app.autocomplete.hide();
            }
        }
        KeyCode::Enter => {
            app.autocomplete.hide();
            app.editor.insert_newline();
        }
        KeyCode::Backspace => {
            app.editor.backspace();
            // Update autocomplete after backspace
            if app.autocomplete.visible {
                app.trigger_autocomplete();
            }
        }
        KeyCode::Delete => {
            app.editor.delete_char();
        }
        KeyCode::Tab => {
            app.editor.insert_tab();
        }
        KeyCode::Left => app.editor.move_cursor_left(),
        KeyCode::Right => app.editor.move_cursor_right(),
        KeyCode::Up => {
            app.editor.move_cursor_up();
            app.editor.ensure_cursor_visible(20);
        }
        KeyCode::Down => {
            app.editor.move_cursor_down();
            app.editor.ensure_cursor_visible(20);
        }
        KeyCode::Home => app.editor.move_to_line_start(),
        KeyCode::End => app.editor.move_to_line_end(),
        _ => {}
    }

    Ok(Some(Action::None))
}

fn handle_command_mode(app: &mut App, key: KeyEvent) -> Result<Option<Action>> {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.command_input.clear();
        }
        KeyCode::Enter => match app.execute_command() {
            Ok(CommandResult::Quit) => {
                return Ok(Some(Action::Quit));
            }
            Ok(CommandResult::Continue) => {}
            Err(e) => {
                app.status_message = format!("Error: {}", e);
            }
        },
        KeyCode::Char(c) => {
            app.command_input.push(c);
        }
        KeyCode::Backspace => {
            app.command_input.pop();
            if app.command_input.is_empty() {
                app.mode = Mode::Normal;
            }
        }
        _ => {}
    }

    Ok(Some(Action::None))
}

fn handle_file_tree_mode(app: &mut App, key: KeyEvent) -> Result<Option<Action>> {
    // Handle resize with Ctrl+arrows (global in file tree mode too)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Left => {
                app.decrease_file_tree_width();
                return Ok(Some(Action::None));
            }
            KeyCode::Right => {
                app.increase_file_tree_width();
                return Ok(Some(Action::None));
            }
            KeyCode::Up => {
                app.increase_output_height();
                return Ok(Some(Action::None));
            }
            KeyCode::Down => {
                app.decrease_output_height();
                return Ok(Some(Action::None));
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = Mode::Normal;
            app.focus = FocusedPanel::Editor;
        }
        // Refresh file tree
        KeyCode::Char('R') => {
            if let Err(e) = app.file_tree.refresh() {
                app.status_message = format!("Refresh failed: {}", e);
            } else {
                app.status_message = String::from("File tree refreshed");
            }
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.file_tree.move_down();
            // Auto-open text files on navigation
            if let Some(entry) = app.file_tree.selected_entry() {
                if !entry.is_dir {
                    let name = &entry.name;
                    if !name.ends_with(".exe")
                        && !name.ends_with(".obj")
                        && !name.ends_with(".lib")
                        && !name.ends_with(".o")
                    {
                        let path = entry.path.clone();
                        let _ = app.open_file(&path);
                    }
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.file_tree.move_up();
            // Auto-open text files on navigation
            if let Some(entry) = app.file_tree.selected_entry() {
                if !entry.is_dir {
                    let name = &entry.name;
                    if !name.ends_with(".exe")
                        && !name.ends_with(".obj")
                        && !name.ends_with(".lib")
                        && !name.ends_with(".o")
                    {
                        let path = entry.path.clone();
                        let _ = app.open_file(&path);
                    }
                }
            }
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            if let Ok(Some(file_path)) = app.file_tree.toggle_expand() {
                // It's a file, open it
                if let Err(e) = app.open_file(&file_path) {
                    app.status_message = format!("Error: {}", e);
                }
                app.mode = Mode::Normal;
            }
        }
        KeyCode::Char('h') | KeyCode::Left => {
            // Collapse if expanded, otherwise do nothing
            if let Some(entry) = app.file_tree.selected_entry() {
                if entry.is_dir && entry.expanded {
                    let _ = app.file_tree.toggle_expand();
                }
            }
        }
        KeyCode::Tab => {
            app.mode = Mode::Normal;
            app.focus = if app.show_output {
                FocusedPanel::Output
            } else {
                FocusedPanel::Editor
            };
        }
        KeyCode::Char('a') => {
            app.mode = Mode::InputPopup;
            app.pending_action = PendingAction::CreateFile;
            app.input_popup_title = String::from("Create File (Enter name):");
            app.input_popup_value.clear();
        }
        KeyCode::Char('A') => {
            app.mode = Mode::InputPopup;
            app.pending_action = PendingAction::CreateDir;
            app.input_popup_title = String::from("Create Directory (Enter name):");
            app.input_popup_value.clear();
        }
        KeyCode::Char('r') => {
            if let Some(entry) = app.file_tree.selected_entry() {
                app.mode = Mode::InputPopup;
                app.pending_action = PendingAction::Rename;
                app.input_popup_title = format!("Rename '{}' to:", entry.name);
                app.input_popup_value = entry.name.clone();
            }
        }
        KeyCode::Char('d') => {
            if let Some(entry) = app.file_tree.selected_entry() {
                app.mode = Mode::InputPopup;
                app.pending_action = PendingAction::Delete;
                app.input_popup_title = format!("Delete '{}'? (y/n):", entry.name);
                app.input_popup_value.clear();
            }
        }
        _ => {}
    }

    Ok(Some(Action::None))
}

fn handle_search_mode(app: &mut App, key: KeyEvent) -> Result<Option<Action>> {
    match key.code {
        KeyCode::Esc => {
            app.cancel_search();
        }
        KeyCode::Enter => {
            app.execute_search();
        }
        KeyCode::Char(c) => {
            app.search_input.push(c);
            // Live search as you type
            app.editor.search(&app.search_input);
        }
        KeyCode::Backspace => {
            app.search_input.pop();
            if app.search_input.is_empty() {
                app.editor.clear_search();
            } else {
                app.editor.search(&app.search_input);
            }
        }
        _ => {}
    }

    Ok(Some(Action::None))
}

fn handle_visual_mode(app: &mut App, key: KeyEvent) -> Result<Option<Action>> {
    // Ctrl+C to copy selection
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        if app.editor.yank_selection() {
            app.status_message = String::from("Copied selection");
        }
        app.editor.clear_selection();
        app.mode = Mode::Normal;
        return Ok(Some(Action::None));
    }

    match key.code {
        // Exit visual mode
        KeyCode::Esc | KeyCode::Char('v') => {
            app.editor.clear_selection();
            app.mode = Mode::Normal;
        }

        // Switch to visual line mode
        KeyCode::Char('V') => {
            app.editor.start_visual_line_selection();
            app.mode = Mode::VisualLine;
        }

        // Navigation - extends selection
        KeyCode::Char('h') | KeyCode::Left => {
            app.editor.move_cursor_left();
            app.editor.update_selection();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.editor.move_cursor_down();
            app.editor.update_selection();
            app.editor.ensure_cursor_visible(20);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.editor.move_cursor_up();
            app.editor.update_selection();
            app.editor.ensure_cursor_visible(20);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.editor.move_cursor_right();
            app.editor.update_selection();
        }

        // Word motions
        KeyCode::Char('w') => {
            app.editor.move_word_forward();
            app.editor.update_selection();
        }
        KeyCode::Char('b') => {
            app.editor.move_word_backward();
            app.editor.update_selection();
        }
        KeyCode::Char('e') => {
            app.editor.move_word_end();
            app.editor.update_selection();
        }

        // Line motions
        KeyCode::Char('0') => {
            app.editor.move_to_line_start();
            app.editor.update_selection();
        }
        KeyCode::Char('$') => {
            app.editor.move_to_line_end();
            app.editor.update_selection();
        }
        KeyCode::Char('^') => {
            app.editor.move_to_first_non_blank();
            app.editor.update_selection();
        }

        // File motions
        KeyCode::Char('g') => {
            {
                let buf = &mut app.editor.buffers[app.editor.active_buffer];
                buf.cursor_y = 0;
                buf.cursor_x = 0;
                buf.scroll_offset = 0;
            }
            app.editor.update_selection();
        }
        KeyCode::Char('G') => {
            {
                let len = app.editor.buffers[app.editor.active_buffer].lines.len();
                app.editor.buffers[app.editor.active_buffer].cursor_y = len.saturating_sub(1);
            }
            app.editor.update_selection();
            app.editor.ensure_cursor_visible(20);
        }

        // Operations on selection
        KeyCode::Char('y') => {
            if app.editor.yank_selection() {
                app.status_message = String::from("Yanked selection");
            }
            app.mode = Mode::Normal;
        }
        KeyCode::Char('d') | KeyCode::Char('x') => {
            if app.editor.delete_selection() {
                app.status_message = String::from("Deleted selection");
            }
            app.mode = Mode::Normal;
        }

        _ => {}
    }

    Ok(Some(Action::None))
}

fn handle_visual_line_mode(app: &mut App, key: KeyEvent) -> Result<Option<Action>> {
    match key.code {
        // Exit visual mode
        KeyCode::Esc | KeyCode::Char('V') => {
            app.editor.clear_selection();
            app.mode = Mode::Normal;
        }

        // Switch to regular visual mode
        KeyCode::Char('v') => {
            app.editor.start_visual_selection();
            app.mode = Mode::Visual;
        }

        // Navigation - extends selection (line-wise)
        KeyCode::Char('j') | KeyCode::Down => {
            app.editor.move_cursor_down();
            app.editor.update_visual_line_selection();
            app.editor.ensure_cursor_visible(20);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.editor.move_cursor_up();
            app.editor.update_visual_line_selection();
            app.editor.ensure_cursor_visible(20);
        }

        // File motions
        KeyCode::Char('g') => {
            {
                let buf = &mut app.editor.buffers[app.editor.active_buffer];
                buf.cursor_y = 0;
                buf.cursor_x = 0;
                buf.scroll_offset = 0;
            }
            app.editor.update_visual_line_selection();
        }
        KeyCode::Char('G') => {
            {
                let len = app.editor.buffers[app.editor.active_buffer].lines.len();
                app.editor.buffers[app.editor.active_buffer].cursor_y = len.saturating_sub(1);
            }
            app.editor.update_visual_line_selection();
            app.editor.ensure_cursor_visible(20);
        }

        // Operations on selection
        KeyCode::Char('y') => {
            if app.editor.yank_selection() {
                app.status_message = String::from("Yanked lines");
            }
            app.mode = Mode::Normal;
        }
        KeyCode::Char('d') | KeyCode::Char('x') => {
            if app.editor.delete_selection() {
                app.status_message = String::from("Deleted lines");
            }
            app.mode = Mode::Normal;
        }

        _ => {}
    }

    Ok(Some(Action::None))
}
