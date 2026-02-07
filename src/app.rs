use crate::autocomplete::{parse_buffer_symbols, AutocompleteState};
use crate::build::Pipeline;
use crate::config::{Config, ProjectConfig};
use crate::diagnostics::{self, Diagnostic, DiagnosticSeverity};
use crate::docs::{self, DocEntry};
use crate::theme::Theme;
use crate::ui::editor::EditorState;
use crate::ui::file_tree::FileTreeState;
use crate::ui::output::OutputState;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    FileTree,
    Search,
    InputPopup,
    Visual,
    VisualLine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingAction {
    None,
    CreateFile,
    CreateDir,
    Rename,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Editor,
    FileTree,
    Output,
}

pub struct App {
    pub mode: Mode,
    pub focus: FocusedPanel,
    pub editor: EditorState,
    pub file_tree: FileTreeState,
    pub output: OutputState,
    pub command_input: String,
    pub search_input: String,
    pub input_popup_title: String,
    pub input_popup_value: String,
    pub pending_action: PendingAction,
    pub status_message: String,
    pub project_dir: PathBuf,
    pub config: Config,
    pub project_config: ProjectConfig,
    pub pipeline: Pipeline,
    pub last_build_success: bool,
    pub show_file_tree: bool,
    pub show_output: bool,
    pub show_help: bool,
    pub help_scroll: usize,
    pub output_only_mode: bool, // Full-screen output view
    pub file_tree_width: u16,
    pub output_height: u16,
    // Vim motion support
    pub pending_count: Option<usize>,
    pub pending_char: Option<char>,    // For f, F, t, T commands
    pub pending_g: bool,               // For gd (go to definition) command
    pub pending_bracket: Option<char>, // For ]e, [e (error navigation) commands
    // Autocomplete
    pub autocomplete: AutocompleteState,
    // Hover documentation
    pub show_hover: bool,
    pub hover_doc: Option<&'static DocEntry>,
    // Diagnostics (build errors/warnings)
    pub diagnostics: Vec<Diagnostic>,
    pub current_diagnostic: usize,
    // Autosave tracking
    pub last_save_time: std::time::Instant,
    pub autosave_enabled: bool,
}

impl App {
    pub fn new(path: PathBuf) -> Result<Self> {
        let config = Config::load()?;

        let project_dir = if path.is_file() {
            path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        } else {
            path.clone()
        };

        let project_config = ProjectConfig::load(&project_dir).unwrap_or_default();

        let file_to_open = if path.is_file() {
            Some(path)
        } else {
            let entry = project_dir.join(&project_config.entry_file);
            if entry.exists() {
                Some(entry)
            } else {
                None
            }
        };

        let mut editor = EditorState::new(config.editor.tab_size);
        editor.auto_indent = config.editor.auto_indent;

        let mut status_message =
            String::from("Press F1 for help | F5 build+run | F6 build | F7 run");

        if let Some(file_path) = file_to_open {
            match editor.open_file(&file_path) {
                Ok(_) => {
                    status_message = format!("Opened: {}", file_path.display());
                }
                Err(e) => {
                    status_message = format!("Failed to open {}: {}", file_path.display(), e);
                }
            }
        }

        let file_tree = FileTreeState::new(&project_dir)?;
        let output = OutputState::new();
        let pipeline = Pipeline::new(&config, &project_config, &project_dir);

        let file_tree_width = config.layout.file_tree_width;
        let output_height = config.layout.output_height;
        let autosave_enabled = config.editor.autosave;

        Ok(Self {
            mode: Mode::Normal,
            focus: FocusedPanel::Editor,
            editor,
            file_tree,
            output,
            command_input: String::new(),
            search_input: String::new(),
            input_popup_title: String::new(),
            input_popup_value: String::new(),
            pending_action: PendingAction::None,
            status_message,
            project_dir,
            config,
            project_config,
            pipeline,
            last_build_success: false,
            show_file_tree: true,
            show_output: true,
            show_help: false,
            help_scroll: 0,
            output_only_mode: false,
            file_tree_width,
            output_height,
            pending_count: None,
            pending_char: None,
            pending_g: false,
            pending_bracket: None,
            autocomplete: AutocompleteState::new(),
            show_hover: false,
            hover_doc: None,
            diagnostics: Vec::new(),
            current_diagnostic: 0,
            last_save_time: std::time::Instant::now(),
            autosave_enabled,
        })
    }

