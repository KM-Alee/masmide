use super::buffer::Buffer;
use super::cursor::CursorOps;
use super::undo::{EditorAction, UndoStack};

/// Whether a yank was line-wise or character-wise
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YankType {
    Line,
    Char,
}

/// Single source of truth for clipboard state.
/// Owns the system clipboard handle, internal yank buffer, and yank type.
pub struct Clipboard {
    system: Option<arboard::Clipboard>,
    yank_buffer: String,
    yank_type: YankType,
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            system: arboard::Clipboard::new().ok(),
            yank_buffer: String::new(),
            yank_type: YankType::Char,
        }
    }

    /// Copy text into the clipboard with an explicit yank type.
    /// Always syncs to the system clipboard.
    pub fn copy(&mut self, text: &str, yank_type: YankType) {
        self.yank_buffer = text.to_string();
        self.yank_type = yank_type;

        // Use CLI tools (wl-copy/xclip) as primary — they persist clipboard
        // independently of the process, which is critical for TUI apps.
        // Fall back to arboard if CLI tools aren't available.
        if !Self::copy_with_cli(text) {
            if let Some(ref mut cb) = self.system {
                if let Err(e) = cb.set_text(text.to_string()) {
                    eprintln!("Warning: Failed to set system clipboard: {}", e);
                }
            }
        }
    }

    /// Copy using CLI tools (wl-copy / xclip). Returns true if successful.
    fn copy_with_cli(text: &str) -> bool {
        use std::io::Write;
        use std::process::{Command, Stdio};

        // Try wl-copy first (Wayland), then xclip (X11)
        let commands: &[&[&str]] = &[
            &["wl-copy"],
            &["xclip", "-selection", "clipboard"],
        ];

        for cmd in commands {
            if let Ok(mut child) = Command::new(cmd[0])
                .args(&cmd[1..])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                if let Some(ref mut stdin) = child.stdin {
                    let _ = stdin.write_all(text.as_bytes());
                }
                if let Ok(status) = child.wait() {
                    if status.success() {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Paste using CLI tools (wl-paste / xclip). Returns clipboard content if successful.
    fn paste_with_cli() -> Option<String> {
        use std::process::{Command, Stdio};

        let commands: &[&[&str]] = &[
            &["wl-paste", "--no-newline"],
            &["xclip", "-selection", "clipboard", "-o"],
        ];

        for cmd in commands {
            if let Ok(output) = Command::new(cmd[0])
                .args(&cmd[1..])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                if output.status.success() {
                    if let Ok(text) = String::from_utf8(output.stdout) {
                        if !text.is_empty() {
                            return Some(text);
                        }
                    }
                }
            }
        }
        None
    }

    /// Paste from clipboard.
    /// - Reads system clipboard first.
    /// - If system content matches yank_buffer, returns stored YankType.
    /// - If system content differs (external copy), returns YankType::Char.
    /// - If system clipboard unavailable, falls back to internal buffer.
    pub fn paste(&mut self) -> Option<(String, YankType)> {
        // Try CLI tools first (most reliable for TUI apps), then arboard
        let system_text = Self::paste_with_cli()
            .or_else(|| self.system.as_mut().and_then(|cb| cb.get_text().ok()));

        match system_text {
            Some(text) if !text.is_empty() => {
                if text == self.yank_buffer {
                    Some((text, self.yank_type))
                } else {
                    // External copy — treat as char-wise
                    Some((text, YankType::Char))
                }
            }
            _ => {
                // System clipboard unavailable or empty — use internal buffer
                if self.yank_buffer.is_empty() {
                    None
                } else {
                    Some((self.yank_buffer.clone(), self.yank_type))
                }
            }
        }
    }
}

// ========== Text editing helpers (not clipboard logic) ==========

/// Insert text inline (may span multiple lines) as a single undo action.
/// Works like vim/nano: splits at newlines, joins prefix/suffix properly.
pub fn paste_text_inline(buf: &mut Buffer, undo_stack: &mut UndoStack, text: &str) {
    if text.is_empty() {
        return;
    }

    let start_line = buf.cursor_y;
    let start_col = buf.cursor_x;

    if start_line >= buf.lines.len() {
        return;
    }

    let current_line = &buf.lines[start_line];
    let insert_pos =
        CursorOps::clamp_to_char_boundary(current_line, start_col.min(current_line.len()));
    let start_col_char = CursorOps::char_index_at_byte(current_line, insert_pos);

    let prefix = buf.lines[start_line][..insert_pos].to_string();
    let suffix = buf.lines[start_line][insert_pos..].to_string();

    buf.lines.remove(start_line);

    let paste_lines: Vec<&str> = text.split('\n').collect();

    let mut result_lines = Vec::new();
    for (idx, paste_line) in paste_lines.iter().enumerate() {
        if idx == 0 {
            result_lines.push(format!("{}{}", prefix, paste_line));
        } else if idx == paste_lines.len() - 1 {
            result_lines.push(format!("{}{}", paste_line, suffix));
        } else {
            result_lines.push(paste_line.to_string());
        }
    }

    for (offset, line) in result_lines.iter().enumerate() {
        buf.lines.insert(start_line + offset, line.clone());
    }

    let end_line = start_line + paste_lines.len() - 1;
    let last_pasted_line = paste_lines[paste_lines.len() - 1];
    buf.cursor_y = end_line;
    buf.cursor_x = last_pasted_line.len();
    buf.modified = true;

    let end_col_char = last_pasted_line.chars().count();

    undo_stack.push(EditorAction::InsertText {
        start_line,
        start_col: start_col_char,
        end_line,
        end_col: end_col_char,
        text: text.to_string(),
    });
}

/// Helper for undoing InsertText action
pub fn undo_insert_text(buf: &mut Buffer, start_line: usize, start_col: usize, text: &str) {
    let lines: Vec<&str> = text.split('\n').collect();

    if lines.len() == 1 {
        if start_line < buf.lines.len() {
            let line = &mut buf.lines[start_line];
            let start_byte = CursorOps::byte_index_of_char(line, start_col);
            let end_byte = start_byte + text.len();
            if end_byte <= line.len() {
                line.drain(start_byte..end_byte);
                buf.cursor_y = start_line;
                buf.cursor_x = start_byte;
                buf.modified = true;
            }
        }
    } else {
        let end_line = start_line + lines.len() - 1;
        if end_line < buf.lines.len() {
            let prefix = if start_line < buf.lines.len() {
                let line = &buf.lines[start_line];
                let start_byte = CursorOps::byte_index_of_char(line, start_col);
                line[..start_byte].to_string()
            } else {
                String::new()
            };

            let suffix = if end_line < buf.lines.len() {
                let last_line_len = lines[lines.len() - 1].len();
                buf.lines[end_line][last_line_len..].to_string()
            } else {
                String::new()
            };

            for _ in start_line..end_line {
                if start_line < buf.lines.len() {
                    buf.lines.remove(start_line);
                }
            }

            if start_line < buf.lines.len() {
                buf.lines[start_line] = prefix + &suffix;
            }

            buf.cursor_y = start_line;
            buf.cursor_x =
                CursorOps::byte_index_of_char(&buf.lines[start_line], start_col);
            buf.modified = true;
        }
    }
}

/// Helper for redoing InsertText action
pub fn redo_insert_text(buf: &mut Buffer, start_line: usize, start_col: usize, text: &str) {
    let lines: Vec<&str> = text.split('\n').collect();

    if lines.len() == 1 {
        if start_line < buf.lines.len() {
            let line = &mut buf.lines[start_line];
            let insert_pos = CursorOps::byte_index_of_char(line, start_col);
            line.insert_str(insert_pos, text);
            buf.cursor_x = insert_pos + text.len();
            buf.cursor_y = start_line;
            buf.modified = true;
        }
    } else {
        if start_line < buf.lines.len() {
            let current_line = &buf.lines[start_line];
            let insert_pos = CursorOps::byte_index_of_char(current_line, start_col);

            let prefix = current_line[..insert_pos].to_string();
            let suffix = current_line[insert_pos..].to_string();

            buf.lines[start_line] = prefix + lines[0];

            for (i, line_text) in lines.iter().enumerate().skip(1).take(lines.len() - 2) {
                buf.lines.insert(start_line + i, line_text.to_string());
            }

            let last_line_text = lines[lines.len() - 1];
            let end_line = start_line + lines.len() - 1;
            buf.lines
                .insert(end_line, last_line_text.to_string() + &suffix);

            buf.cursor_y = end_line;
            buf.cursor_x = last_line_text.len();
            buf.modified = true;
        }
    }
}
