use anyhow::Result;
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::fs;
use std::path::{Path, PathBuf};

use crate::theme::Theme;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

#[derive(Clone)]
pub struct FileTreeState {
    pub entries: Vec<FileEntry>,
    pub list_state: ListState,
    pub root: PathBuf,
}

impl FileTreeState {
    pub fn new(root: &Path) -> Result<Self> {
        let mut state = Self {
            entries: Vec::new(),
            list_state: ListState::default(),
            root: root.to_path_buf(),
        };
        state.refresh()?;
        if !state.entries.is_empty() {
            state.list_state.select(Some(0));
        }
        Ok(state)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.entries.clear();
        self.scan_directory(&self.root.clone(), 0)?;
        Ok(())
    }

    fn scan_directory(&mut self, dir: &PathBuf, depth: usize) -> Result<()> {
        let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();

        entries.sort_by(|a, b| {
            let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in entries {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files and build artifacts
            if name.starts_with('.') || name == "target" {
                continue;
            }

            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

            self.entries.push(FileEntry {
                path: path.clone(),
                name,
                is_dir,
                depth,
                expanded: false,
            });
        }

        Ok(())
    }

    /// Validate that a filename is safe (no path traversal)
    fn validate_filename(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(anyhow::anyhow!("Filename cannot be empty"));
        }
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            return Err(anyhow::anyhow!(
                "Invalid filename: contains path separators or '..'"
            ));
        }
        if name.starts_with('.') && name.len() == 1 {
            return Err(anyhow::anyhow!("Invalid filename"));
        }
        // Check for other dangerous characters
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\0'];
        if name.chars().any(|c| invalid_chars.contains(&c)) {
            return Err(anyhow::anyhow!("Filename contains invalid characters"));
        }
        Ok(())
    }

    /// Verify a path is within the project root
    fn verify_path_in_project(&self, path: &Path) -> Result<()> {
        // Canonicalize both paths to resolve symlinks and '..'
        let canonical_root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());

        // For new files, check the parent directory
        let check_path = if path.exists() {
            path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
        } else if let Some(parent) = path.parent() {
            if parent.exists() {
                parent
                    .canonicalize()
                    .unwrap_or_else(|_| parent.to_path_buf())
                    .join(path.file_name().unwrap_or_default())
            } else {
                path.to_path_buf()
            }
        } else {
            path.to_path_buf()
        };

