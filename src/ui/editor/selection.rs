use super::buffer::Buffer;
use super::clipboard::{Clipboard, YankType};
use super::cursor::CursorOps;
use super::undo::{EditorAction, UndoStack};

/// Selection operations for visual mode
pub struct SelectionOps;

impl SelectionOps {
    /// Start visual selection at cursor
    pub fn start_selection(buf: &mut Buffer) {
        buf.selection_start = Some((buf.cursor_y, buf.cursor_x));
        buf.selection_end = Some((buf.cursor_y, buf.cursor_x));
    }

    /// Update selection end to cursor
    pub fn update_selection(buf: &mut Buffer) {
        if buf.selection_start.is_some() {
            buf.selection_end = Some((buf.cursor_y, buf.cursor_x));
        }
    }

    /// Clear selection
    pub fn clear_selection(buf: &mut Buffer) {
        buf.selection_start = None;
        buf.selection_end = None;
    }

    /// Check if there's an active selection
    pub fn has_selection(buf: &Buffer) -> bool {
        buf.selection_start.is_some() && buf.selection_end.is_some()
    }

    /// Get normalized selection range (start always before end)
    pub fn get_selection_range(buf: &Buffer) -> Option<((usize, usize), (usize, usize))> {
        if let (Some(start), Some(end)) = (buf.selection_start, buf.selection_end) {
            let (start, end) = if start.0 < end.0 || (start.0 == end.0 && start.1 <= end.1) {
                (start, end)
            } else {
                (end, start)
            };
            Some((start, end))
        } else {
            None
        }
    }

    /// Yank (copy) selected text
    pub fn yank_selection(buf: &Buffer, clipboard: &mut Clipboard) -> bool {
        if let Some(((start_line, start_col), (end_line, end_col))) = Self::get_selection_range(buf)
        {
            let text = Self::extract_selection_text(buf, start_line, start_col, end_line, end_col);
            clipboard.copy(&text, YankType::Char);
            true
        } else {
            false
        }
    }

    /// Delete selected text
    pub fn delete_selection(
        buf: &mut Buffer,
        undo_stack: &mut UndoStack,
        clipboard: &mut Clipboard,
    ) -> bool {
        if let Some(((start_line, start_col), (end_line, end_col))) = Self::get_selection_range(buf)
        {
            let text = Self::extract_selection_text(buf, start_line, start_col, end_line, end_col);
            clipboard.copy(&text, YankType::Char);

            // Perform deletion
            if start_line == end_line {
                // Single line deletion
                if start_line < buf.lines.len() {
                    let line = &mut buf.lines[start_line];
                    let start_byte = CursorOps::clamp_to_char_boundary(line, start_col);
                    let end_byte = CursorOps::clamp_to_char_boundary(line, end_col);
                    line.drain(start_byte..end_byte);
                    buf.cursor_x = start_byte;
                    buf.cursor_y = start_line;
                    buf.modified = true;
                }
            } else {
                // Multi-line deletion
                let start_byte = if start_line < buf.lines.len() {
                    CursorOps::clamp_to_char_boundary(&buf.lines[start_line], start_col)
                } else {
                    0
                };

                let end_byte = if end_line < buf.lines.len() {
                    CursorOps::clamp_to_char_boundary(&buf.lines[end_line], end_col)
                } else {
                    0
                };

                // Remove middle lines
                for _ in (start_line + 1)..end_line {
                    if start_line + 1 < buf.lines.len() {
                        buf.lines.remove(start_line + 1);
                    }
                }

                // Join start and end lines
                if start_line < buf.lines.len() && start_line + 1 < buf.lines.len() {
                    let end_suffix = buf.lines[start_line + 1][end_byte..].to_string();
                    buf.lines.remove(start_line + 1);
                    buf.lines[start_line].truncate(start_byte);
                    buf.lines[start_line].push_str(&end_suffix);
                }

                buf.cursor_x = start_byte;
                buf.cursor_y = start_line;
                buf.modified = true;
            }

            undo_stack.push(EditorAction::DeleteText {
                start_line,
                start_col,
                end_line,
                end_col,
                text,
            });

            Self::clear_selection(buf);
            true
        } else {
            false
        }
    }

    /// Extract text from selection range
    pub fn extract_selection_text(
        buf: &Buffer,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> String {
        if start_line == end_line {
            // Single line
            if start_line < buf.lines.len() {
                let line = &buf.lines[start_line];
                let start_byte = CursorOps::clamp_to_char_boundary(line, start_col);
                let end_byte = CursorOps::clamp_to_char_boundary(line, end_col);
                line[start_byte..end_byte].to_string()
            } else {
                String::new()
            }
        } else {
            // Multi-line
            let mut result = String::new();

            for line_idx in start_line..=end_line {
                if line_idx >= buf.lines.len() {
                    break;
                }

                let line = &buf.lines[line_idx];
                if line_idx == start_line {
                    let start_byte = CursorOps::clamp_to_char_boundary(line, start_col);
                    result.push_str(&line[start_byte..]);
                } else if line_idx == end_line {
                    let end_byte = CursorOps::clamp_to_char_boundary(line, end_col);
                    result.push('\n');
                    result.push_str(&line[..end_byte]);
                } else {
                    result.push('\n');
                    result.push_str(line);
                }
            }

            result
        }
    }
}
