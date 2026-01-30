use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: usize,
    pub column: Option<usize>,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub code: Option<String>, // e.g., "A2008" for JWasm error codes
}

impl Diagnostic {
    pub fn error(file: PathBuf, line: usize, message: String) -> Self {
        Self {
            file,
            line,
            column: None,
            severity: DiagnosticSeverity::Error,
            message,
            code: None,
        }
    }

    pub fn warning(file: PathBuf, line: usize, message: String) -> Self {
        Self {
            file,
            line,
            column: None,
            severity: DiagnosticSeverity::Warning,
            message,
            code: None,
        }
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }
}

/// Parse JWasm assembler output for errors and warnings.
///
/// JWasm error format examples:
/// - `main.asm(15): error A2008: syntax error : mov`
/// - `main.asm(10): warning A4031: constant too large`
/// - `main.asm(15) : error A2008: syntax error : mov` (with space before colon)
/// - `/path/to/file.asm(20): error A2006: undefined symbol : myLabel`
/// - `Fatal error A1106: Cannot open file: "test/main.asm"` (file-level error)
/// - `test/main.asm(15) : error A2008: syntax error` (stdout format)
pub fn parse_jwasm_output(output: &str, project_dir: &Path) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for line in output.lines() {
        // Try line-specific error format first
        if let Some(diag) = parse_jwasm_line(line, project_dir) {
            diagnostics.push(diag);
        }
        // Try file-level/fatal error format
        else if let Some(diag) = parse_jwasm_fatal_error(line, project_dir) {
            diagnostics.push(diag);
        }
    }

    diagnostics
}

/// Parse fatal/file-level errors without line numbers
/// Format: `Fatal error A1106: message` or `error A1000: message`
fn parse_jwasm_fatal_error(line: &str, project_dir: &Path) -> Option<Diagnostic> {
    let line = line.trim();
    let lower = line.to_lowercase();

    // Check for "Fatal error" or standalone "error" at the start
    let (_severity, rest) = if lower.starts_with("fatal error") {
        (DiagnosticSeverity::Error, &line[11..].trim_start())
    } else if lower.starts_with("error") && !line.contains('(') {
        (DiagnosticSeverity::Error, &line[5..].trim_start())
    } else if lower.starts_with("warning") && !line.contains('(') {
        (DiagnosticSeverity::Warning, &line[7..].trim_start())
    } else {
        return None;
    };

    // Parse error code (Axxxx:) and message
    let (code, message) = if rest.starts_with('A') || rest.starts_with('a') {
        if let Some(colon_pos) = rest.find(':') {
            let code = rest[..colon_pos].trim().to_string();
            let msg = rest[colon_pos + 1..].trim().to_string();
            (Some(code), msg)
        } else {
            (None, rest.to_string())
        }
    } else {
        (None, rest.to_string())
    };

    // Try to extract filename from message if present (e.g., "Cannot open file: \"test.asm\"")
    let file_path = if let Some(start) = message.find('"') {
        if let Some(end) = message[start + 1..].find('"') {
            let filename = &message[start + 1..start + 1 + end];
            if PathBuf::from(filename).is_absolute() {
                PathBuf::from(filename)
            } else {
                project_dir.join(filename)
            }
        } else {
            project_dir.join("unknown")
        }
    } else {
        project_dir.join("unknown")
    };

    let mut diag = Diagnostic::error(file_path, 0, message); // line 0 for file-level errors
    if let Some(c) = code {
        diag = diag.with_code(c);
    }

    Some(diag)
}

