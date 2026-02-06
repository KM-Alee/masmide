// Re-export all public types and the render function
pub mod buffer;
pub mod clipboard;
pub mod cursor;
pub mod edit;
pub mod render;
pub mod search;
pub mod selection;
pub mod undo;

// Public re-exports for API compatibility
pub use buffer::Buffer;
pub use undo::{EditorAction, UndoStack};

use anyhow::Result;
use std::path::PathBuf;

use clipboard::{Clipboard, YankType};
use cursor::CursorOps;
use edit::EditOps;
use selection::SelectionOps;

/// Main editor state managing multiple buffers
pub struct EditorState {
    pub buffers: Vec<Buffer>,
    pub active_buffer: usize,
    pub tab_size: usize,
    pub auto_indent: bool,
    // Search state
    pub search_query: String,
    pub search_matches: Vec<(usize, usize)>,
    pub current_match: usize,
    // Undo/Redo
    pub undo_stack: UndoStack,
    // Clipboard
    pub clipboard: Clipboard,
    // Jump stack for go-to-definition navigation
    pub jump_stack: Vec<(PathBuf, usize, usize)>,
}

impl EditorState {
    pub fn new(tab_size: usize) -> Self {
        Self {
            buffers: vec![Buffer::new()],
            active_buffer: 0,
            tab_size,
            auto_indent: true,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match: 0,
            undo_stack: UndoStack::default(),
            clipboard: Clipboard::new(),
            jump_stack: Vec::new(),
        }
    }

    // ========== Buffer Accessors ==========

    fn buf(&self) -> &Buffer {
        &self.buffers[self.active_buffer]
    }

    fn buf_mut(&mut self) -> &mut Buffer {
        &mut self.buffers[self.active_buffer]
    }

    pub fn lines(&self) -> &Vec<String> {
        &self.buf().lines
    }

    pub fn cursor_x(&self) -> usize {
        self.buf().cursor_x
    }

    pub fn cursor_y(&self) -> usize {
        self.buf().cursor_y
    }

    pub fn scroll_offset(&self) -> usize {
        self.buf().scroll_offset
    }

    pub fn current_file(&self) -> Option<&PathBuf> {
        self.buf().file_path.as_ref()
    }

    pub fn modified(&self) -> bool {
        self.buf().modified
    }

    pub fn set_modified(&mut self, val: bool) {
        self.buf_mut().modified = val;
    }

    // Compatibility shims
    #[allow(non_snake_case)]
    pub fn get_cursor_x(&self) -> usize {
        self.cursor_x()
    }

    #[allow(non_snake_case)]
    pub fn get_cursor_y(&self) -> usize {
        self.cursor_y()
    }

    // ========== File Operations ==========

    pub fn open_file(&mut self, path: &PathBuf) -> Result<()> {
        // Check if file is already open
        for (idx, buf) in self.buffers.iter().enumerate() {
            if buf.file_path.as_ref() == Some(path) {
                self.active_buffer = idx;
                return Ok(());
            }
        }

        let buffer = Buffer::from_file(path)?;

        // If current buffer is empty and unmodified, replace it
        if self.buffers.len() == 1
            && self.buf().lines.len() == 1
            && self.buf().lines[0].is_empty()
            && self.buf().file_path.is_none()
            && !self.buf().modified
        {
            self.buffers[0] = buffer;
        } else {
            self.buffers.push(buffer);
            self.active_buffer = self.buffers.len() - 1;
        }

        Ok(())
    }

    pub fn get_content(&self) -> String {
        self.buf().get_content()
    }

    // ========== Editing Operations ==========

    pub fn insert_char(&mut self, c: char) {
        let buf = self.buf_mut();
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
        buf.sync_rope();  // Sync rope after modifying lines

        self.undo_stack.push(EditorAction::InsertChar {
            line: ln,
            col: col_c,
            ch: c,
        });
        self.clear_search();
    }