    pub fn theme(&self) -> &Theme {
        &self.config.theme
    }

    pub fn set_theme(&mut self, name: &str) {
        self.config.set_theme(name);
        self.status_message = format!("Theme changed to: {}", name);
    }

    pub fn increase_file_tree_width(&mut self) {
        let max = self.config.layout.file_tree_max_width;
        if self.file_tree_width < max {
            self.file_tree_width += 2;
        }
    }

    pub fn decrease_file_tree_width(&mut self) {
        let min = self.config.layout.file_tree_min_width;
        if self.file_tree_width > min {
            self.file_tree_width -= 2;
        }
    }

    pub fn increase_output_height(&mut self) {
        let max = self.config.layout.output_max_height;
        if self.output_height < max {
            self.output_height += 2;
        }
    }

    pub fn decrease_output_height(&mut self) {
        let min = self.config.layout.output_min_height;
        if self.output_height > min {
            self.output_height -= 2;
        }
    }

    pub fn start_search(&mut self) {
        self.mode = Mode::Search;
        self.search_input.clear();
    }

    pub fn execute_search(&mut self) {
        self.editor.search(&self.search_input);
        if let Some(status) = self.editor.search_status() {
            self.status_message = format!("Search: {} - {}", self.search_input, status);
        }
        self.mode = Mode::Normal;
    }

    pub fn cancel_search(&mut self) {
        self.search_input.clear();
        self.editor.clear_search();
        self.mode = Mode::Normal;
    }

    pub fn execute_input_popup(&mut self) -> Result<()> {
        let value = self.input_popup_value.trim().to_string();
        self.input_popup_value.clear();

        match self.pending_action {
            PendingAction::CreateFile => {
                if !value.is_empty() {
                    self.file_tree.create_file(&value)?;
                    self.status_message = format!("Created file: {}", value);
                }
            }
            PendingAction::CreateDir => {
                if !value.is_empty() {
                    self.file_tree.create_dir(&value)?;
                    self.status_message = format!("Created directory: {}", value);
                }
            }
            PendingAction::Rename => {
                if !value.is_empty() {
                    self.file_tree.rename_current(&value)?;
                    self.status_message = format!("Renamed to: {}", value);
                }
            }
            PendingAction::Delete => {
                if value.to_lowercase() == "y" {
                    self.file_tree.delete_current()?;
                    self.status_message = String::from("Deleted item");
                } else {
                    self.status_message = String::from("Deletion cancelled");
                }
            }
            PendingAction::None => {}
        }

        self.pending_action = PendingAction::None;
        self.mode = Mode::FileTree; // Return to file tree
        Ok(())
    }

    pub fn cancel_input_popup(&mut self) {
        self.input_popup_value.clear();
        self.pending_action = PendingAction::None;
        self.mode = Mode::FileTree;
        self.status_message = String::from("Cancelled");
    }