fn parse_jwasm_line(line: &str, project_dir: &Path) -> Option<Diagnostic> {
    // Pattern: filename(line): error/warning Axxxx: message
    // Or: filename(line) : error/warning Axxxx: message

    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Find the opening parenthesis for line number
    let paren_open = line.find('(')?;
    let paren_close = line[paren_open..].find(')')? + paren_open;

    // Extract filename
    let filename = line[..paren_open].trim();
    if filename.is_empty() {
        return None;
    }

    // Extract line number
    let line_num_str = &line[paren_open + 1..paren_close];
    let line_num: usize = line_num_str.parse().ok()?;

    // Find the colon after the closing paren (may have space before it)
    let after_paren = &line[paren_close + 1..];
    let colon_pos = after_paren.find(':')?;
    let after_colon = after_paren[colon_pos + 1..].trim();

    // Determine severity and parse the rest
    let lower_after = after_colon.to_lowercase();
    let (severity, rest): (DiagnosticSeverity, &str) = if lower_after.starts_with("error") {
        (DiagnosticSeverity::Error, after_colon[5..].trim_start())
    } else if lower_after.starts_with("warning") {
        (DiagnosticSeverity::Warning, after_colon[7..].trim_start())
    } else {
        // Try without the extra colon - maybe format is "error Axxxx:"
        if lower_after.starts_with("error ") {
            (DiagnosticSeverity::Error, &after_colon[6..])
        } else if lower_after.starts_with("warning ") {
            (DiagnosticSeverity::Warning, &after_colon[8..])
        } else {
            return None;
        }
    };

    // Parse error code and message
    // Format after severity: "A2008: syntax error : mov" or just "message"
    let (code, message): (Option<String>, String) =
        if rest.starts_with('A') || rest.starts_with('a') {
            // Look for error code pattern Axxxx:
            if let Some(code_end) = rest.find(':') {
                let code = rest[..code_end].trim().to_string();
                let msg = rest[code_end + 1..].trim().to_string();
                (Some(code), msg)
            } else {
                (None, rest.to_string())
            }
        } else {
            (None, rest.to_string())
        };

    // Resolve file path
    let file_path = if PathBuf::from(filename).is_absolute() {
        PathBuf::from(filename)
    } else {
        project_dir.join(filename)
    };

    let mut diag = match severity {
        DiagnosticSeverity::Error => Diagnostic::error(file_path, line_num, message),
        DiagnosticSeverity::Warning => Diagnostic::warning(file_path, line_num, message),
    };

    if let Some(c) = code {
        diag = diag.with_code(c);
    }

    Some(diag)
}

/// Get diagnostics for a specific file
pub fn diagnostics_for_file<'a>(diagnostics: &'a [Diagnostic], file: &Path) -> Vec<&'a Diagnostic> {
    diagnostics.iter().filter(|d| d.file == file).collect()
}

/// Get diagnostic for a specific line in a file (if any)
pub fn diagnostic_for_line<'a>(
    diagnostics: &'a [Diagnostic],
    file: &Path,
    line: usize,
) -> Option<&'a Diagnostic> {
    diagnostics
        .iter()
        .find(|d| d.file == file && d.line == line)
}

/// Count errors and warnings
pub fn count_by_severity(diagnostics: &[Diagnostic]) -> (usize, usize) {
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Warning)
        .count();
    (errors, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_jwasm_error() {
        let output = "main.asm(15): error A2008: syntax error : mov";
        let project_dir = PathBuf::from("/project");
        let diagnostics = parse_jwasm_output(output, &project_dir);

        assert_eq!(diagnostics.len(), 1);
        let d = &diagnostics[0];
        assert_eq!(d.line, 15);
        assert_eq!(d.severity, DiagnosticSeverity::Error);
        assert_eq!(d.code.as_deref(), Some("A2008"));
        assert!(d.message.contains("syntax error"));
    }

    #[test]
    fn test_parse_jwasm_warning() {
        let output = "test.asm(10): warning A4031: constant too large";
        let project_dir = PathBuf::from("/project");
        let diagnostics = parse_jwasm_output(output, &project_dir);

        assert_eq!(diagnostics.len(), 1);
        let d = &diagnostics[0];
        assert_eq!(d.line, 10);
        assert_eq!(d.severity, DiagnosticSeverity::Warning);
    }

    #[test]
    fn test_parse_multiple_errors() {
        let output = r#"main.asm(5): error A2006: undefined symbol : myLabel
main.asm(10): error A2008: syntax error
main.asm(15): warning A4031: constant too large"#;
        let project_dir = PathBuf::from("/project");
        let diagnostics = parse_jwasm_output(output, &project_dir);

        assert_eq!(diagnostics.len(), 3);

        let (errors, warnings) = count_by_severity(&diagnostics);
        assert_eq!(errors, 2);
        assert_eq!(warnings, 1);
    }

    #[test]
    fn test_parse_fatal_error() {
        let output = r#"Fatal error A1106: Cannot open file: "test/main.asm" [ENOENT]"#;
        let project_dir = PathBuf::from("/project");
        let diagnostics = parse_jwasm_output(output, &project_dir);

        assert_eq!(diagnostics.len(), 1);
        let d = &diagnostics[0];
        assert_eq!(d.severity, DiagnosticSeverity::Error);
        assert_eq!(d.code.as_deref(), Some("A1106"));
        assert!(d.message.contains("Cannot open file"));
        assert_eq!(d.line, 0); // File-level error has no line
    }

    #[test]
    fn test_parse_stdout_error_format() {
        // JWasm sometimes writes errors to stdout with space before colon
        let output = "test/main.asm(15) : error A2008: syntax error : mov";
        let project_dir = PathBuf::from("/project");
        let diagnostics = parse_jwasm_output(output, &project_dir);

        assert_eq!(diagnostics.len(), 1);
        let d = &diagnostics[0];
        assert_eq!(d.line, 15);
        assert_eq!(d.severity, DiagnosticSeverity::Error);
    }
}
