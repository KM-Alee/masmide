use std::collections::VecDeque;

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
    InsertText {
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
        text: String,
    },
    DeleteText {
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
        text: String,
    },
    Batch(Vec<EditorAction>),
}

/// Undo/Redo stack for editor actions using VecDeque for O(1) front removal
#[derive(Debug, Clone)]
pub struct UndoStack {
    pub(crate) undo_stack: VecDeque<EditorAction>,
    pub(crate) redo_stack: VecDeque<EditorAction>,
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
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
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