    pub fn build(&mut self) -> Result<()> {
        self.output.clear();
        self.diagnostics.clear();
        self.current_diagnostic = 0;
        self.status_message = String::from("Building...");

        let source_path = match self.editor.current_file().cloned() {
            Some(p) => p,
            None => {
                self.output.append_error("No file open to build");
                self.status_message = String::from("Build failed: no file open");
                self.last_build_success = false;
                return Ok(());
            }
        };

        // Save before building
        self.save_current_file()?;

        match self.pipeline.build(&source_path) {
            Ok(build_output) => {
                // Parse diagnostics from both stdout and stderr (JWasm writes to both)
                let mut all_diagnostics =
                    diagnostics::parse_jwasm_output(&build_output.stdout, &self.project_dir);
                all_diagnostics.extend(diagnostics::parse_jwasm_output(
                    &build_output.stderr,
                    &self.project_dir,
                ));
                self.diagnostics = all_diagnostics;

                let (errors, warnings) = diagnostics::count_by_severity(&self.diagnostics);
                self.last_build_success = build_output.success;

                if build_output.success {
                    // Show success message
                    self.output.append_success(&build_output.stdout);
                    if warnings > 0 {
                        self.output.append_stderr(&build_output.stderr);
                        self.status_message = format!(
                            "Build successful ({} warning{})",
                            warnings,
                            if warnings == 1 { "" } else { "s" }
                        );
                    } else {
                        self.status_message = String::from("Build successful");
                    }
                } else {
                    // Show errors
                    if !build_output.stderr.is_empty() {
                        self.output.append_stderr(&build_output.stderr);
                    }
                    self.status_message = format!(
                        "Build failed: {} error{}, {} warning{}",
                        errors,
                        if errors == 1 { "" } else { "s" },
                        warnings,
                        if warnings == 1 { "" } else { "s" }
                    );
                };
            }
            Err(e) => {
                self.output.append_error(&format!("{e}"));
                self.status_message = String::from("Build failed");
                self.last_build_success = false;
            }
        }

        self.show_output = true;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        self.status_message = String::from("Running...");

        // Add blank line separator if there's already output from build
        if !self.output.is_empty() {
            self.output.append_divider();
        }

        match self.pipeline.run() {
            Ok(run_output) => {
                // Show program output
                let trimmed_stdout = run_output.stdout.trim();
                if !trimmed_stdout.is_empty() {
                    self.output.append_stdout(trimmed_stdout);
                }
                if !run_output.stderr.is_empty() {
                    self.output.append_stderr(&run_output.stderr);
                }
                // Show exit status in status bar only, not in output panel
                if run_output.exit_code == 0 {
                    self.status_message = String::from("Program finished");
                } else {
                    self.status_message = format!("Exit code {}", run_output.exit_code);
                }
            }
            Err(e) => {
                self.output.append_error(&format!("{e}"));
                self.status_message = String::from("Run failed");
            }
        }

        self.show_output = true;
        Ok(())
    }

    pub fn build_succeeded(&self) -> bool {
        self.last_build_success
    }

    pub fn save_current_file(&mut self) -> Result<()> {
        if let Some(path) = self.editor.current_file().cloned() {
            let content = self.editor.get_content();
            fs::write(&path, content)
                .with_context(|| format!("Failed to save: {}", path.display()))?;
            self.editor.set_modified(false);
            self.status_message = format!("Saved: {}", path.display());
        } else {
            self.status_message = String::from("No file to save");
        }
        Ok(())
    }

    pub fn open_file(&mut self, path: &PathBuf) -> Result<()> {
        self.editor.open_file(path)?;
        self.status_message = format!("Opened: {}", path.display());
        self.focus = FocusedPanel::Editor;
        Ok(())
    }

