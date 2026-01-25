//! Autocomplete system for MASM assembly language

/// Kind of suggestion for display purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionKind {
    Keyword,
    Register,
    Directive,
    TypeKeyword,
    Label,
    Procedure,
    Macro,
}

impl SuggestionKind {
    pub fn icon(&self) -> &'static str {
        match self {
            SuggestionKind::Keyword => "K",
            SuggestionKind::Register => "R",
            SuggestionKind::Directive => "D",
            SuggestionKind::TypeKeyword => "T",
            SuggestionKind::Label => "L",
            SuggestionKind::Procedure => "P",
            SuggestionKind::Macro => "M",
        }
    }
}

/// A single autocomplete suggestion
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub text: String,
    pub kind: SuggestionKind,
    pub detail: Option<String>,
}

impl Suggestion {
    pub fn new(text: impl Into<String>, kind: SuggestionKind) -> Self {
        Self {
            text: text.into(),
            kind,
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// State for the autocomplete popup
#[derive(Debug, Clone)]
pub struct AutocompleteState {
    pub suggestions: Vec<Suggestion>,
    pub selected: usize,
    pub trigger_pos: (usize, usize), // (line, col) where autocomplete was triggered
    pub visible: bool,
    pub scroll_offset: usize,
    all_suggestions: Vec<Suggestion>, // Cached full list
}

impl Default for AutocompleteState {
    fn default() -> Self {
        Self::new()
    }
}

impl AutocompleteState {
    pub fn new() -> Self {
        let all_suggestions = Self::build_suggestion_cache();
        Self {
            suggestions: Vec::new(),
            selected: 0,
            trigger_pos: (0, 0),
            visible: false,
            scroll_offset: 0,
            all_suggestions,
        }
    }

    fn build_suggestion_cache() -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // Keywords (instructions)
        for kw in KEYWORDS {
            suggestions.push(Suggestion::new(*kw, SuggestionKind::Keyword));
        }

        // Registers
        for reg in REGISTERS {
            suggestions.push(Suggestion::new(*reg, SuggestionKind::Register));
        }

        // Directives
        for dir in DIRECTIVES {
            suggestions.push(Suggestion::new(*dir, SuggestionKind::Directive));
        }

        // Type keywords
        for tk in TYPE_KEYWORDS {
            suggestions.push(Suggestion::new(*tk, SuggestionKind::TypeKeyword));
        }

        suggestions
    }

    /// Show autocomplete with suggestions filtered by prefix
    pub fn show(
        &mut self,
        prefix: &str,
        line: usize,
        col: usize,
        buffer_symbols: &[(String, SuggestionKind)],
    ) {
        self.trigger_pos = (line, col);
        self.selected = 0;
        self.scroll_offset = 0;

        let prefix_lower = prefix.to_lowercase();

        // Filter and collect matching suggestions
        let mut matches: Vec<(Suggestion, usize)> = Vec::new();

        // Add buffer symbols first (labels, procedures)
        for (name, kind) in buffer_symbols {
            if name.to_lowercase().starts_with(&prefix_lower) {
                let score = if name.to_lowercase() == prefix_lower {
                    0
                } else {
                    1
                };
                matches.push((Suggestion::new(name.clone(), *kind), score));
            }
        }

        // Add built-in suggestions
        for suggestion in &self.all_suggestions {
            if suggestion.text.to_lowercase().starts_with(&prefix_lower) {
                let score = if suggestion.text.to_lowercase() == prefix_lower {
                    0
                } else if suggestion.text.to_lowercase().starts_with(&prefix_lower) {
                    2
                } else {
                    3
                };
                matches.push((suggestion.clone(), score));
            }
        }

        // Sort by score (exact match first), then alphabetically
        matches.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.text.cmp(&b.0.text)));

        // Deduplicate by text
        let mut seen = std::collections::HashSet::new();
        self.suggestions = matches
            .into_iter()
            .filter(|(s, _)| seen.insert(s.text.to_lowercase()))
            .map(|(s, _)| s)
            .collect();

        self.visible = !self.suggestions.is_empty();
    }

