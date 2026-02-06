use super::buffer::Buffer;
use super::clipboard::{Clipboard, YankType};
use super::cursor::CursorOps;
use super::undo::{EditorAction, UndoStack};

/// Editing operations (insert, delete, etc.)
pub struct EditOps;

impl EditOps {
    /// Calculate indentation for a new line based on the previous line
    pub fn calculate_indent(line: &str) -> String {
        // Get leading whitespace
        let leading_ws: String = line.chars().take_while(|c| c.is_whitespace()).collect();

        // Check if line ends with something that should increase indent
        let trimmed = line.trim().to_lowercase();
        let increase_indent = trimmed.ends_with("proc")
            || trimmed.ends_with("macro")
            || trimmed.ends_with(':') // Labels
            || trimmed.starts_with(".data")
            || trimmed.starts_with(".code");

        if increase_indent {
            format!("{}    ", leading_ws)
        } else {
            leading_ws
        }
    }

    /// Insert a character at the cursor position
    pub fn insert_char(
        buf: &mut Buffer,
        undo_stack: &mut UndoStack,
        c: char,
        clear_search_fn: &mut dyn FnMut(),
    ) {
        if buf.cursor_y >= buf.lines.len() {
            return;
        }

        let line = &mut buf.lines[buf.cursor_y];
        buf.cursor_x = CursorOps::clamp_to_char_boundary(line, buf.cursor_x);
        if buf.cursor_x > line.len() {
            return;
        }

        let ln = buf.cursor_y;
        let col_b = buf.cursor_x;
        let col_c = CursorOps::char_index_at_byte(line, col_b);

        line.insert(col_b, c);
        buf.cursor_x = col_b + c.len_utf8();
        buf.modified = true;

        undo_stack.push(EditorAction::InsertChar {
            line: ln,
            col: col_c,
            ch: c,
        });

        clear_search_fn();
    }

    /// Insert a newline at the cursor position
    pub fn insert_newline(
        buf: &mut Buffer,
        undo_stack: &mut UndoStack,
        auto_indent: bool,
        clear_search_fn: &mut dyn FnMut(),
    ) {
        if buf.cursor_y >= buf.lines.len() {
            return;
        }

        let ln = buf.cursor_y;
        let current_line = &buf.lines[buf.cursor_y];
        buf.cursor_x = CursorOps::clamp_to_char_boundary(current_line, buf.cursor_x);
        let col_b = buf.cursor_x;
        let col_c = CursorOps::char_index_at_byte(current_line, col_b);

        let remainder = current_line[col_b..].to_string();
        buf.lines[buf.cursor_y] = current_line[..col_b].to_string();

        let indent = if auto_indent {
            Self::calculate_indent(&buf.lines[buf.cursor_y])
        } else {
            String::new()
        };

        buf.cursor_y += 1;
        buf.lines
            .insert(buf.cursor_y, format!("{}{}", indent, remainder));
        buf.cursor_x = indent.len();
        buf.modified = true;

        undo_stack.push(EditorAction::SplitLine {
            line: ln,
            col: col_c,
        });

        clear_search_fn();
    }

    /// Delete character before cursor (backspace)
    pub fn backspace(
        buf: &mut Buffer,
        undo_stack: &mut UndoStack,
        clear_search_fn: &mut dyn FnMut(),
    ) {
        let action = if buf.cursor_y >= buf.lines.len() {
            None
        } else if buf.cursor_x > 0 {
            let line = &mut buf.lines[buf.cursor_y];
            buf.cursor_x = CursorOps::clamp_to_char_boundary(line, buf.cursor_x);
            let start = CursorOps::prev_char_boundary(line, buf.cursor_x);
            let end = buf.cursor_x;

            if start == end {
                None
            } else {
                let ch = line[start..end].chars().next().unwrap_or(' ');
                let line_num = buf.cursor_y;
                let col_char = CursorOps::char_index_at_byte(line, start);

                line.drain(start..end);
                buf.cursor_x = start;
                buf.modified = true;

                Some(EditorAction::DeleteChar {
                    line: line_num,
                    col: col_char,
                    ch,
                })
            }
        } else if buf.cursor_y > 0 {
            let current_line = buf.lines.remove(buf.cursor_y);
            let line_num = buf.cursor_y;
            buf.cursor_y -= 1;

            let prev_line = &mut buf.lines[buf.cursor_y];
            let join_col_char = prev_line.chars().count();
            prev_line.push_str(&current_line);

            buf.cursor_x = prev_line.len();
            buf.modified = true;

            Some(EditorAction::JoinLines {
                line: line_num - 1,
                col: join_col_char,
                deleted_content: current_line,
            })
        } else {
            None
        };

        if let Some(act) = action {
            undo_stack.push(act);
        }
        clear_search_fn();
    }

