use anyhow::Result;
use ropey::Rope;
use std::fs;
use std::path::PathBuf;

/// A single buffer representing an open file
/// Now using ropey::Rope for proper text editing semantics
#[derive(Debug, Clone)]
pub struct Buffer {
    text: Rope,  // Private - use methods to access
    pub cursor_x: usize,  // Byte position in current line
    pub cursor_y: usize,  // Line number
    pub scroll_offset: usize,
    pub file_path: Option<PathBuf>,
    pub modified: bool,
    // Selection state for visual mode
    pub selection_start: Option<(usize, usize)>, // (line, col_byte)
    pub selection_end: Option<(usize, usize)>,   // (line, col_byte)
    
    // COMPATIBILITY: Provide Vec<String> interface for existing code
    pub lines: Vec<String>,  // Cached copy of lines for compatibility
    lines_dirty: bool,  // Track if cache needs refresh
}

impl Buffer {
    pub fn new() -> Self {
        let text = Rope::from("\n");
        let lines = vec![String::new()];
        Self {
            text,
            cursor_x: 0,
            cursor_y: 0,
            scroll_offset: 0,
            file_path: None,
            modified: false,
            selection_start: None,
            selection_end: None,
            lines,
            lines_dirty: false,
        }
    }
    
    /// Sync lines cache from rope (call after modifications)
    pub fn sync_lines(&mut self) {
        self.lines = self.text
            .lines()
            .map(|line| {
                let s = line.to_string();
                // Remove trailing \n if present
                s.trim_end_matches('\n').to_string()
            })
            .collect();
        self.lines_dirty = false;
    }
    
    /// Sync rope from lines cache (call after modifying lines)
    pub fn sync_rope(&mut self) {
        self.text = Rope::from(self.lines.join("\n") + "\n");
        self.lines_dirty = false;
    }
    
    /// Get reference to the rope (for advanced operations)
    pub fn text(&self) -> &Rope {
        &self.text
    }
    
    /// Get mutable reference to the rope (for advanced operations)
    pub fn text_mut(&mut self) -> &mut Rope {
        self.lines_dirty = true;
        &mut self.text
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

        // Create rope from file content
        let text = if content.is_empty() {
            Rope::from("\n")  // Empty file = one blank line
        } else {
            Rope::from(content)
        };

        // Create lines cache
        let lines: Vec<String> = text
            .lines()
            .map(|line| {
                let s = line.to_string();
                s.trim_end_matches('\n').to_string()
            })
            .collect();
        
        Ok(Self {
            text,
            cursor_x: 0,
            cursor_y: 0,
            scroll_offset: 0,
            file_path: Some(path.clone()),
            modified: false,
            selection_start: None,
            selection_end: None,
            lines,
            lines_dirty: false,
        })
    }

    pub fn get_content(&self) -> String {
        self.text.to_string()
    }

    pub fn filename(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("[untitled]"))
    }

    /// Helper: Get line count
    pub fn len_lines(&self) -> usize {
        self.text.len_lines()
    }

    /// Helper: Get a specific line as a string
    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx < self.text.len_lines() {
            Some(self.text.line(line_idx).to_string())
        } else {
            None
        }
    }

    /// Helper: Get line slice for rendering
    pub fn line_slice(&self, line_idx: usize) -> Option<ropey::RopeSlice> {
        if line_idx < self.text.len_lines() {
            Some(self.text.line(line_idx))
        } else {
            None
        }
    }
}
