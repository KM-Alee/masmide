use anyhow::{bail, Result};
use std::path::PathBuf;
use std::process::Command;

pub struct Toolchain {
    pub jwasm: PathBuf,
    pub linker: PathBuf,
    pub wine: PathBuf,
}

impl Toolchain {
    pub fn detect(
        jwasm_path: &PathBuf,
        linker_path: &PathBuf,
        wine_path: &PathBuf,
    ) -> Result<Self> {
        let jwasm = Self::find_executable(jwasm_path, "jwasm")?;
        let linker = Self::find_executable(linker_path, "x86_64-w64-mingw32-ld")?;
        let wine = Self::find_executable(wine_path, "wine")?;

        Ok(Self {
            jwasm,
            linker,
            wine,
        })
    }

    fn find_executable(configured: &PathBuf, name: &str) -> Result<PathBuf> {
        // First try the configured path
        if configured.exists() {
            return Ok(configured.clone());
        }

        // If it's just a name, try to find it in PATH
        let configured_str = configured.to_string_lossy();
        if !configured_str.contains('/') {
            if let Ok(output) = Command::new("which")
                .arg(&configured_str.to_string())
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Ok(PathBuf::from(path));
                    }
                }
            }
        }

        // Try common names
        if let Ok(output) = Command::new("which").arg(name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(PathBuf::from(path));
                }
            }
        }

        bail!(
            "Could not find '{}'. Please install it or configure the path in config.",
            name
        )
    }

    pub fn verify(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check jwasm
        match Command::new(&self.jwasm).arg("-?").output() {
            Ok(output) => {
                if !output.status.success() && output.stderr.is_empty() {
                    warnings.push(format!(
                        "jwasm at {} may not be working correctly",
                        self.jwasm.display()
                    ));
                }
            }
            Err(e) => {
                warnings.push(format!("Cannot execute jwasm: {}", e));
            }
        }

        // Check linker
        match Command::new(&self.linker).arg("--version").output() {
            Ok(_) => {}
            Err(e) => {
                warnings.push(format!("Cannot execute linker: {}", e));
            }
        }

        // Check wine
        match Command::new(&self.wine).arg("--version").output() {
            Ok(_) => {}
            Err(e) => {
                warnings.push(format!("Cannot execute wine: {}", e));
            }
        }

        Ok(warnings)
    }
}