    pub fn insert_newline(&mut self) {
        self.insert_newline_with_indent(self.auto_indent);
    }

    pub fn insert_newline_with_indent(&mut self, auto_indent: bool) {
        let buf = self.buf_mut();
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
            EditOps::calculate_indent(&buf.lines[buf.cursor_y])
        } else {
            String::new()
        };

        buf.cursor_y += 1;
        buf.lines.insert(buf.cursor_y, format!("{}{}", indent, remainder));
        buf.cursor_x = indent.len();
        buf.modified = true;
        buf.sync_rope();  // Sync rope after modifying lines

        self.undo_stack.push(EditorAction::SplitLine {
            line: ln,
            col: col_c,
        });
        self.clear_search();
    }

    pub fn backspace(&mut self) {
        let action = {
            let buf = self.buf_mut();
            if buf.cursor_y >= buf.lines.len() {
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
                    buf.sync_rope();  // Sync rope after modifying lines

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
                buf.sync_rope();  // Sync rope after modifying lines

                Some(EditorAction::JoinLines {
                    line: line_num - 1,
                    col: join_col_char,
                    deleted_content: current_line,
                })
            } else {
                None
            }
        };

        if let Some(act) = action {
            self.undo_stack.push(act);
        }
        self.clear_search();
    }

    pub fn delete_char(&mut self) {
        let action = {
            let buf = self.buf_mut();
            if buf.cursor_y >= buf.lines.len() {
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
                        buf.sync_rope();  // Sync rope after modifying lines
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
                    buf.sync_rope();  // Sync rope after modifying lines

                    Some(EditorAction::JoinLines {
                        line: cursor_y,
                        col: join_col_char,
                        deleted_content: next_line,
                    })
                } else {
                    None
                }
            }
        };

        if let Some(act) = action {
            self.undo_stack.push(act);
        }
        self.clear_search();
    }

    pub fn delete_line(&mut self) {
        let (line_num, content, was_single) = {
            let buf = self.buf_mut();
            let ln = buf.cursor_y;
            if buf.lines.len() > 1 {
                let c = buf.lines.remove(buf.cursor_y);
                if buf.cursor_y >= buf.lines.len() {
                    buf.cursor_y = buf.lines.len() - 1;
                }
                CursorOps::clamp_cursor_x(buf);
                buf.modified = true;
                buf.sync_rope();  // Sync rope after modifying lines
                (ln, c, false)
            } else {
                let c = buf.lines[0].clone();
                buf.lines[0].clear();
                buf.cursor_x = 0;
                buf.modified = true;
                buf.sync_rope();  // Sync rope after modifying lines
                (ln, c, true)
            }
        };

        self.clipboard.copy(&(content.clone() + "\n"), YankType::Line);

        if was_single {
            if !content.is_empty() {
                self.undo_stack.push(EditorAction::ReplaceLine {
                    line_num,
                    old: content,
                    new: String::new(),
                });
            }
        } else {
            self.undo_stack.push(EditorAction::DeleteLine { line_num, content });
        }
        self.clear_search();
    }

    pub fn insert_tab(&mut self) {
        for _ in 0..self.tab_size {
            self.insert_char(' ');
        }
    }

    // ========== Cursor Movement ==========

    pub fn move_cursor_up(&mut self) {
        CursorOps::move_up(self.buf_mut());
    }

    pub fn move_cursor_down(&mut self) {
        CursorOps::move_down(self.buf_mut());
    }

    pub fn move_cursor_left(&mut self) {
        CursorOps::move_left(self.buf_mut());
    }

    pub fn move_cursor_right(&mut self) {
        CursorOps::move_right(self.buf_mut());
    }

    pub fn move_to_line_start(&mut self) {
        CursorOps::move_to_line_start(self.buf_mut());
    }

    pub fn move_to_line_end(&mut self) {
        CursorOps::move_to_line_end(self.buf_mut());
    }

    pub fn ensure_cursor_visible(&mut self, visible_height: usize) {
        CursorOps::ensure_visible(self.buf_mut(), visible_height);
    }

    // ========== Clipboard Operations ==========

    pub fn yank_line(&mut self) {
        let buf = &self.buffers[self.active_buffer];
        if buf.cursor_y < buf.lines.len() {
            let content = buf.lines[buf.cursor_y].clone() + "\n";
            self.clipboard.copy(&content, YankType::Line);
        }
    }

    pub fn paste_after(&mut self) {
        let (text, yank_type) = match self.clipboard.paste() {
            Some(v) => v,
            None => return,
        };

        if text.is_empty() {
            return;
        }

        match yank_type {
            YankType::Line => {
                let line_content = text.trim_end_matches('\n').to_string();
                let buf = self.buf_mut();
                let at = buf.cursor_y + 1;
                buf.lines.insert(at, line_content.clone());
                buf.cursor_y = at;
                buf.cursor_x = 0;
                buf.modified = true;

                self.undo_stack.push(EditorAction::InsertLine {
                    line_num: at,
                    content: line_content,
                });
            }
            YankType::Char => {
                let buf = &mut self.buffers[self.active_buffer];
                clipboard::paste_text_inline(buf, &mut self.undo_stack, &text);
            }
        }
    }

    pub fn paste_before(&mut self) {
        let (text, yank_type) = match self.clipboard.paste() {
            Some(v) => v,
            None => return,
        };

        if text.is_empty() {
            return;
        }

        match yank_type {
            YankType::Line => {
                let line_content = text.trim_end_matches('\n').to_string();
                let buf = self.buf_mut();
                let at = buf.cursor_y;
                buf.lines.insert(at, line_content.clone());
                buf.cursor_x = 0;
                buf.modified = true;

                self.undo_stack.push(EditorAction::InsertLine {
                    line_num: at,
                    content: line_content,
                });
            }
            YankType::Char => {
                let buf = &mut self.buffers[self.active_buffer];
                clipboard::paste_text_inline(buf, &mut self.undo_stack, &text);
            }
        }
    }

    // ========== Selection Operations ==========

    pub fn start_selection(&mut self) {
        SelectionOps::start_selection(self.buf_mut());
    }

    pub fn update_selection(&mut self) {
        SelectionOps::update_selection(self.buf_mut());
    }

    pub fn clear_selection(&mut self) {
        SelectionOps::clear_selection(self.buf_mut());
    }

    pub fn get_selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        SelectionOps::get_selection_range(self.buf())
    }

    pub fn yank_selection(&mut self) -> bool {
        let buf = &self.buffers[self.active_buffer];
        if let Some(((start_line, start_col), (end_line, end_col))) =
            SelectionOps::get_selection_range(buf)
        {
            let text = SelectionOps::extract_selection_text(buf, start_line, start_col, end_line, end_col);
            self.clipboard.copy(&text, YankType::Char);
            true
        } else {
            false
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        let buf = &mut self.buffers[self.active_buffer];
        SelectionOps::delete_selection(
            buf,
            &mut self.undo_stack,
            &mut self.clipboard,
        )
    }

    // ========== Search Operations ==========

    pub fn search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.search_matches.clear();
        self.current_match = 0;

        if query.is_empty() {
            return;
        }

        let query_lower = query.to_lowercase();
        let lines: Vec<String> = self.buf().lines.clone();
        for (line_idx, line) in lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            let mut start = 0;
            while let Some(pos) = line_lower[start..].find(&query_lower) {
                self.search_matches.push((line_idx, start + pos));
                start += pos + 1;
            }
        }
    }

    pub fn find_next(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = (self.current_match + 1) % self.search_matches.len();
        self.jump_to_current_match();
    }

    pub fn find_prev(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = if self.current_match == 0 {
            self.search_matches.len() - 1
        } else {
            self.current_match - 1
        };
        self.jump_to_current_match();
    }

    fn jump_to_current_match(&mut self) {
        if let Some(&(line, col)) = self.search_matches.get(self.current_match) {
            let buf = self.buf_mut();
            buf.cursor_y = line;
            buf.cursor_x = col;
        }
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_matches.clear();
        self.current_match = 0;
    }

    pub fn search_status(&self) -> Option<String> {
        if self.search_matches.is_empty() {
            if !self.search_query.is_empty() {
                Some(String::from("No matches"))
            } else {
                None
            }
        } else {
            Some(format!(
                "{}/{}",
                self.current_match + 1,
                self.search_matches.len()
            ))
        }
    }

    // ========== Undo/Redo Operations ==========

    pub fn undo(&mut self) -> bool {
        if let Some(action) = self.undo_stack.pop_undo() {
            self.apply_undo_action(&action);
            self.undo_stack.push_redo(action);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(action) = self.undo_stack.pop_redo() {
            self.apply_redo_action(&action);
            self.undo_stack.undo_stack.push_back(action);
            true
        } else {
            false
        }
    }

    fn apply_undo_action(&mut self, action: &EditorAction) {
        let buf = self.buf_mut();
        match action {
            EditorAction::InsertChar { line, col, .. } => {
                if *line < buf.lines.len() {
                    let ln = &mut buf.lines[*line];
                    let col_b = CursorOps::byte_index_of_char(ln, *col);
                    if col_b < ln.len() {
                        let end = CursorOps::next_char_boundary(ln, col_b);
                        ln.drain(col_b..end);
                        buf.cursor_y = *line;
                        buf.cursor_x = col_b;
                        buf.modified = true;
                    }
                }
            }
            EditorAction::DeleteChar { line, col, ch } => {
                if *line < buf.lines.len() {
                    let ln = &mut buf.lines[*line];
                    let col_b = CursorOps::byte_index_of_char(ln, *col);
                    ln.insert(col_b, *ch);
                    buf.cursor_y = *line;
                    buf.cursor_x = (col_b + ch.len_utf8()).min(ln.len());
                    buf.modified = true;
                }
            }
            EditorAction::InsertLine { line_num, .. } => {
                if *line_num < buf.lines.len() {
                    buf.lines.remove(*line_num);
                    buf.cursor_y = line_num.saturating_sub(1);
                    buf.cursor_x = 0;
                    buf.modified = true;
                }
            }
            EditorAction::DeleteLine { line_num, content } => {
                buf.lines.insert(*line_num, content.clone());
                buf.cursor_y = *line_num;
                buf.cursor_x = 0;
                buf.modified = true;
            }
            EditorAction::ReplaceLine { line_num, old, .. } => {
                if *line_num < buf.lines.len() {
                    buf.lines[*line_num] = old.clone();
                    buf.cursor_y = *line_num;
                    buf.cursor_x = buf.cursor_x.min(buf.lines[*line_num].len());
                    CursorOps::set_cursor_x_char_boundary(buf);
                    buf.modified = true;
                }
            }
            EditorAction::SplitLine { line, col } => {
                if *line + 1 < buf.lines.len() {
                    let next_line = buf.lines.remove(*line + 1);
                    let trimmed = next_line.trim_start();
                    let ln = &mut buf.lines[*line];
                    let col_b = CursorOps::byte_index_of_char(ln, *col);
                    ln.truncate(col_b);
                    ln.push_str(trimmed);
                    buf.cursor_y = *line;
                    buf.cursor_x = col_b.min(ln.len());
                    buf.modified = true;
                }
            }
            EditorAction::JoinLines {
                line,
                col,
                deleted_content,
            } => {
                if *line < buf.lines.len() {
                    let ln = &mut buf.lines[*line];
                    let col_b = CursorOps::byte_index_of_char(ln, *col);
                    let tail = ln.get(col_b..).unwrap_or("").to_string();
                    ln.truncate(col_b);
                    buf.lines.insert(*line + 1, deleted_content.clone() + &tail);
                    buf.cursor_y = *line + 1;
                    buf.cursor_x = 0;
                    buf.modified = true;
                }
            }
            EditorAction::InsertText {
                start_line,
                start_col,
                text,
                ..
            } => {
                clipboard::undo_insert_text(buf, *start_line, *start_col, text);
            }
            EditorAction::DeleteText {
                start_line,
                start_col,
                text,
                ..
            } => {
                clipboard::redo_insert_text(buf, *start_line, *start_col, text);
            }
            EditorAction::Batch(actions) => {
                for action in actions.iter().rev() {
                    self.apply_undo_action(action);
                }
            }
        }
    }

    fn apply_redo_action(&mut self, action: &EditorAction) {
        let buf = self.buf_mut();
        match action {
            EditorAction::InsertChar { line, col, ch } => {
                if *line < buf.lines.len() {
                    let ln = &mut buf.lines[*line];
                    let col_b = CursorOps::byte_index_of_char(ln, *col);
                    ln.insert(col_b, *ch);
                    buf.cursor_y = *line;
                    buf.cursor_x = (col_b + ch.len_utf8()).min(ln.len());
                    buf.modified = true;
                }
            }
            EditorAction::DeleteChar { line, col, .. } => {
                if *line < buf.lines.len() {
                    let ln = &mut buf.lines[*line];
                    let col_b = CursorOps::byte_index_of_char(ln, *col);
                    if col_b < ln.len() {
                        let end = CursorOps::next_char_boundary(ln, col_b);
                        ln.drain(col_b..end);
                        buf.cursor_y = *line;
                        buf.cursor_x = col_b;
                        buf.modified = true;
                    }
                }
            }
            EditorAction::InsertLine { line_num, content } => {
                buf.lines.insert(*line_num, content.clone());
                buf.cursor_y = *line_num;
                buf.cursor_x = 0;
                buf.modified = true;
            }
            EditorAction::DeleteLine { line_num, .. } => {
                if *line_num < buf.lines.len() {
                    buf.lines.remove(*line_num);
                    buf.cursor_y = (*line_num).min(buf.lines.len().saturating_sub(1));
                    buf.cursor_x = 0;
                    buf.modified = true;
                }
            }
            EditorAction::ReplaceLine { line_num, new, .. } => {
                if *line_num < buf.lines.len() {
                    buf.lines[*line_num] = new.clone();
                    buf.cursor_y = *line_num;
                    buf.cursor_x = buf.cursor_x.min(buf.lines[*line_num].len());
                    CursorOps::set_cursor_x_char_boundary(buf);
                    buf.modified = true;
                }
            }
            EditorAction::SplitLine { line, col } => {
                if *line < buf.lines.len() {
                    let ln = &mut buf.lines[*line];
                    let col_b = CursorOps::byte_index_of_char(ln, *col);
                    let remainder = ln.get(col_b..).unwrap_or("").to_string();
                    ln.truncate(col_b);
                    buf.lines.insert(*line + 1, remainder);
                    buf.cursor_y = *line + 1;
                    buf.cursor_x = 0;
                    buf.modified = true;
                }
            }
            EditorAction::JoinLines { line, col, .. } => {
                if *line + 1 < buf.lines.len() {
                    let next = buf.lines.remove(*line + 1);
                    buf.lines[*line].push_str(&next);
                    buf.cursor_y = *line;
                    let ln = &buf.lines[*line];
                    let col_b = CursorOps::byte_index_of_char(ln, *col);
                    buf.cursor_x = col_b.min(ln.len());
                    buf.modified = true;
                }
            }
            EditorAction::InsertText {
                start_line,
                start_col,
                text,
                ..
            } => {
                clipboard::redo_insert_text(buf, *start_line, *start_col, text);
            }
            EditorAction::DeleteText {
                start_line,
                start_col,
                text,
                ..
            } => {
                clipboard::undo_insert_text(buf, *start_line, *start_col, text);
            }
            EditorAction::Batch(actions) => {
                for action in actions.iter() {
                    self.apply_redo_action(action);
                }
            }
        }
    }

    // ========== Buffer Management ==========

    pub fn next_buffer(&mut self) {
        if self.buffers.len() > 1 {
            self.active_buffer = (self.active_buffer + 1) % self.buffers.len();
        }
    }

    pub fn prev_buffer(&mut self) {
        if self.buffers.len() > 1 {
            self.active_buffer = if self.active_buffer == 0 {
                self.buffers.len() - 1
            } else {
                self.active_buffer - 1
            };
        }
    }

    pub fn close_buffer(&mut self) -> bool {
        if self.buffers.len() > 1 {
            self.buffers.remove(self.active_buffer);
            if self.active_buffer >= self.buffers.len() {
                self.active_buffer = self.buffers.len() - 1;
            }
            true
        } else {
            false
        }
    }

    pub fn has_unsaved_buffers(&self) -> bool {
        self.buffers.iter().any(|b| b.modified)
    }

    // ========== Advanced Navigation ==========

    pub fn go_to_line(&mut self, line_num: usize) {
        let buf = self.buf_mut();
        let target = line_num
            .saturating_sub(1)
            .min(buf.lines.len().saturating_sub(1));
        buf.cursor_y = target;
        buf.cursor_x = 0;
    }

    pub fn get_word_under_cursor(&self) -> Option<String> {
        let buf = self.buf();
        if buf.cursor_y >= buf.lines.len() {
            return None;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let cursor_char_idx = CursorOps::char_index_at_byte(line, buf.cursor_x);

        if cursor_char_idx >= chars.len() {
            return None;
        }

        let c = chars[cursor_char_idx];
        if !c.is_alphanumeric() && c != '_' && c != '@' && c != '?' {
            return None;
        }

        let mut start = cursor_char_idx;
        while start > 0 {
            let prev = chars[start - 1];
            if prev.is_alphanumeric() || prev == '_' || prev == '@' || prev == '?' {
                start -= 1;
            } else {
                break;
            }
        }

        let mut end = cursor_char_idx;
        while end < chars.len() {
            let ch = chars[end];
            if ch.is_alphanumeric() || ch == '_' || ch == '@' || ch == '?' {
                end += 1;
            } else {
                break;
            }
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    pub fn find_definition_in_buffer(&self, symbol: &str) -> Option<(usize, usize)> {
        let buf = self.buf();
        let symbol_lower = symbol.to_lowercase();

        for (line_idx, line) in buf.lines.iter().enumerate() {
            let trimmed = line.trim();
            let trimmed_lower = trimmed.to_lowercase();

            if let Some(colon_pos) = trimmed.find(':') {
                let label_part = trimmed[..colon_pos].trim();
                if label_part.to_lowercase() == symbol_lower {
                    let col = line.find(label_part).unwrap_or(0);
                    return Some((line_idx, col));
                }
            }

            if trimmed_lower.contains(" proc") || trimmed_lower.ends_with(" proc") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(name) = parts.first() {
                    if name.to_lowercase() == symbol_lower {
                        let col = line.find(*name).unwrap_or(0);
                        return Some((line_idx, col));
                    }
                }
            }

            if trimmed_lower.contains(" macro") || trimmed_lower.ends_with(" macro") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(name) = parts.first() {
                    if name.to_lowercase() == symbol_lower {
                        let col = line.find(*name).unwrap_or(0);
                        return Some((line_idx, col));
                    }
                }
            }

            if trimmed_lower.contains(" equ ") || trimmed_lower.contains(" equ\t") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(name) = parts.first() {
                    if name.to_lowercase() == symbol_lower {
                        let col = line.find(*name).unwrap_or(0);
                        return Some((line_idx, col));
                    }
                }
            }
        }

        None
    }

    pub fn go_to_definition(&mut self) -> Option<String> {
        let word = self.get_word_under_cursor()?;
        if let Some((line, col)) = self.find_definition_in_buffer(&word) {
            let buf = self.buf();
            if let Some(file_path) = buf.file_path.clone() {
                self.jump_stack.push((file_path, buf.cursor_y, buf.cursor_x));
            }
            let buf = self.buf_mut();
            buf.cursor_y = line;
            buf.cursor_x = col;
            Some(word)
        } else {
            None
        }
    }

    pub fn go_back(&mut self) -> bool {
        if let Some((file_path, line, col)) = self.jump_stack.pop() {
            if self.buf().file_path.as_ref() == Some(&file_path) {
                let buf = self.buf_mut();
                buf.cursor_y = line;
                buf.cursor_x = col;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn move_word_forward(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let mut cursor_char_idx = CursorOps::char_index_at_byte(line, buf.cursor_x);

        while cursor_char_idx < chars.len()
            && (chars[cursor_char_idx].is_alphanumeric() || chars[cursor_char_idx] == '_')
        {
            cursor_char_idx += 1;
        }

        while cursor_char_idx < chars.len() && chars[cursor_char_idx].is_whitespace() {
            cursor_char_idx += 1;
        }

        buf.cursor_x = CursorOps::byte_index_of_char(line, cursor_char_idx);
    }

    pub fn move_word_backward(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let mut cursor_char_idx = CursorOps::char_index_at_byte(line, buf.cursor_x);

        if cursor_char_idx > 0 {
            cursor_char_idx -= 1;
        }

        while cursor_char_idx > 0 && chars[cursor_char_idx].is_whitespace() {
            cursor_char_idx -= 1;
        }

        while cursor_char_idx > 0
            && (chars[cursor_char_idx - 1].is_alphanumeric() || chars[cursor_char_idx - 1] == '_')
        {
            cursor_char_idx -= 1;
        }

        buf.cursor_x = CursorOps::byte_index_of_char(line, cursor_char_idx);
    }

    pub fn move_word_end(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let mut cursor_char_idx = CursorOps::char_index_at_byte(line, buf.cursor_x);

        if cursor_char_idx + 1 < chars.len() {
            cursor_char_idx += 1;
        }

        while cursor_char_idx < chars.len() && chars[cursor_char_idx].is_whitespace() {
            cursor_char_idx += 1;
        }

        while cursor_char_idx + 1 < chars.len()
            && (chars[cursor_char_idx + 1].is_alphanumeric() || chars[cursor_char_idx + 1] == '_')
        {
            cursor_char_idx += 1;
        }

        buf.cursor_x = CursorOps::byte_index_of_char(line, cursor_char_idx);
    }

    pub fn move_to_first_non_blank(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return;
        }

        let line = &buf.lines[buf.cursor_y];
        let first_non_blank = line
            .char_indices()
            .find(|(_, c)| !c.is_whitespace())
            .map(|(i, _)| i)
            .unwrap_or(0);

        buf.cursor_x = first_non_blank;
    }

    pub fn find_char_forward(&mut self, target: char) -> bool {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return false;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let cursor_char_idx = CursorOps::char_index_at_byte(line, buf.cursor_x);

        for i in (cursor_char_idx + 1)..chars.len() {
            if chars[i] == target {
                buf.cursor_x = CursorOps::byte_index_of_char(line, i);
                return true;
            }
        }
        false
    }

    pub fn find_char_backward(&mut self, target: char) -> bool {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return false;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let cursor_char_idx = CursorOps::char_index_at_byte(line, buf.cursor_x);

        for i in (0..cursor_char_idx).rev() {
            if chars[i] == target {
                buf.cursor_x = CursorOps::byte_index_of_char(line, i);
                return true;
            }
        }
        false
    }

    pub fn find_char_till_forward(&mut self, target: char) -> bool {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return false;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let cursor_char_idx = CursorOps::char_index_at_byte(line, buf.cursor_x);

        for i in (cursor_char_idx + 1)..chars.len() {
            if chars[i] == target {
                buf.cursor_x = CursorOps::byte_index_of_char(line, i.saturating_sub(1));
                return true;
            }
        }
        false
    }

    pub fn find_char_till_backward(&mut self, target: char) -> bool {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return false;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let cursor_char_idx = CursorOps::char_index_at_byte(line, buf.cursor_x);

        for i in (0..cursor_char_idx).rev() {
            if chars[i] == target {
                buf.cursor_x = CursorOps::byte_index_of_char(line, (i + 1).min(chars.len() - 1));
                return true;
            }
        }
        false
    }

    pub fn find_matching_bracket(&mut self) -> bool {
        if let Some((line, col)) = self.get_matching_bracket_pos() {
            let buf = self.buf_mut();
            buf.cursor_y = line;
            buf.cursor_x = col;
            true
        } else {
            false
        }
    }

    pub fn get_matching_bracket_pos(&self) -> Option<(usize, usize)> {
        let buf = self.buf();
        if buf.cursor_y >= buf.lines.len() {
            return None;
        }

        let line = &buf.lines[buf.cursor_y];
        if buf.cursor_x >= line.len() {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        let cursor_char_idx = CursorOps::char_index_at_byte(line, buf.cursor_x);

        if cursor_char_idx >= chars.len() {
            return None;
        }

        let ch = chars[cursor_char_idx];
        let (opening, closing, direction) = match ch {
            '(' => ('(', ')', 1),
            ')' => ('(', ')', -1),
            '[' => ('[', ']', 1),
            ']' => ('[', ']', -1),
            '{' => ('{', '}', 1),
            '}' => ('{', '}', -1),
            _ => return None,
        };

        let mut depth = 0;
        let mut current_line = buf.cursor_y;
        let mut current_col_char = cursor_char_idx;

        loop {
            if current_line >= buf.lines.len() {
                break;
            }

            let line = &buf.lines[current_line];
            let chars: Vec<char> = line.chars().collect();

            if direction == 1 {
                while current_col_char < chars.len() {
                    if chars[current_col_char] == opening {
                        depth += 1;
                    } else if chars[current_col_char] == closing {
                        depth -= 1;
                        if depth == 0 {
                            return Some((
                                current_line,
                                CursorOps::byte_index_of_char(line, current_col_char),
                            ));
                        }
                    }
                    current_col_char += 1;
                }
                current_line += 1;
                current_col_char = 0;
            } else {
                while current_col_char > 0 {
                    current_col_char -= 1;
                    if chars[current_col_char] == closing {
                        depth += 1;
                    } else if chars[current_col_char] == opening {
                        depth -= 1;
                        if depth == 0 {
                            return Some((
                                current_line,
                                CursorOps::byte_index_of_char(line, current_col_char),
                            ));
                        }
                    }
                }
                if current_line == 0 {
                    break;
                }
                current_line -= 1;
                current_col_char = buf.lines[current_line].chars().count();
            }

            if (direction == 1 && current_line >= buf.lines.len())
                || (direction == -1 && current_line == 0 && current_col_char == 0)
            {
                break;
            }
        }

        None
    }

    // Compatibility aliases for selection
    pub fn start_visual_selection(&mut self) {
        self.start_selection();
    }

    pub fn update_visual_selection(&mut self) {
        self.update_selection();
    }

    pub fn clear_visual_selection(&mut self) {
        self.clear_selection();
    }

    pub fn start_visual_line_selection(&mut self) {
        self.start_selection();
    }

    pub fn update_visual_line_selection(&mut self) {
        self.update_selection();
    }
}

impl std::ops::Deref for EditorState {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        self.buf()
    }
}

// Re-export the render function from editor_render module
pub use crate::ui::editor_render::render;
