use super::buffer::Buffer;

/// Search state and operations
pub struct SearchState {
    pub query: String,
    pub matches: Vec<(usize, usize)>, // (line, col)
    pub current_match: usize,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            matches: Vec::new(),
            current_match: 0,
        }
    }

    /// Perform search and populate matches
    pub fn search(&mut self, query: &str, buf: &Buffer) {
        self.query = query.to_string();
        self.matches.clear();
        self.current_match = 0;

        if query.is_empty() {
            return;
        }

        let query_lower = query.to_lowercase();
        for (line_idx, line) in buf.lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            let mut start = 0;
            while let Some(pos) = line_lower[start..].find(&query_lower) {
                self.matches.push((line_idx, start + pos));
                start += pos + 1;
            }
        }
    }

    /// Move to next match
    pub fn find_next(&mut self, buf: &mut Buffer) {
        if self.matches.is_empty() {
            return;
        }
        self.current_match = (self.current_match + 1) % self.matches.len();
        self.jump_to_current_match(buf);
    }

    /// Move to previous match
    pub fn find_prev(&mut self, buf: &mut Buffer) {
        if self.matches.is_empty() {
            return;
        }
        self.current_match = if self.current_match == 0 {
            self.matches.len() - 1
        } else {
            self.current_match - 1
        };
        self.jump_to_current_match(buf);
    }

    /// Jump cursor to current match
    fn jump_to_current_match(&self, buf: &mut Buffer) {
        if let Some(&(line, col)) = self.matches.get(self.current_match) {
            buf.cursor_y = line;
            buf.cursor_x = col;
        }
    }

    /// Clear search state
    pub fn clear(&mut self) {
        self.query.clear();
        self.matches.clear();
        self.current_match = 0;
    }

    /// Get search status string for display
    pub fn status(&self) -> Option<String> {
        if self.matches.is_empty() {
            if !self.query.is_empty() {
                Some(String::from("No matches"))
            } else {
                None
            }
        } else {
            Some(format!("{}/{}", self.current_match + 1, self.matches.len()))
        }
    }
}