    pub fn execute_command(&mut self) -> Result<crate::input::CommandResult> {
        use crate::input::CommandResult;

        let cmd = self.command_input.trim().to_string();
        self.command_input.clear();

        // Handle commands with arguments
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let base_cmd = parts.first().map(|s| s.to_lowercase()).unwrap_or_default();

        match base_cmd.as_str() {
            "q" | "quit" => return Ok(CommandResult::Quit),
            "q!" => return Ok(CommandResult::Quit), // Force quit without save check
            "w" | "save" => self.save_current_file()?,
            "wq" => {
                self.save_current_file()?;
                return Ok(CommandResult::Quit);
            }
            "build" | "b" => self.build()?,
            "run" | "r" => self.run()?,
            "br" => {
                self.build()?;
                if self.build_succeeded() {
                    self.run()?;
                }
            }
            "tree" => self.show_file_tree = !self.show_file_tree,
            "output" => self.show_output = !self.show_output,
            "help" => self.show_help = !self.show_help,
            "theme" => {
                if parts.len() > 1 {
                    self.set_theme(parts[1]);
                } else {
                    let themes = crate::theme::Theme::available_themes().join(", ");
                    self.status_message = format!("Available themes: {}", themes);
                }
            }
            "e" | "edit" => {
                if parts.len() > 1 {
                    let path = PathBuf::from(parts[1]);
                    let full_path = if path.is_absolute() {
                        path
                    } else {
                        self.project_dir.join(path)
                    };
                    if let Err(e) = self.editor.open_file(&full_path) {
                        self.status_message = format!("Failed to open: {}", e);
                    } else {
                        self.status_message = format!("Opened: {}", full_path.display());
                    }
                } else {
                    self.status_message = String::from("Usage: :e <filename>");
                }
            }
            "bn" | "bnext" => {
                self.editor.next_buffer();
                self.status_message = format!(
                    "Buffer: {}",
                    self.editor.buffers[self.editor.active_buffer].filename()
                );
            }
            "bp" | "bprev" => {
                self.editor.prev_buffer();
                self.status_message = format!(
                    "Buffer: {}",
                    self.editor.buffers[self.editor.active_buffer].filename()
                );
            }
            "bd" | "bdelete" => {
                if self.editor.modified() {
                    self.status_message =
                        String::from("Buffer has unsaved changes. Use :bd! to force close.");
                } else if self.editor.close_buffer() {
                    self.status_message = String::from("Buffer closed");
                } else {
                    self.status_message = String::from("Cannot close last buffer");
                }
            }
            "bd!" => {
                if self.editor.close_buffer() {
                    self.status_message = String::from("Buffer closed");
                } else {
                    self.status_message = String::from("Cannot close last buffer");
                }
            }
            "autosave" => {
                self.toggle_autosave();
            }
            "refresh" => {
                if let Err(e) = self.file_tree.refresh() {
                    self.status_message = format!("Refresh failed: {}", e);
                } else {
                    self.status_message = String::from("File tree refreshed");
                }
            }
            _ => {
                // Try parsing as line number (e.g., :123)
                if let Ok(line_num) = cmd.parse::<usize>() {
                    self.editor.go_to_line(line_num);
                    self.editor.ensure_cursor_visible(20);
                    self.status_message = format!("Line {}", line_num);
                } else {
                    self.status_message = format!("Unknown command: {}", cmd);
                }
            }
        }

        self.mode = Mode::Normal;
        Ok(CommandResult::Continue)
    }

    // ========== Autocomplete ==========

    pub fn trigger_autocomplete(&mut self) {
        let buf = &self.editor.buffers[self.editor.active_buffer];
        let line = buf.cursor_y;
        let col_byte = buf.cursor_x;

        // Get the prefix (word being typed)
        if col_byte == 0 {
            self.autocomplete.hide();
            return;
        }

        let current_line = &buf.lines[line];
        let chars: Vec<char> = current_line.chars().collect();

        // Convert byte position to character index
        let col_char = current_line[..col_byte.min(current_line.len())]
            .chars()
            .count();

        // Find word start (in character indices)
        let mut start = col_char;
        while start > 0 && start <= chars.len() {
            let c = chars[start - 1];
            if c.is_alphanumeric() || c == '_' || c == '.' || c == '@' {
                start -= 1;
            } else {
                break;
            }
        }

        if start == col_char {
            self.autocomplete.hide();
            return;
        }

        let prefix: String = chars[start..col_char].iter().collect();

        // Get symbols from current buffer
        let buffer_symbols = parse_buffer_symbols(&buf.lines);

        self.autocomplete
            .show(&prefix, line, start, &buffer_symbols);
    }

    pub fn accept_autocomplete(&mut self) {
        if let Some(suggestion) = self.autocomplete.get_selected().cloned() {
            let buf = &self.editor.buffers[self.editor.active_buffer];
            let (_line, trigger_col) = self.autocomplete.trigger_pos;
            let current_col = buf.cursor_x;

            // Delete the prefix that was typed
            let delete_count = current_col.saturating_sub(trigger_col);
            for _ in 0..delete_count {
                self.editor.backspace();
            }

            // Insert the suggestion
            for c in suggestion.text.chars() {
                self.editor.insert_char(c);
            }

            self.autocomplete.hide();
        }
    }

