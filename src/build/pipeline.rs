use crate::config::{Config, ProjectConfig};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct BuildOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub struct RunOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub struct Pipeline {
    jwasm_path: PathBuf,
    linker_path: PathBuf,
    wine_path: PathBuf,
    irvine_lib_path: PathBuf,
    irvine_inc_path: PathBuf,
    project_dir: PathBuf,
    output_name: String,
    libs: Vec<String>,
    last_exe: Option<PathBuf>,
}

impl Pipeline {
    pub fn new(config: &Config, project_config: &ProjectConfig, project_dir: &Path) -> Self {
        Self {
            jwasm_path: config.toolchain.jwasm_path.clone(),
            linker_path: config.toolchain.linker_path.clone(),
            wine_path: config.toolchain.wine_path.clone(),
            irvine_lib_path: config.toolchain.irvine_lib_path.clone(),
            irvine_inc_path: config.toolchain.irvine_inc_path.clone(),
            project_dir: project_dir.to_path_buf(),
            output_name: project_config.output_name.clone(),
            libs: project_config.libs.clone(),
            last_exe: None,
        }
    }

    pub fn build(&mut self, source_file: &PathBuf) -> Result<BuildOutput> {
        let mut stderr_log = String::new();

        // Canonicalize the source file path to get absolute path
        let source_file = if source_file.is_absolute() {
            source_file.clone()
        } else {
            self.project_dir.join(source_file)
        };

        // Verify the source file exists
        if !source_file.exists() {
            return Ok(BuildOutput {
                success: false,
                stdout: String::new(),
                stderr: format!("File not found: {}", source_file.display()),
            });
        }

        // Determine output paths
        let file_stem = source_file
            .file_stem()
            .context("Invalid source file name")?
            .to_string_lossy();

        let obj_file = self.project_dir.join(format!("{}.obj", file_stem));
        let exe_file = self.project_dir.join(&self.output_name);

        let source_name = source_file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Step 1: Assemble with JWasm
        let jwasm_result = Command::new(&self.jwasm_path)
            .arg("-coff")
            .arg(format!("-Fo{}", obj_file.display()))
            .arg(format!("-I{}", self.irvine_inc_path.display()))
            .arg(&source_file)
            .current_dir(&self.project_dir)
            .output()
            .context("Failed to execute jwasm")?;

        // Extract only error/warning lines from jwasm output
        let jwasm_stdout = String::from_utf8_lossy(&jwasm_result.stdout);
        let jwasm_stderr = String::from_utf8_lossy(&jwasm_result.stderr);

        for line in jwasm_stdout.lines().chain(jwasm_stderr.lines()) {
            let lower = line.to_lowercase();
            // Keep only actual error/warning messages, skip banner and info
            if (lower.contains("error") || lower.contains("warning"))
                && !line.contains("JWasm")
                && !line.contains("Copyright")
            {
                stderr_log.push_str(line);
                stderr_log.push('\n');
            }
        }

        if !jwasm_result.status.success() {
            return Ok(BuildOutput {
                success: false,
                stdout: String::new(),
                stderr: stderr_log.trim().to_string(),
            });
        }

        // Step 2: Link with MinGW-w64 ld
        let mut link_cmd = Command::new(&self.linker_path);

        link_cmd
            .arg("-o")
            .arg(&exe_file)
            .arg(&obj_file)
            .arg("--subsystem")
            .arg("console");

        // Add .lib files directly from irvine lib path
        let lib_files = ["Irvine32.lib", "Kernel32.Lib", "User32.Lib"];
        for lib_file in &lib_files {
            let lib_path = self.irvine_lib_path.join(lib_file);
            if lib_path.exists() {
                link_cmd.arg(&lib_path);
            }
        }

        link_cmd.current_dir(&self.project_dir);

        let link_result = link_cmd.output().context("Failed to execute linker")?;

        // Extract linker errors
        let link_stderr = String::from_utf8_lossy(&link_result.stderr);
        for line in link_stderr.lines() {
            let lower = line.to_lowercase();
            if lower.contains("error") || lower.contains("undefined") {
                stderr_log.push_str(line);
                stderr_log.push('\n');
            }
        }

        if !link_result.status.success() {
            return Ok(BuildOutput {
                success: false,
                stdout: String::new(),
                stderr: stderr_log.trim().to_string(),
            });
        }

        self.last_exe = Some(exe_file.clone());

        // Clean up object file
        let _ = std::fs::remove_file(&obj_file);

        Ok(BuildOutput {
            success: true,
            stdout: format!("Built {} → {}", source_name, self.output_name),
            stderr: stderr_log.trim().to_string(),
        })
    }

    pub fn run(&self) -> Result<RunOutput> {
        let exe_path = self
            .last_exe
            .as_ref()
            .context("No executable to run. Build first.")?;

        if !exe_path.exists() {
            anyhow::bail!("Executable not found: {}", exe_path.display());
        }

        // Use temp file to capture PTY output from script command
        let tmp_file = self.project_dir.join(".masmide_output.tmp");

        // Validate paths to prevent command injection
        let wine_path_str = self.wine_path.to_string_lossy();
        let exe_path_str = exe_path.to_string_lossy();

        // Reject paths with shell metacharacters
        let dangerous_chars = [
            '\'', '"', '`', '$', '\\', ';', '&', '|', '>', '<', '(', ')', '{', '}', '\n',
        ];
        if wine_path_str.chars().any(|c| dangerous_chars.contains(&c)) {
            anyhow::bail!("Wine path contains invalid characters");
        }
        if exe_path_str.chars().any(|c| dangerous_chars.contains(&c)) {
            anyhow::bail!("Executable path contains invalid characters");
        }

        // Use 'script' command to run wine in a PTY for proper console I/O
        // Quote the paths to handle spaces safely
        let result = Command::new("script")
            .arg("-q") // quiet
            .arg("-c") // command
            .arg(format!("'{}' '{}'", wine_path_str, exe_path_str))
            .arg(&tmp_file)
            .current_dir(&self.project_dir)
            .output()
            .context("Failed to execute wine via script")?;

        // Read output from temp file
        let raw_output = std::fs::read_to_string(&tmp_file).unwrap_or_default();
        let _ = std::fs::remove_file(&tmp_file);

        // Clean up script header/footer and control characters
        let stdout: String = raw_output
            .lines()
            .filter(|line| !line.starts_with("Script started") && !line.starts_with("Script done"))
            .collect::<Vec<_>>()
            .join("\n")
            .replace("\r", "")
            .replace("\x1b[?25l", "")
            .replace("\x1b[?25h", "")
            .chars()
            .filter(|c| !c.is_control() || *c == '\n')
            .collect();

        Ok(RunOutput {
            exit_code: result.status.code().unwrap_or(-1),
            stdout: stdout.trim().to_string(),
            stderr: String::from_utf8_lossy(&result.stderr).to_string(),
        })
    }
}