    /// Delete character at cursor
    pub fn delete_char(
        buf: &mut Buffer,
        undo_stack: &mut UndoStack,
        clear_search_fn: &mut dyn FnMut(),
    ) {
        let action = if buf.cursor_y >= buf.lines.len() {
            None
        } else {
            let line_len = buf.lines[buf.cursor_y].len();
            let cursor_y = buf.cursor_y;

            let cursor_x = {
                let line_ref = &buf.lines[cursor_y];
                CursorOps::clamp_to_char_boundary(line_ref, buf.cursor_x)
            };
            buf.cursor_x = cursor_x;

            if cursor_x < line_len {
                let end = {
                    let line_ref = &buf.lines[cursor_y];
                    CursorOps::next_char_boundary(line_ref, cursor_x)
                };

                if end <= cursor_x {
                    None
                } else {
                    let ch = {
                        let line_ref = &buf.lines[cursor_y];
                        line_ref[cursor_x..end].chars().next().unwrap_or(' ')
                    };
                    let col_char = {
                        let line_ref = &buf.lines[cursor_y];
                        CursorOps::char_index_at_byte(line_ref, cursor_x)
                    };

                    {
                        let line_mut = &mut buf.lines[cursor_y];
                        line_mut.drain(cursor_x..end);
                    }

                    buf.modified = true;
                    Some(EditorAction::DeleteChar {
                        line: cursor_y,
                        col: col_char,
                        ch,
                    })
                }
            } else if cursor_y + 1 < buf.lines.len() {
                let next_line = buf.lines.remove(cursor_y + 1);
                let join_col_char = buf.lines[cursor_y].chars().count();
                buf.lines[cursor_y].push_str(&next_line);
                buf.modified = true;

                Some(EditorAction::JoinLines {
                    line: cursor_y,
                    col: join_col_char,
                    deleted_content: next_line,
                })
            } else {
                None
            }
        };

        if let Some(act) = action {
            undo_stack.push(act);
        }
        clear_search_fn();
    }

    /// Delete entire line (also copies to clipboard)
    pub fn delete_line(
        buf: &mut Buffer,
        undo_stack: &mut UndoStack,
        clipboard: &mut Clipboard,
        clear_search_fn: &mut dyn FnMut(),
    ) {
        let (line_num, content, was_single) = {
            let ln = buf.cursor_y;
            if buf.lines.len() > 1 {
                let c = buf.lines.remove(buf.cursor_y);
                if buf.cursor_y >= buf.lines.len() {
                    buf.cursor_y = buf.lines.len() - 1;
                }
                CursorOps::clamp_cursor_x(buf);
                buf.modified = true;
                (ln, c, false)
            } else {
                let c = buf.lines[0].clone();
                buf.lines[0].clear();
                buf.cursor_x = 0;
                buf.modified = true;
                (ln, c, true)
            }
        };

        clipboard.copy(&(content.clone() + "\n"), YankType::Line);

        if was_single {
            if !content.is_empty() {
                undo_stack.push(EditorAction::ReplaceLine {
                    line_num,
                    old: content,
                    new: String::new(),
                });
            }
        } else {
            undo_stack.push(EditorAction::DeleteLine { line_num, content });
        }
        clear_search_fn();
    }

    /// Insert tab (as spaces)
    pub fn insert_tab(
        buf: &mut Buffer,
        undo_stack: &mut UndoStack,
        tab_size: usize,
        clear_search_fn: &mut dyn FnMut(),
    ) {
        for _ in 0..tab_size {
            Self::insert_char(buf, undo_stack, ' ', clear_search_fn);
        }
    }
}