    // ========== Hover Documentation ==========

    pub fn show_hover_docs(&mut self) {
        if let Some(word) = self.editor.get_word_under_cursor() {
            if let Some(doc) = docs::get_documentation(&word) {
                self.hover_doc = Some(doc);
                self.show_hover = true;
            } else {
                self.status_message = format!("No documentation for '{}'", word);
            }
        }
    }

    pub fn hide_hover(&mut self) {
        self.show_hover = false;
        self.hover_doc = None;
    }

    // ========== Diagnostics Navigation ==========

    /// Navigate to the next diagnostic (error/warning)
    pub fn next_diagnostic(&mut self) -> bool {
        if self.diagnostics.is_empty() {
            self.status_message = String::from("No diagnostics");
            return false;
        }

        self.current_diagnostic = (self.current_diagnostic + 1) % self.diagnostics.len();
        self.jump_to_diagnostic(self.current_diagnostic)
    }

    /// Navigate to the previous diagnostic (error/warning)
    pub fn prev_diagnostic(&mut self) -> bool {
        if self.diagnostics.is_empty() {
            self.status_message = String::from("No diagnostics");
            return false;
        }

        self.current_diagnostic = if self.current_diagnostic == 0 {
            self.diagnostics.len() - 1
        } else {
            self.current_diagnostic - 1
        };
        self.jump_to_diagnostic(self.current_diagnostic)
    }

    /// Jump to a specific diagnostic by index
    fn jump_to_diagnostic(&mut self, index: usize) -> bool {
        if index >= self.diagnostics.len() {
            return false;
        }

        let diag = &self.diagnostics[index];
        let file_path = diag.file.clone();
        let line = diag.line;
        let severity = diag.severity;
        let message = diag.message.clone();

        // Open the file if not already open
        if self.editor.current_file() != Some(&file_path) {
            if let Err(e) = self.editor.open_file(&file_path) {
                self.status_message = format!("Cannot open file: {}", e);
                return false;
            }
        }

        // Jump to the error line
        self.editor.go_to_line(line);
        self.editor.ensure_cursor_visible(20);

        // Update status message with diagnostic info
        let severity_str = match severity {
            DiagnosticSeverity::Error => "Error",
            DiagnosticSeverity::Warning => "Warning",
        };
        self.status_message = format!(
            "[{}/{}] {}: {}",
            index + 1,
            self.diagnostics.len(),
            severity_str,
            message
        );

        true
    }

    /// Get the diagnostic for the current cursor line (if any)
    pub fn diagnostic_at_cursor(&self) -> Option<&Diagnostic> {
        let file = self.editor.current_file()?;
        let line = self.editor.cursor_y() + 1; // diagnostics use 1-based line numbers
        diagnostics::diagnostic_for_line(&self.diagnostics, file, line)
    }

    /// Get all diagnostics for the current file
    pub fn diagnostics_for_current_file(&self) -> Vec<&Diagnostic> {
        if let Some(file) = self.editor.current_file() {
            diagnostics::diagnostics_for_file(&self.diagnostics, file)
        } else {
            Vec::new()
        }
    }

    /// Update the editor's visible height (called after terminal resize)
    pub fn update_editor_visible_height(&mut self, height: usize) {
        // Ensure cursor remains visible after resize
        self.editor.ensure_cursor_visible(height);
    }

    /// Scroll output panel up
    pub fn output_scroll_up(&mut self, lines: usize) {
        self.output.scroll_up(lines);
    }

    /// Scroll output panel down
    pub fn output_scroll_down(&mut self, lines: usize) {
        self.output.scroll_down(lines);
    }

    /// Scroll output panel to top
    pub fn output_scroll_to_top(&mut self) {
        self.output.scroll_to_top();
    }

