use anyhow::Result;
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{Block, Borders, Paragraph},
};
use std::fs;
use std::path::PathBuf;

use crate::diagnostics::{Diagnostic, DiagnosticSeverity};
use crate::syntax::Highlighter;
use crate::theme::Theme;

/// Represents a single editor action for undo/redo
#[derive(Debug, Clone)]
pub enum EditorAction {
    InsertChar {
        line: usize,
        col: usize,
        ch: char,
    },
    DeleteChar {
        line: usize,
        col: usize,
        ch: char,
    },
    InsertLine {
        line_num: usize,
        content: String,
    },
    DeleteLine {
        line_num: usize,
        content: String,
    },
    ReplaceLine {
        line_num: usize,
        old: String,
        new: String,
    },
    SplitLine {
        line: usize,
        col: usize,
    },
    JoinLines {
        line: usize,
        col: usize,
        deleted_content: String,
    },
    Batch(Vec<EditorAction>),
}

/// Undo/Redo stack for editor actions using VecDeque for O(1) front removal
#[derive(Debug, Clone)]
pub struct UndoStack {
    undo_stack: std::collections::VecDeque<EditorAction>,
    redo_stack: std::collections::VecDeque<EditorAction>,
    max_size: usize,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl UndoStack {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: std::collections::VecDeque::new(),
            redo_stack: std::collections::VecDeque::new(),
            max_size,
        }
    }

    pub fn push(&mut self, action: EditorAction) {
        self.undo_stack.push_back(action);
        self.redo_stack.clear(); // Clear redo on new action

        // Trim from front if exceeds max size - O(1) with VecDeque
        while self.undo_stack.len() > self.max_size {
            self.undo_stack.pop_front();
        }
    }

    pub fn pop_undo(&mut self) -> Option<EditorAction> {
        self.undo_stack.pop_back()
    }

    pub fn push_redo(&mut self, action: EditorAction) {
        self.redo_stack.push_back(action);
    }

    pub fn pop_redo(&mut self) -> Option<EditorAction> {
        self.redo_stack.pop_back()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

/// A single buffer representing an open file
#[derive(Debug, Clone)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub scroll_offset: usize,
    pub file_path: Option<PathBuf>,
    pub modified: bool,
    // Selection state for visual mode
    pub selection_start: Option<(usize, usize)>, // (line, col)
    pub selection_end: Option<(usize, usize)>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            scroll_offset: 0,
            file_path: None,
            modified: false,
            selection_start: None,
            selection_end: None,
        }
    }

    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let metadata = fs::metadata(path)?;
        if metadata.len() > 10 * 1024 * 1024 {
            return Err(anyhow::anyhow!("File too large to open (max 10MB)"));
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                // If it's an encoding error, it's likely binary
                if e.kind() == std::io::ErrorKind::InvalidData {
                    return Err(anyhow::anyhow!("Cannot open binary file"));
                }
                return Err(e.into());
            }
        };

        // Double check for null bytes which might indicate binary content even if valid UTF-8 (rare but possible)
        if content.contains('\0') {
            return Err(anyhow::anyhow!("Cannot open binary file"));
        }

        let lines: Vec<String> = content.lines().map(String::from).collect();
        Ok(Self {
            lines: if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            },
            cursor_x: 0,
            cursor_y: 0,
            scroll_offset: 0,
            file_path: Some(path.clone()),
            modified: false,
            selection_start: None,
            selection_end: None,
        })
    }

    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn filename(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("[untitled]"))
    }
}