    /// Hide the autocomplete popup
    pub fn hide(&mut self) {
        self.visible = false;
        self.suggestions.clear();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if !self.suggestions.is_empty() {
            if self.selected == 0 {
                self.selected = self.suggestions.len() - 1;
            } else {
                self.selected -= 1;
            }
            self.adjust_scroll();
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected = (self.selected + 1) % self.suggestions.len();
            self.adjust_scroll();
        }
    }

    fn adjust_scroll(&mut self) {
        const MAX_VISIBLE: usize = 10;
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + MAX_VISIBLE {
            self.scroll_offset = self.selected - MAX_VISIBLE + 1;
        }
    }

    /// Get the currently selected suggestion
    pub fn get_selected(&self) -> Option<&Suggestion> {
        self.suggestions.get(self.selected)
    }

    /// Get visible suggestions (max 10)
    pub fn visible_suggestions(&self) -> &[Suggestion] {
        const MAX_VISIBLE: usize = 10;
        let end = (self.scroll_offset + MAX_VISIBLE).min(self.suggestions.len());
        &self.suggestions[self.scroll_offset..end]
    }

    /// Get adjusted selected index for visible range
    pub fn visible_selected(&self) -> usize {
        self.selected.saturating_sub(self.scroll_offset)
    }
}

/// Parse buffer content to extract labels and procedures
pub fn parse_buffer_symbols(lines: &[String]) -> Vec<(String, SuggestionKind)> {
    let mut symbols = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }

        // Check for label (word followed by colon)
        if let Some(colon_pos) = trimmed.find(':') {
            let potential_label = trimmed[..colon_pos].trim();
            if is_valid_identifier(potential_label) && !potential_label.starts_with('.') {
                symbols.push((potential_label.to_string(), SuggestionKind::Label));
            }
        }

        // Check for procedure (word PROC)
        let upper = trimmed.to_uppercase();
        if upper.contains(" PROC") || upper.ends_with(" PROC") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if let Some(name) = parts.first() {
                if is_valid_identifier(name) {
                    symbols.push((name.to_string(), SuggestionKind::Procedure));
                }
            }
        }

        // Check for macro (word MACRO)
        if upper.contains(" MACRO") || upper.ends_with(" MACRO") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if let Some(name) = parts.first() {
                if is_valid_identifier(name) {
                    symbols.push((name.to_string(), SuggestionKind::Macro));
                }
            }
        }
    }

    symbols
}

fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' && first != '@' {
        return false;
    }
    s.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '@' || c == '?')
}

use crate::masm_lang::{DIRECTIVES, KEYWORDS, REGISTERS, TYPE_KEYWORDS};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_labels() {
        let lines = vec![
            "main:".to_string(),
            "    mov eax, 1".to_string(),
            "loop_start:".to_string(),
        ];
        let symbols = parse_buffer_symbols(&lines);
        assert!(symbols
            .iter()
            .any(|(n, k)| n == "main" && *k == SuggestionKind::Label));
        assert!(symbols
            .iter()
            .any(|(n, k)| n == "loop_start" && *k == SuggestionKind::Label));
    }

    #[test]
    fn test_parse_procedures() {
        let lines = vec![
            "MyProc PROC".to_string(),
            "    ret".to_string(),
            "MyProc ENDP".to_string(),
        ];
        let symbols = parse_buffer_symbols(&lines);
        assert!(symbols
            .iter()
            .any(|(n, k)| n == "MyProc" && *k == SuggestionKind::Procedure));
    }

    #[test]
    fn test_filter_suggestions() {
        let mut state = AutocompleteState::new();
        state.show("mo", 0, 0, &[]);
        assert!(state.suggestions.iter().any(|s| s.text == "mov"));
        assert!(state.suggestions.iter().any(|s| s.text == "movsx"));
    }
}
