use super::buffer::Buffer;

/// Cursor movement utilities using ropey
/// Much simpler now - ropey handles UTF-8 automatically!
pub struct CursorOps;

impl CursorOps {

    /// DEPRECATED: No longer needed with ropey
    pub fn clamp_to_char_boundary(s: &str, idx: usize) -> usize {
        let idx = idx.min(s.len());
        if s.is_char_boundary(idx) {
            return idx;
        }

        // Search left for the nearest boundary
        let mut i = idx;
        while i > 0 {
            i -= 1;
            if s.is_char_boundary(i) {
                return i;
            }
        }
        0
    }

    /// Find the byte offset of the previous character boundary
    pub fn prev_char_boundary(s: &str, idx: usize) -> usize {
        let idx = Self::clamp_to_char_boundary(s, idx);
        if idx == 0 {
            return 0;
        }

        // Find the start byte offset of the previous char
        let mut i = idx - 1;
        while i > 0 && !s.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    /// Find the byte offset of the next character boundary
    pub fn next_char_boundary(s: &str, idx: usize) -> usize {
        let idx = Self::clamp_to_char_boundary(s, idx);
        if idx >= s.len() {
            return s.len();
        }

        let ch = s[idx..].chars().next().unwrap_or('\0');
        (idx + ch.len_utf8()).min(s.len())
    }

    /// Convert byte index to character index
    pub fn char_index_at_byte(s: &str, byte_idx: usize) -> usize {
        let byte_idx = Self::clamp_to_char_boundary(s, byte_idx);
        s[..byte_idx].chars().count()
    }

    /// Convert character index to byte index
    pub fn byte_index_of_char(s: &str, char_idx: usize) -> usize {
        if char_idx == 0 {
            return 0;
        }
        match s.char_indices().nth(char_idx) {
            Some((b, _)) => b,
            None => s.len(),
        }
    }

    /// Ensure cursor_x is on a valid character boundary
    pub fn set_cursor_x_char_boundary(buf: &mut Buffer) {
        if buf.cursor_y >= buf.lines.len() {
            buf.cursor_x = 0;
            return;
        }
        let line = &buf.lines[buf.cursor_y];
        buf.cursor_x = Self::clamp_to_char_boundary(line, buf.cursor_x);
    }

    /// Clamp cursor_x to valid range for current line
    pub fn clamp_cursor_x(buf: &mut Buffer) {
        if buf.cursor_y >= buf.lines.len() {
            buf.cursor_x = 0;
            return;
        }

        let line = &buf.lines[buf.cursor_y];
        buf.cursor_x = buf.cursor_x.min(line.len());
        // With ropey, UTF-8 is handled automatically, but keep the check for safety
        buf.cursor_x = Self::clamp_to_char_boundary(line, buf.cursor_x);
    }

    /// Move cursor up one line
    pub fn move_up(buf: &mut Buffer) {
        if buf.cursor_y > 0 {
            buf.cursor_y -= 1;
            Self::clamp_cursor_x(buf);
        }
    }

    /// Move cursor down one line
    pub fn move_down(buf: &mut Buffer) {
        if buf.cursor_y + 1 < buf.lines.len() {
            buf.cursor_y += 1;
            Self::clamp_cursor_x(buf);
        }
    }

    /// Move cursor left one character
    pub fn move_left(buf: &mut Buffer) {
        if buf.cursor_y >= buf.lines.len() {
            buf.cursor_y = 0;
            buf.cursor_x = 0;
            return;
        }

        let line = &buf.lines[buf.cursor_y];
        buf.cursor_x = Self::clamp_to_char_boundary(line, buf.cursor_x);

        if buf.cursor_x > 0 {
            buf.cursor_x = Self::prev_char_boundary(line, buf.cursor_x);
        } else if buf.cursor_y > 0 {
            buf.cursor_y -= 1;
            buf.cursor_x = buf.lines[buf.cursor_y].len();
            Self::set_cursor_x_char_boundary(buf);
        }
    }

    /// Move cursor right one character
    pub fn move_right(buf: &mut Buffer) {
        if buf.cursor_y >= buf.lines.len() {
            buf.cursor_y = 0;
            buf.cursor_x = 0;
            return;
        }

        let line = &buf.lines[buf.cursor_y];
        buf.cursor_x = Self::clamp_to_char_boundary(line, buf.cursor_x);

        if buf.cursor_x < line.len() {
            buf.cursor_x = Self::next_char_boundary(line, buf.cursor_x);
        } else if buf.cursor_y + 1 < buf.lines.len() {
            buf.cursor_y += 1;
            buf.cursor_x = 0;
        }
    }

    /// Move cursor to start of line
    pub fn move_to_line_start(buf: &mut Buffer) {
        buf.cursor_x = 0;
    }

    /// Move cursor to end of line
    pub fn move_to_line_end(buf: &mut Buffer) {
        if buf.cursor_y < buf.lines.len() {
            buf.cursor_x = buf.lines[buf.cursor_y].len();
        }
    }

    /// Ensure cursor is visible within the given viewport height
    pub fn ensure_visible(buf: &mut Buffer, visible_height: usize) {
        if buf.cursor_y < buf.scroll_offset {
            buf.scroll_offset = buf.cursor_y;
        } else if buf.cursor_y >= buf.scroll_offset + visible_height {
            buf.scroll_offset = buf.cursor_y - visible_height + 1;
        }
    }
}