        if !check_path.starts_with(&canonical_root) {
            return Err(anyhow::anyhow!("Path escapes project directory"));
        }
        Ok(())
    }

    pub fn create_file(&mut self, name: &str) -> Result<()> {
        Self::validate_filename(name)?;

        let base_path = self.get_base_path_for_creation();
        let new_path = base_path.join(name);

        self.verify_path_in_project(&new_path)?;

        if new_path.exists() {
            return Err(anyhow::anyhow!("File already exists"));
        }

        fs::File::create(&new_path)?;
        self.refresh()?;
        Ok(())
    }

    pub fn create_dir(&mut self, name: &str) -> Result<()> {
        Self::validate_filename(name)?;

        let base_path = self.get_base_path_for_creation();
        let new_path = base_path.join(name);

        self.verify_path_in_project(&new_path)?;

        if new_path.exists() {
            return Err(anyhow::anyhow!("Directory already exists"));
        }

        fs::create_dir(&new_path)?;
        self.refresh()?;
        Ok(())
    }

    pub fn delete_current(&mut self) -> Result<()> {
        if let Some(path) = self.selected_path() {
            self.verify_path_in_project(&path)?;

            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
            self.refresh()?;
        }
        Ok(())
    }

    pub fn rename_current(&mut self, new_name: &str) -> Result<()> {
        Self::validate_filename(new_name)?;

        if let Some(path) = self.selected_path() {
            self.verify_path_in_project(&path)?;

            let parent = path.parent().unwrap_or(&self.root);
            let new_path = parent.join(new_name);

            self.verify_path_in_project(&new_path)?;

            if new_path.exists() {
                return Err(anyhow::anyhow!("Target already exists"));
            }

            fs::rename(&path, &new_path)?;
            self.refresh()?;
        }
        Ok(())
    }

    fn get_base_path_for_creation(&self) -> PathBuf {
        if let Some(entry) = self.selected_entry() {
            if entry.is_dir {
                entry.path.clone()
            } else {
                entry
                    .path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or(self.root.clone())
            }
        } else {
            self.root.clone()
        }
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.list_state.selected().and_then(|i| self.entries.get(i))
    }

    pub fn selected_path(&self) -> Option<PathBuf> {
        self.selected_entry().map(|e| e.path.clone())
    }

    pub fn move_up(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected > 0 {
                self.list_state.select(Some(selected - 1));
            }
        }
    }

    pub fn move_down(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected + 1 < self.entries.len() {
                self.list_state.select(Some(selected + 1));
            }
        } else if !self.entries.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn toggle_expand(&mut self) -> Result<Option<PathBuf>> {
        if let Some(idx) = self.list_state.selected() {
            let entry = &self.entries[idx];

            if entry.is_dir {
                let path = entry.path.clone();
                let depth = entry.depth;
                let was_expanded = entry.expanded;

                if was_expanded {
                    // Collapse: remove children
                    self.entries[idx].expanded = false;
                    let mut remove_count = 0;
                    for i in (idx + 1)..self.entries.len() {
                        if self.entries[i].depth > depth {
                            remove_count += 1;
                        } else {
                            break;
                        }
                    }
                    self.entries.drain((idx + 1)..(idx + 1 + remove_count));
                } else {
                    // Expand: insert children
                    self.entries[idx].expanded = true;
                    let mut children = Vec::new();
                    self.collect_children(&path, depth + 1, &mut children)?;
                    for (i, child) in children.into_iter().enumerate() {
                        self.entries.insert(idx + 1 + i, child);
                    }
                }
                Ok(None)
            } else {
                // It's a file - return it for opening (skip binary files)
                let name = &entry.name;
                if name.ends_with(".exe")
                    || name.ends_with(".obj")
                    || name.ends_with(".lib")
                    || name.ends_with(".o")
                {
                    Ok(None)
                } else {
                    Ok(Some(entry.path.clone()))
                }
            }
        } else {
            Ok(None)
        }
    }

    fn collect_children(
        &self,
        dir: &PathBuf,
        depth: usize,
        out: &mut Vec<FileEntry>,
    ) -> Result<()> {
        let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();

        entries.sort_by(|a, b| {
            let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in entries {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') || name == "target" {
                continue;
            }

            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

            out.push(FileEntry {
                path,
                name,
                is_dir,
                depth,
                expanded: false,
            });
        }

        Ok(())
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut FileTreeState,
    focused: bool,
    theme: &Theme,
) {
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
        .title(Span::styled(" Files ", title_style))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(border_style)
        .style(Style::default().bg(theme.ui.background.to_color()));

    let items: Vec<ListItem> = state
        .entries
        .iter()
        .map(|entry| {
            let indent = "  ".repeat(entry.depth);
            let icon = " ";
            let style = if entry.is_dir {
                Style::default().fg(theme.ui.file_tree_dir.to_color())
            } else if entry.name.ends_with(".asm") {
                Style::default().fg(theme.ui.file_tree_asm.to_color())
            } else if entry.name.ends_with(".exe") {
                Style::default().fg(theme.ui.file_tree_exe.to_color())
            } else {
                Style::default().fg(theme.ui.file_tree_file.to_color())
            };
            ListItem::new(format!("{}{}{}", indent, icon, entry.name)).style(style)
        })
        .collect();

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(theme.ui.file_tree_selected.to_color())
            .add_modifier(Modifier::BOLD),
    );

    frame.render_stateful_widget(list, area, &mut state.list_state);
}
