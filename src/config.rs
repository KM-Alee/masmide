use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::theme::Theme;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub toolchain: ToolchainConfig,
    pub editor: EditorConfig,
    pub layout: LayoutConfig,
    pub theme_name: String,
    #[serde(skip)]
    pub theme: Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolchainConfig {
    pub jwasm_path: PathBuf,
    pub linker_path: PathBuf,
    pub wine_path: PathBuf,
    pub irvine_lib_path: PathBuf,
    pub irvine_inc_path: PathBuf,
}

impl Default for ToolchainConfig {
    fn default() -> Self {
        Self {
            jwasm_path: PathBuf::from("jwasm"),
            linker_path: PathBuf::from("i686-w64-mingw32-ld"),
            wine_path: PathBuf::from("wine"),
            irvine_lib_path: PathBuf::from("/usr/local/lib/irvine"),
            irvine_inc_path: PathBuf::from("/usr/local/include/irvine"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub auto_indent: bool,
    pub show_line_numbers: bool,
    pub autosave: bool,
    pub autosave_interval_secs: u64,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            insert_spaces: true,
            auto_indent: true,
            show_line_numbers: true,
            autosave: true,
            autosave_interval_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    pub file_tree_width: u16,
    pub output_height: u16,
    pub file_tree_min_width: u16,
    pub file_tree_max_width: u16,
    pub output_min_height: u16,
    pub output_max_height: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            toolchain: ToolchainConfig {
                jwasm_path: PathBuf::from("jwasm"),
                linker_path: PathBuf::from("i686-w64-mingw32-ld"),
                wine_path: PathBuf::from("wine"),
                irvine_lib_path: PathBuf::from("/usr/local/lib/irvine"),
                irvine_inc_path: PathBuf::from("/usr/local/include/irvine"),
            },
            editor: EditorConfig {
                tab_size: 4,
                insert_spaces: true,
                auto_indent: true,
                show_line_numbers: true,
                autosave: true,
                autosave_interval_secs: 30,
            },
            layout: LayoutConfig::default(),
            theme_name: String::from("gruvbox"),
            theme: Theme::gruvbox(),
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            file_tree_width: 22,
            output_height: 16,
            file_tree_min_width: 15,
            file_tree_max_width: 50,
            output_min_height: 5,
            output_max_height: 40,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config: {}", config_path.display()))?;
            let mut config: Config =
                toml::from_str(&content).with_context(|| "Failed to parse config file")?;
            // Initialize theme from theme_name
            config.theme = Theme::from_name(&config.theme_name);
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn set_theme(&mut self, name: &str) {
        self.theme_name = name.to_string();
        self.theme = Theme::from_name(name);
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_file_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "masmide", "masmide")
            .context("Could not determine config directory")?;
        Ok(proj_dirs.config_dir().join("config.toml"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub entry_file: PathBuf,
    pub output_name: String,
    pub include_paths: Vec<PathBuf>,
    pub lib_paths: Vec<PathBuf>,
    pub libs: Vec<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: String::from("untitled"),
            entry_file: PathBuf::from("main.asm"),
            output_name: String::from("main.exe"),
            include_paths: vec![],
            lib_paths: vec![],
            libs: vec![
                String::from("irvine32"),
                String::from("kernel32"),
                String::from("user32"),
            ],
        }
    }
}

impl ProjectConfig {
    pub fn load(project_dir: &PathBuf) -> Result<Self> {
        let config_path = project_dir.join(".masmide.toml");

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: ProjectConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(ProjectConfig::default())
        }
    }

    pub fn save(&self, project_dir: &PathBuf) -> Result<()> {
        let config_path = project_dir.join(".masmide.toml");
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}