    /// Scroll output panel to bottom
    pub fn output_scroll_to_bottom(&mut self) {
        self.output.scroll_to_bottom();
    }

    /// Page up in output panel
    pub fn output_page_up(&mut self) {
        self.output.page_up();
    }

    /// Page down in output panel
    pub fn output_page_down(&mut self) {
        self.output.page_down();
    }

    /// Toggle output-only fullscreen mode
    pub fn toggle_output_only_mode(&mut self) {
        self.output_only_mode = !self.output_only_mode;
        if self.output_only_mode {
            self.focus = FocusedPanel::Output;
            self.status_message =
                String::from("Output view (F8 or Esc to exit, F9 to save screenshot)");
        } else {
            self.focus = FocusedPanel::Editor;
            self.mode = Mode::Normal;
            self.status_message = String::from("Back to editor");
        }
    }

    /// Export output to a text file for screenshots/labs
    pub fn export_output(&self) -> Result<PathBuf> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let filename = format!("output_{}.txt", timestamp);
        let output_path = self.project_dir.join(&filename);

        let mut content = String::new();

        // Add a header
        content.push_str("═══════════════════════════════════════════════════════════════\n");
        content.push_str("                         PROGRAM OUTPUT\n");
        content.push_str("═══════════════════════════════════════════════════════════════\n\n");

        // Add all output lines
        for line in &self.output.lines {
            match line.output_type {
                crate::ui::output::OutputType::Success => {
                    content.push_str(&format!("✓ {}\n", line.text));
                }
                crate::ui::output::OutputType::Error => {
                    content.push_str(&format!("✗ {}\n", line.text));
                }
                crate::ui::output::OutputType::Stderr => {
                    content.push_str(&format!("⚠ {}\n", line.text));
                }
                crate::ui::output::OutputType::Info => {
                    content.push_str(&format!("→ {}\n", line.text));
                }
                crate::ui::output::OutputType::Stdout => {
                    content.push_str(&format!("  {}\n", line.text));
                }
                crate::ui::output::OutputType::Divider => {
                    content.push('\n');
                }
            }
        }

        content.push_str("\n═══════════════════════════════════════════════════════════════\n");

        fs::write(&output_path, content)
            .with_context(|| format!("Failed to export output to {}", output_path.display()))?;

        Ok(output_path)
    }

    /// Check and perform autosave if needed
    pub fn check_autosave(&mut self) {
        if !self.autosave_enabled {
            return;
        }

        let interval = std::time::Duration::from_secs(self.config.editor.autosave_interval_secs);
        if self.last_save_time.elapsed() >= interval {
            // Check if any buffer is modified
            let has_unsaved = self.editor.buffers.iter().any(|b| b.modified);
            if has_unsaved {
                if let Err(e) = self.save_all() {
                    self.status_message = format!("Autosave failed: {}", e);
                } else {
                    self.status_message = String::from("Autosaved");
                }
            }
            self.last_save_time = std::time::Instant::now();
        }
    }

    /// Save all modified buffers
    pub fn save_all(&mut self) -> anyhow::Result<()> {
        for buffer in &mut self.editor.buffers {
            if buffer.modified {
                if let Some(ref path) = buffer.file_path {
                    std::fs::write(path, buffer.lines.join("\n"))?;
                    buffer.modified = false;
                }
            }
        }
        Ok(())
    }

    /// Toggle autosave
    pub fn toggle_autosave(&mut self) {
        self.autosave_enabled = !self.autosave_enabled;
        self.status_message = format!(
            "Autosave {}",
            if self.autosave_enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }

    /// Copy output to clipboard
    pub fn copy_output_to_clipboard(&mut self) {
        let mut content = String::new();

        for line in &self.output.lines {
            match line.output_type {
                crate::ui::output::OutputType::Divider => {
                    content.push('\n');
                }
                _ => {
                    content.push_str(&line.text);
                    content.push('\n');
                }
            }
        }

        // Use editor's clipboard functionality
        use crate::ui::editor::clipboard::YankType;
        self.editor.clipboard.copy(&content, YankType::Char);
    }
}