pub struct EditorState {
    pub buffers: Vec<Buffer>,
    pub active_buffer: usize,
    pub tab_size: usize,
    pub auto_indent: bool,
    // Search state
    pub search_query: String,
    pub search_matches: Vec<(usize, usize)>, // (line, col)
    pub current_match: usize,
    // Undo/Redo
    pub undo_stack: UndoStack,
    // Clipboard
    pub clipboard: Option<arboard::Clipboard>,
    pub yank_buffer: String,
    // Jump stack for go-to-definition navigation
    pub jump_stack: Vec<(PathBuf, usize, usize)>, // (file, line, col)
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
            clipboard: arboard::Clipboard::new().ok(),
            yank_buffer: String::new(),
            jump_stack: Vec::new(),
        }
    }

    // Accessor for current buffer
    fn buf(&self) -> &Buffer {
        &self.buffers[self.active_buffer]
    }

    fn buf_mut(&mut self) -> &mut Buffer {
        &mut self.buffers[self.active_buffer]
    }

    // Public accessors that delegate to current buffer
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

    // Compatibility shims for existing code
    #[allow(non_snake_case)]
    pub fn get_cursor_x(&self) -> usize {
        self.cursor_x()
    }
    #[allow(non_snake_case)]
    pub fn get_cursor_y(&self) -> usize {
        self.cursor_y()
    }

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

    pub fn insert_char(&mut self, c: char) {
        let (should_push, line_num, col) = {
            let buf = self.buf_mut();
            if buf.cursor_y < buf.lines.len() {
                let line = &mut buf.lines[buf.cursor_y];
                if buf.cursor_x <= line.len() {
                    let ln = buf.cursor_y;
                    let cl = buf.cursor_x;
                    line.insert(buf.cursor_x, c);
                    buf.cursor_x += 1;
                    buf.modified = true;
                    (true, ln, cl)
                } else {
                    (false, 0, 0)
                }
            } else {
                (false, 0, 0)
            }
        };
        if should_push {
            self.undo_stack.push(EditorAction::InsertChar {
                line: line_num,
                col,
                ch: c,
            });
        }
        self.clear_search();
    }

    pub fn insert_newline(&mut self) {
        self.insert_newline_with_indent(self.auto_indent);
    }

    pub fn insert_newline_with_indent(&mut self, auto_indent: bool) {
        let (line_num, col) = {
            let buf = self.buf_mut();
            if buf.cursor_y >= buf.lines.len() {
                return;
            }

            let ln = buf.cursor_y;
            let cl = buf.cursor_x;
            let current_line = &buf.lines[buf.cursor_y];
            let remainder = current_line[buf.cursor_x..].to_string();
            buf.lines[buf.cursor_y] = current_line[..buf.cursor_x].to_string();

            // Calculate indentation
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
            (ln, cl)
        };

        self.undo_stack.push(EditorAction::SplitLine {
            line: line_num,
            col,
        });
        self.clear_search();
    }

    fn calculate_indent(line: &str) -> String {
        // Get leading whitespace
        let leading_ws: String = line.chars().take_while(|c| c.is_whitespace()).collect();

        // Check if line ends with something that should increase indent
        let trimmed = line.trim().to_lowercase();
        let increase_indent = trimmed.ends_with("proc")
            || trimmed.ends_with("macro")
            || trimmed.ends_with(':')  // Labels
            || trimmed.starts_with(".data")
            || trimmed.starts_with(".code");

        if increase_indent {
            format!("{}    ", leading_ws)
        } else {
            leading_ws
        }
    }

    pub fn backspace(&mut self) {
        let action = {
            let buf = self.buf_mut();
            if buf.cursor_x > 0 {
                let line = &mut buf.lines[buf.cursor_y];
                let ch = line.chars().nth(buf.cursor_x - 1).unwrap_or(' ');
                let line_num = buf.cursor_y;
                let col = buf.cursor_x - 1;
                line.remove(buf.cursor_x - 1);
                buf.cursor_x -= 1;
                buf.modified = true;
                Some(EditorAction::DeleteChar {
                    line: line_num,
                    col,
                    ch,
                })
            } else if buf.cursor_y > 0 {
                let current_line = buf.lines.remove(buf.cursor_y);
                let line_num = buf.cursor_y;
                buf.cursor_y -= 1;
                buf.cursor_x = buf.lines[buf.cursor_y].len();
                let col = buf.cursor_x;
                buf.lines[buf.cursor_y].push_str(&current_line);
                buf.modified = true;
                Some(EditorAction::JoinLines {
                    line: line_num - 1,
                    col,
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
            if buf.cursor_y < buf.lines.len() {
                let line_len = buf.lines[buf.cursor_y].len();
                if buf.cursor_x < line_len {
                    let ch = buf.lines[buf.cursor_y]
                        .chars()
                        .nth(buf.cursor_x)
                        .unwrap_or(' ');
                    let line_num = buf.cursor_y;
                    let col = buf.cursor_x;
                    buf.lines[buf.cursor_y].remove(buf.cursor_x);
                    buf.modified = true;
                    Some(EditorAction::DeleteChar {
                        line: line_num,
                        col,
                        ch,
                    })
                } else if buf.cursor_y + 1 < buf.lines.len() {
                    let next_line = buf.lines.remove(buf.cursor_y + 1);
                    let line_num = buf.cursor_y;
                    let col = buf.lines[buf.cursor_y].len();
                    buf.lines[buf.cursor_y].push_str(&next_line);
                    buf.modified = true;
                    Some(EditorAction::JoinLines {
                        line: line_num,
                        col,
                        deleted_content: next_line,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(act) = action {
            self.undo_stack.push(act);
        }
        self.clear_search();
    }

    pub fn move_cursor_up(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y > 0 {
            buf.cursor_y -= 1;
            Self::clamp_cursor_x_internal(buf);
        }
    }

    pub fn move_cursor_down(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y + 1 < buf.lines.len() {
            buf.cursor_y += 1;
            Self::clamp_cursor_x_internal(buf);
        }
    }

    pub fn move_cursor_left(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_x > 0 {
            buf.cursor_x -= 1;
        } else if buf.cursor_y > 0 {
            buf.cursor_y -= 1;
            buf.cursor_x = buf.lines[buf.cursor_y].len();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y < buf.lines.len() {
            let line_len = buf.lines[buf.cursor_y].len();
            if buf.cursor_x < line_len {
                buf.cursor_x += 1;
            } else if buf.cursor_y + 1 < buf.lines.len() {
                buf.cursor_y += 1;
                buf.cursor_x = 0;
            }
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.buf_mut().cursor_x = 0;
    }

    pub fn move_to_line_end(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y < buf.lines.len() {
            buf.cursor_x = buf.lines[buf.cursor_y].len();
        }
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
                Self::clamp_cursor_x_internal(buf);
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

        self.yank_buffer = content.clone() + "\n";
        self.set_system_clipboard(&self.yank_buffer.clone());

        if was_single {
            if !content.is_empty() {
                self.undo_stack.push(EditorAction::ReplaceLine {
                    line_num,
                    old: content,
                    new: String::new(),
                });
            }
        } else {
            self.undo_stack
                .push(EditorAction::DeleteLine { line_num, content });
        }
        self.clear_search();
    }

    fn clamp_cursor_x_internal(buf: &mut Buffer) {
        if buf.cursor_y < buf.lines.len() {
            let line_len = buf.lines[buf.cursor_y].len();
            if buf.cursor_x > line_len {
                buf.cursor_x = line_len;
            }
        }
    }

    pub fn ensure_cursor_visible(&mut self, visible_height: usize) {
        let buf = self.buf_mut();
        if buf.cursor_y < buf.scroll_offset {
            buf.scroll_offset = buf.cursor_y;
        } else if buf.cursor_y >= buf.scroll_offset + visible_height {
            buf.scroll_offset = buf.cursor_y - visible_height + 1;
        }
    }

    pub fn insert_tab(&mut self) {
        for _ in 0..self.tab_size {
            self.insert_char(' ');
        }
    }

    // Buffer management
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

    // Search functionality
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

    // ========== Undo/Redo ==========

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
                // Undo insert = delete
                if *line < buf.lines.len() && *col < buf.lines[*line].len() {
                    buf.lines[*line].remove(*col);
                    buf.cursor_y = *line;
                    buf.cursor_x = *col;
                    buf.modified = true;
                }
            }
            EditorAction::DeleteChar { line, col, ch } => {
                // Undo delete = insert
                if *line < buf.lines.len() {
                    buf.lines[*line].insert(*col, *ch);
                    buf.cursor_y = *line;
                    buf.cursor_x = col + 1;
                    buf.modified = true;
                }
            }
            EditorAction::InsertLine { line_num, .. } => {
                // Undo insert line = delete line
                if *line_num < buf.lines.len() {
                    buf.lines.remove(*line_num);
                    buf.cursor_y = line_num.saturating_sub(1);
                    buf.cursor_x = 0;
                    buf.modified = true;
                }
            }
            EditorAction::DeleteLine { line_num, content } => {
                // Undo delete line = insert line
                buf.lines.insert(*line_num, content.clone());
                buf.cursor_y = *line_num;
                buf.cursor_x = 0;
                buf.modified = true;
            }
            EditorAction::ReplaceLine { line_num, old, .. } => {
                // Undo replace = restore old
                if *line_num < buf.lines.len() {
                    buf.lines[*line_num] = old.clone();
                    buf.cursor_y = *line_num;
                    buf.cursor_x = old.len().min(buf.cursor_x);
                    buf.modified = true;
                }
            }
            EditorAction::SplitLine { line, col } => {
                // Undo split = join lines
                if *line + 1 < buf.lines.len() {
                    let next_line = buf.lines.remove(*line + 1);
                    // Remove auto-indent from joined content
                    let trimmed = next_line.trim_start();
                    buf.lines[*line].truncate(*col);
                    buf.lines[*line].push_str(trimmed);
                    buf.cursor_y = *line;
                    buf.cursor_x = *col;
                    buf.modified = true;
                }
            }
            EditorAction::JoinLines {
                line,
                col,
                deleted_content,
            } => {
                // Undo join = split back
                if *line < buf.lines.len() {
                    let remaining = buf.lines[*line][*col..].to_string();
                    buf.lines[*line].truncate(*col);
                    buf.lines.insert(*line + 1, deleted_content.clone());
                    // The remaining was appended to deleted_content, but we stored original
                    // Actually for join, the deleted_content was the line that got joined
                    buf.cursor_y = *line + 1;
                    buf.cursor_x = 0;
                    buf.modified = true;
                    // Fix: remaining was part of the operation
                    let _ = remaining; // Already handled
                }
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
                    buf.lines[*line].insert(*col, *ch);
                    buf.cursor_y = *line;
                    buf.cursor_x = col + 1;
                    buf.modified = true;
                }
            }
            EditorAction::DeleteChar { line, col, .. } => {
                if *line < buf.lines.len() && *col < buf.lines[*line].len() {
                    buf.lines[*line].remove(*col);
                    buf.cursor_y = *line;
                    buf.cursor_x = *col;
                    buf.modified = true;
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
                    buf.cursor_x = new.len().min(buf.cursor_x);
                    buf.modified = true;
                }
            }
            EditorAction::SplitLine { line, col } => {
                if *line < buf.lines.len() {
                    let remainder = buf.lines[*line][*col..].to_string();
                    buf.lines[*line].truncate(*col);
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
                    buf.cursor_x = *col;
                    buf.modified = true;
                }
            }
            EditorAction::Batch(actions) => {
                for action in actions.iter() {
                    self.apply_redo_action(action);
                }
            }
        }
    }

    // ========== Clipboard ==========

    fn set_system_clipboard(&mut self, text: &str) {
        if let Some(ref mut clipboard) = self.clipboard {
            let _ = clipboard.set_text(text.to_string());
        }
    }

    fn get_system_clipboard(&mut self) -> Option<String> {
        if let Some(ref mut clipboard) = self.clipboard {
            clipboard.get_text().ok()
        } else {
            None
        }
    }

    pub fn yank_line(&mut self) {
        let buf = self.buf();
        if buf.cursor_y < buf.lines.len() {
            let content = buf.lines[buf.cursor_y].clone() + "\n";
            self.yank_buffer = content.clone();
            self.set_system_clipboard(&content);
        }
    }

    pub fn paste_after(&mut self) {
        let text = self
            .get_system_clipboard()
            .unwrap_or_else(|| self.yank_buffer.clone());

        if text.is_empty() {
            return;
        }

        if text.ends_with('\n') {
            // Line paste - paste on next line
            let line_content = text.trim_end_matches('\n').to_string();
            let insert_at = {
                let buf = self.buf_mut();
                let at = buf.cursor_y + 1;
                buf.lines.insert(at, line_content.clone());
                buf.cursor_y = at;
                buf.cursor_x = 0;
                buf.modified = true;
                at
            };
            self.undo_stack.push(EditorAction::InsertLine {
                line_num: insert_at,
                content: line_content,
            });
        } else {
            // Character paste
            for ch in text.chars() {
                if ch == '\n' {
                    self.insert_newline();
                } else {
                    self.insert_char(ch);
                }
            }
        }
    }

    pub fn paste_before(&mut self) {
        let text = self
            .get_system_clipboard()
            .unwrap_or_else(|| self.yank_buffer.clone());

        if text.is_empty() {
            return;
        }

        if text.ends_with('\n') {
            // Line paste - paste on current line, push content down
            let line_content = text.trim_end_matches('\n').to_string();
            let insert_at = {
                let buf = self.buf_mut();
                let at = buf.cursor_y;
                buf.lines.insert(at, line_content.clone());
                buf.cursor_x = 0;
                buf.modified = true;
                at
            };
            self.undo_stack.push(EditorAction::InsertLine {
                line_num: insert_at,
                content: line_content,
            });
        } else {
            // Character paste at cursor
            for ch in text.chars() {
                if ch == '\n' {
                    self.insert_newline();
                } else {
                    self.insert_char(ch);
                }
            }
        }
    }

    // ========== Visual Mode ==========

    pub fn start_visual_selection(&mut self) {
        let buf = self.buf_mut();
        buf.selection_start = Some((buf.cursor_y, buf.cursor_x));
        buf.selection_end = Some((buf.cursor_y, buf.cursor_x));
    }

    pub fn start_visual_line_selection(&mut self) {
        let buf = self.buf_mut();
        buf.selection_start = Some((buf.cursor_y, 0));
        let line_len = buf.lines.get(buf.cursor_y).map(|l| l.len()).unwrap_or(0);
        buf.selection_end = Some((buf.cursor_y, line_len));
    }

    pub fn update_selection(&mut self) {
        let buf = self.buf_mut();
        if buf.selection_start.is_some() {
            buf.selection_end = Some((buf.cursor_y, buf.cursor_x));
        }
    }

    pub fn update_visual_line_selection(&mut self) {
        let buf = self.buf_mut();
        if let Some((start_line, _)) = buf.selection_start {
            let end_line = buf.cursor_y;
            let (first, last) = if start_line <= end_line {
                (start_line, end_line)
            } else {
                (end_line, start_line)
            };
            buf.selection_start = Some((first, 0));
            let line_len = buf.lines.get(last).map(|l| l.len()).unwrap_or(0);
            buf.selection_end = Some((last, line_len));
        }
    }

    pub fn clear_selection(&mut self) {
        let buf = self.buf_mut();
        buf.selection_start = None;
        buf.selection_end = None;
    }

    pub fn has_selection(&self) -> bool {
        let buf = self.buf();
        buf.selection_start.is_some() && buf.selection_end.is_some()
    }

    pub fn get_selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let buf = self.buf();
        match (buf.selection_start, buf.selection_end) {
            (Some(start), Some(end)) => {
                // Normalize: ensure start <= end
                if start.0 < end.0 || (start.0 == end.0 && start.1 <= end.1) {
                    Some((start, end))
                } else {
                    Some((end, start))
                }
            }
            _ => None,
        }
    }

    pub fn get_selected_text(&self) -> Option<String> {
        let ((start_line, start_col), (end_line, end_col)) = self.get_selection_range()?;
        let buf = self.buf();

        if start_line == end_line {
            // Single line selection
            let line = buf.lines.get(start_line)?;
            let end = end_col.min(line.len());
            let start = start_col.min(end);
            Some(line[start..end].to_string())
        } else {
            // Multi-line selection
            let mut result = String::new();
            for i in start_line..=end_line {
                if let Some(line) = buf.lines.get(i) {
                    if i == start_line {
                        result.push_str(&line[start_col.min(line.len())..]);
                        result.push('\n');
                    } else if i == end_line {
                        result.push_str(&line[..end_col.min(line.len())]);
                    } else {
                        result.push_str(line);
                        result.push('\n');
                    }
                }
            }
            Some(result)
        }
    }

    pub fn yank_selection(&mut self) -> bool {
        if let Some(text) = self.get_selected_text() {
            self.yank_buffer = text.clone();
            self.set_system_clipboard(&text);
            self.clear_selection();
            true
        } else {
            false
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        let range = match self.get_selection_range() {
            Some(r) => r,
            None => return false,
        };

        let ((start_line, start_col), (end_line, end_col)) = range;

        // Yank first
        if let Some(text) = self.get_selected_text() {
            self.yank_buffer = text.clone();
            self.set_system_clipboard(&text);
        }

        let buf = self.buf_mut();

        if start_line == end_line {
            // Single line deletion
            if let Some(line) = buf.lines.get_mut(start_line) {
                let end = end_col.min(line.len());
                let start = start_col.min(end);
                line.drain(start..end);
                buf.cursor_x = start;
                buf.cursor_y = start_line;
            }
        } else {
            // Multi-line deletion
            // Get the part before selection on first line
            let prefix = buf
                .lines
                .get(start_line)
                .map(|l| l[..start_col.min(l.len())].to_string())
                .unwrap_or_default();

            // Get the part after selection on last line
            let suffix = buf
                .lines
                .get(end_line)
                .map(|l| l[end_col.min(l.len())..].to_string())
                .unwrap_or_default();

            // Remove lines from end to start+1
            for _ in (start_line + 1..=end_line).rev() {
                if start_line + 1 < buf.lines.len() {
                    buf.lines.remove(start_line + 1);
                }
            }

            // Combine prefix and suffix on start line
            if start_line < buf.lines.len() {
                buf.lines[start_line] = prefix.clone() + &suffix;
            }

            buf.cursor_x = prefix.len();
            buf.cursor_y = start_line;
        }

        buf.modified = true;
        buf.selection_start = None;
        buf.selection_end = None;

        true
    }

    // ========== Vim Motions ==========

    pub fn move_word_forward(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let mut x = buf.cursor_x;

        // Skip current word (non-whitespace)
        while x < chars.len() && !chars[x].is_whitespace() {
            x += 1;
        }
        // Skip whitespace
        while x < chars.len() && chars[x].is_whitespace() {
            x += 1;
        }

        if x >= chars.len() && buf.cursor_y + 1 < buf.lines.len() {
            // Move to next line
            buf.cursor_y += 1;
            buf.cursor_x = 0;
            // Skip leading whitespace on new line
            let new_line = &buf.lines[buf.cursor_y];
            let new_chars: Vec<char> = new_line.chars().collect();
            let mut new_x = 0;
            while new_x < new_chars.len() && new_chars[new_x].is_whitespace() {
                new_x += 1;
            }
            buf.cursor_x = new_x;
        } else {
            buf.cursor_x = x.min(chars.len().saturating_sub(1).max(0));
        }
    }

    pub fn move_word_backward(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return;
        }

        if buf.cursor_x == 0 {
            // Move to end of previous line
            if buf.cursor_y > 0 {
                buf.cursor_y -= 1;
                buf.cursor_x = buf.lines[buf.cursor_y].len();
            }
            return;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let mut x = buf.cursor_x.saturating_sub(1);

        // Skip whitespace backwards
        while x > 0 && chars[x].is_whitespace() {
            x -= 1;
        }
        // Skip word backwards
        while x > 0 && !chars[x - 1].is_whitespace() {
            x -= 1;
        }

        buf.cursor_x = x;
    }

    pub fn move_word_end(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let mut x = buf.cursor_x;

        // Move at least one character
        if x < chars.len() {
            x += 1;
        }

        // Skip whitespace
        while x < chars.len() && chars[x].is_whitespace() {
            x += 1;
        }
        // Move to end of word
        while x < chars.len() && !chars[x].is_whitespace() {
            x += 1;
        }

        if x > 0 {
            x -= 1;
        }

        if x >= chars.len() && buf.cursor_y + 1 < buf.lines.len() {
            // Move to next line
            buf.cursor_y += 1;
            buf.cursor_x = 0;
        } else {
            buf.cursor_x = x;
        }
    }

    pub fn move_to_first_non_blank(&mut self) {
        let buf = self.buf_mut();
        if buf.cursor_y < buf.lines.len() {
            let line = &buf.lines[buf.cursor_y];
            let pos = line.chars().take_while(|c| c.is_whitespace()).count();
            buf.cursor_x = pos;
        }
    }

    pub fn find_matching_bracket(&mut self) -> bool {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return false;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();

        if buf.cursor_x >= chars.len() {
            return false;
        }

        let current = chars[buf.cursor_x];
        let (target, forward) = match current {
            '(' => (')', true),
            ')' => ('(', false),
            '[' => (']', true),
            ']' => ('[', false),
            '{' => ('}', true),
            '}' => ('{', false),
            '<' => ('>', true),
            '>' => ('<', false),
            _ => return false,
        };

        let mut depth = 1;
        let mut y = buf.cursor_y;
        let mut x = buf.cursor_x;

        if forward {
            x += 1;
            while y < buf.lines.len() {
                let search_line: Vec<char> = buf.lines[y].chars().collect();
                while x < search_line.len() {
                    if search_line[x] == current {
                        depth += 1;
                    } else if search_line[x] == target {
                        depth -= 1;
                        if depth == 0 {
                            buf.cursor_y = y;
                            buf.cursor_x = x;
                            return true;
                        }
                    }
                    x += 1;
                }
                y += 1;
                x = 0;
            }
        } else {
            if x == 0 {
                if y == 0 {
                    return false;
                }
                y -= 1;
                x = buf.lines[y].len();
            } else {
                x -= 1;
            }

            loop {
                let search_line: Vec<char> = buf.lines[y].chars().collect();
                while x < search_line.len() {
                    let check_x = if forward {
                        x
                    } else {
                        search_line.len() - 1 - x
                    };
                    if check_x < search_line.len() {
                        if search_line[x] == current {
                            depth += 1;
                        } else if search_line[x] == target {
                            depth -= 1;
                            if depth == 0 {
                                buf.cursor_y = y;
                                buf.cursor_x = x;
                                return true;
                            }
                        }
                    }
                    if x == 0 {
                        break;
                    }
                    x -= 1;
                }
                if y == 0 {
                    break;
                }
                y -= 1;
                x = buf.lines[y].len().saturating_sub(1);
            }
        }

        false
    }

    pub fn find_char_forward(&mut self, target: char) -> bool {
        let buf = self.buf_mut();
        if buf.cursor_y >= buf.lines.len() {
            return false;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();

        for x in (buf.cursor_x + 1)..chars.len() {
            if chars[x] == target {
                buf.cursor_x = x;
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

        for x in (0..buf.cursor_x).rev() {
            if chars[x] == target {
                buf.cursor_x = x;
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

        for x in (buf.cursor_x + 1)..chars.len() {
            if chars[x] == target {
                buf.cursor_x = x.saturating_sub(1);
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

        for x in (0..buf.cursor_x).rev() {
            if chars[x] == target {
                buf.cursor_x = x + 1;
                return true;
            }
        }
        false
    }

    pub fn go_to_line(&mut self, line_num: usize) {
        let buf = self.buf_mut();
        let target = line_num
            .saturating_sub(1)
            .min(buf.lines.len().saturating_sub(1));
        buf.cursor_y = target;
        buf.cursor_x = 0;
    }

    // ========== Go to Definition ==========

    /// Get the word under the cursor
    pub fn get_word_under_cursor(&self) -> Option<String> {
        let buf = self.buf();
        if buf.cursor_y >= buf.lines.len() {
            return None;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();

        if buf.cursor_x >= chars.len() {
            return None;
        }

        // Check if cursor is on a valid identifier character
        let c = chars[buf.cursor_x];
        if !c.is_alphanumeric() && c != '_' && c != '@' && c != '?' {
            return None;
        }

        // Find word start
        let mut start = buf.cursor_x;
        while start > 0 {
            let prev = chars[start - 1];
            if prev.is_alphanumeric() || prev == '_' || prev == '@' || prev == '?' {
                start -= 1;
            } else {
                break;
            }
        }

        // Find word end
        let mut end = buf.cursor_x;
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

    /// Find the definition of a symbol in the current buffer
    /// Returns (line_number, column) if found
    pub fn find_definition_in_buffer(&self, symbol: &str) -> Option<(usize, usize)> {
        let buf = self.buf();
        let symbol_lower = symbol.to_lowercase();

        for (line_idx, line) in buf.lines.iter().enumerate() {
            let trimmed = line.trim();
            let trimmed_lower = trimmed.to_lowercase();

            // Check for label: symbol followed by colon
            if let Some(colon_pos) = trimmed.find(':') {
                let label_part = trimmed[..colon_pos].trim();
                if label_part.to_lowercase() == symbol_lower {
                    // Find the actual position in the original line
                    let col = line.find(label_part).unwrap_or(0);
                    return Some((line_idx, col));
                }
            }

            // Check for procedure: symbol PROC
            if trimmed_lower.contains(" proc") || trimmed_lower.ends_with(" proc") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(name) = parts.first() {
                    if name.to_lowercase() == symbol_lower {
                        let col = line.find(*name).unwrap_or(0);
                        return Some((line_idx, col));
                    }
                }
            }

            // Check for macro: symbol MACRO
            if trimmed_lower.contains(" macro") || trimmed_lower.ends_with(" macro") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(name) = parts.first() {
                    if name.to_lowercase() == symbol_lower {
                        let col = line.find(*name).unwrap_or(0);
                        return Some((line_idx, col));
                    }
                }
            }

            // Check for EQU constant: symbol EQU value
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

    /// Push current position to jump stack and go to definition
    pub fn go_to_definition(&mut self) -> Option<String> {
        let word = self.get_word_under_cursor()?;

        if let Some((line, col)) = self.find_definition_in_buffer(&word) {
            // Save current position to jump stack
            if let Some(path) = self.current_file().cloned() {
                let cur_line = self.cursor_y();
                let cur_col = self.cursor_x();
                self.jump_stack.push((path, cur_line, cur_col));
            }

            // Jump to definition
            let buf = self.buf_mut();
            buf.cursor_y = line;
            buf.cursor_x = col;

            Some(word)
        } else {
            None
        }
    }

    /// Go back to previous position in jump stack
    pub fn go_back(&mut self) -> bool {
        if let Some((path, line, col)) = self.jump_stack.pop() {
            // Check if we need to switch buffers
            let current_path = self.current_file().cloned();
            if current_path.as_ref() != Some(&path) {
                // Try to find the buffer or open it
                let mut found_idx = None;
                for (idx, buf) in self.buffers.iter().enumerate() {
                    if buf.file_path.as_ref() == Some(&path) {
                        found_idx = Some(idx);
                        break;
                    }
                }

                if let Some(idx) = found_idx {
                    self.active_buffer = idx;
                }
            }

            let buf = self.buf_mut();
            buf.cursor_y = line;
            buf.cursor_x = col;
            true
        } else {
            false
        }
    }

    /// Get matching bracket/delimiter position for highlighting
    pub fn get_matching_bracket_pos(&self) -> Option<(usize, usize)> {
        let buf = self.buf();
        if buf.cursor_y >= buf.lines.len() {
            return None;
        }

        let line = &buf.lines[buf.cursor_y];
        let chars: Vec<char> = line.chars().collect();

        if buf.cursor_x >= chars.len() {
            return None;
        }

        let current = chars[buf.cursor_x];
        let (target, forward) = match current {
            '(' => (')', true),
            ')' => ('(', false),
            '[' => (']', true),
            ']' => ('[', false),
            '{' => ('}', true),
            '}' => ('{', false),
            '<' => ('>', true),
            '>' => ('<', false),
            _ => return None,
        };

        let mut depth = 1;
        let mut y = buf.cursor_y;
        let mut x = buf.cursor_x;

        if forward {
            x += 1;
            while y < buf.lines.len() {
                let search_line: Vec<char> = buf.lines[y].chars().collect();
                while x < search_line.len() {
                    if search_line[x] == current {
                        depth += 1;
                    } else if search_line[x] == target {
                        depth -= 1;
                        if depth == 0 {
                            return Some((y, x));
                        }
                    }
                    x += 1;
                }
                y += 1;
                x = 0;
            }
        } else {
            if x == 0 {
                if y == 0 {
                    return None;
                }
                y -= 1;
                x = buf.lines[y].len();
            } else {
                x -= 1;
            }

            loop {
                let search_line: Vec<char> = buf.lines[y].chars().collect();
                loop {
                    if x < search_line.len() {
                        if search_line[x] == current {
                            depth += 1;
                        } else if search_line[x] == target {
                            depth -= 1;
                            if depth == 0 {
                                return Some((y, x));
                            }
                        }
                    }
                    if x == 0 {
                        break;
                    }
                    x -= 1;
                }
                if y == 0 {
                    break;
                }
                y -= 1;
                x = buf.lines[y].len().saturating_sub(1);
            }
        }

        None
    }

    // Compatibility properties
    pub fn cursor_x_compat(&self) -> usize {
        self.cursor_x()
    }
    pub fn cursor_y_compat(&self) -> usize {
        self.cursor_y()
    }
}

// Compatibility layer - these are used by app.rs
impl EditorState {
    // Expose as public fields for compatibility
    #[inline]
    pub fn get_lines(&self) -> &Vec<String> {
        &self.buf().lines
    }
}

// Add property-like access
impl std::ops::Deref for EditorState {
    type Target = Buffer;
    fn deref(&self) -> &Self::Target {
        self.buf()
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &EditorState,
    focused: bool,
    theme: &Theme,
    diagnostics: &[Diagnostic],
    current_file: Option<&PathBuf>,
) {
    let buf = &state.buffers[state.active_buffer];
    let show_line_numbers = true;
    // Add extra space for diagnostic gutter indicator
    let line_number_width = if show_line_numbers {
        (buf.lines.len().to_string().len()).max(3) + 2 // +2 for space and diagnostic indicator
    } else {
        1 // Just diagnostic indicator
    };

    // Build a map of line numbers to diagnostics for the current file
    let diag_map: std::collections::HashMap<usize, &Diagnostic> = diagnostics
        .iter()
        .filter(|d| current_file.map_or(false, |f| &d.file == f))
        .map(|d| (d.line, d))
        .collect();

    let title = match &buf.file_path {
        Some(path) => {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if buf.modified {
                format!(" {} ● ", name)
            } else {
                format!(" {} ", name)
            }
        }
        None => String::from(" [No File] "),
    };

    let (border_style, title_style) = if focused {
        (
            Style::default().fg(theme.ui.border_focused.to_color()),
            Style::default()
                .fg(theme.ui.title_focused.to_color())
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            Style::default().fg(theme.ui.border.to_color()),
            Style::default().fg(theme.ui.title.to_color()),
        )
    };

    let block = Block::default()
        .title(Span::styled(title, title_style))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner.height as usize;

    // Get current match position for highlighting
    let current_match_pos: Vec<(usize, usize)> = if !state.search_matches.is_empty() {
        vec![state.search_matches[state.current_match]]
    } else {
        vec![]
    };

    // Get selection range for visual mode highlighting
    let selection_range = state.get_selection_range();

    let visible_lines: Vec<Line> = buf
        .lines
        .iter()
        .enumerate()
        .skip(buf.scroll_offset)
        .take(visible_height)
        .map(|(idx, line)| {
            let mut spans = Vec::new();
            let line_num_1based = idx + 1;

            // Check if this line has a diagnostic
            let line_diagnostic = diag_map.get(&line_num_1based);

            // Diagnostic gutter indicator
            let diag_indicator = match line_diagnostic {
                Some(d) if d.severity == DiagnosticSeverity::Error => Span::styled(
                    "● ",
                    Style::default().fg(theme.ui.diagnostic_error.to_color()),
                ),
                Some(d) if d.severity == DiagnosticSeverity::Warning => Span::styled(
                    "▲ ",
                    Style::default().fg(theme.ui.diagnostic_warning.to_color()),
                ),
                _ => Span::raw("  "),
            };
            spans.push(diag_indicator);

            // Line number
            if show_line_numbers {
                let num_width = line_number_width - 2; // Subtract diagnostic indicator width
                let line_num = format!("{:>width$} ", line_num_1based, width = num_width);
                spans.push(Span::styled(
                    line_num,
                    Style::default().fg(theme.ui.line_numbers.to_color()),
                ));
            }

            // Check if this line is part of a selection
            let line_selection =
                if let Some(((start_line, start_col), (end_line, end_col))) = selection_range {
                    if idx >= start_line && idx <= end_line {
                        let sel_start = if idx == start_line { start_col } else { 0 };
                        let sel_end = if idx == end_line { end_col } else { line.len() };
                        Some((sel_start, sel_end))
                    } else {
                        None
                    }
                } else {
                    None
                };

            // Syntax highlighted content with search and selection highlighting
            let search_query = if state.search_query.is_empty() {
                None
            } else {
                Some(state.search_query.as_str())
            };

            if let Some((sel_start, sel_end)) = line_selection {
                // Apply selection highlighting
                let chars: Vec<char> = line.chars().collect();
                let sel_start = sel_start.min(chars.len());
                let sel_end = sel_end.min(chars.len()).max(sel_start);

                // Before selection
                if sel_start > 0 {
                    let before: String = chars[..sel_start].iter().collect();
                    let highlighted = Highlighter::highlight_line_with_search(
                        &before,
                        &theme.syntax,
                        search_query,
                        &theme.ui.search_match,
                        &current_match_pos,
                        idx,
                        &theme.ui.search_match_current,
                    );
                    spans.extend(highlighted);
                }

                // Selected portion
                if sel_end > sel_start {
                    let selected: String = chars[sel_start..sel_end].iter().collect();
                    spans.push(Span::styled(
                        selected,
                        Style::default()
                            .bg(theme.ui.selection.to_color())
                            .fg(theme.ui.selection_fg.to_color()),
                    ));
                }

                // After selection
                if sel_end < chars.len() {
                    let after: String = chars[sel_end..].iter().collect();
                    let highlighted = Highlighter::highlight_line_with_search(
                        &after,
                        &theme.syntax,
                        search_query,
                        &theme.ui.search_match,
                        &current_match_pos,
                        idx,
                        &theme.ui.search_match_current,
                    );
                    spans.extend(highlighted);
                }
            } else {
                // No selection, just syntax highlight
                let highlighted = Highlighter::highlight_line_with_search(
                    line,
                    &theme.syntax,
                    search_query,
                    &theme.ui.search_match,
                    &current_match_pos,
                    idx,
                    &theme.ui.search_match_current,
                );
                spans.extend(highlighted);
            }

            Line::from(spans)
        })
        .collect();

    let paragraph =
        Paragraph::new(visible_lines).style(Style::default().bg(theme.ui.background.to_color()));
    frame.render_widget(paragraph, inner);

    if focused {
        let cursor_screen_y = buf.cursor_y.saturating_sub(buf.scroll_offset);
        let cursor_screen_x = line_number_width + 1 + buf.cursor_x;

        if cursor_screen_y < visible_height {
            frame.set_cursor_position(Position::new(
                inner.x + cursor_screen_x as u16,
                inner.y + cursor_screen_y as u16,
            ));
        }
    }
}
