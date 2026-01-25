use crate::config::ProjectConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const HELLO_TEMPLATE: &str = r#"; =============================================================================
; Program: Hello World
; Description: A simple MASM program using Irvine32 library
; =============================================================================

INCLUDE Irvine32.inc

.data
    message BYTE "Hello, World!", 0

.code
main PROC
    ; Display the message
    mov  edx, OFFSET message
    call WriteString
    call Crlf

    ; Exit program
    exit
main ENDP

END main
"#;

pub fn create_new_project(name: &str) -> Result<()> {
    let project_dir = PathBuf::from(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(&project_dir)
        .with_context(|| format!("Failed to create directory: {}", name))?;

    // Create main.asm
    let main_asm = project_dir.join("main.asm");
    fs::write(&main_asm, HELLO_TEMPLATE).context("Failed to create main.asm")?;

    // Create project config
    let config = ProjectConfig {
        name: name.to_string(),
        entry_file: PathBuf::from("main.asm"),
        output_name: format!("{}.exe", name),
        include_paths: vec![],
        lib_paths: vec![],
        libs: vec![
            String::from("irvine32"),
            String::from("kernel32"),
            String::from("user32"),
        ],
    };
    config.save(&project_dir)?;

    // Create a basic README
    let readme = project_dir.join("README.md");
    let readme_content = format!(
        r#"# {}

A MASM assembly project using Irvine32.

## Build & Run

Open with masmide:
```bash
cd {}
masmide
```

Or build manually:
```bash
jwasm -coff -Fo main.obj main.asm
x86_64-w64-mingw32-ld -o {}.exe main.obj -L/path/to/irvine -lirvine32 -lkernel32
wine {}.exe
```

## Keybindings

- `F5` - Build and run
- `F6` - Build only
- `F7` - Run only
- `i` - Enter insert mode
- `Esc` - Return to normal mode
- `:w` or `:save` - Save file
- `:q` or `:quit` - Quit
- `:wq` - Save and quit
- `Ctrl+S` - Save
- `Ctrl+Q` - Quit
"#,
        name, name, name, name
    );
    fs::write(&readme, readme_content).context("Failed to create README.md")?;

    Ok(())
}
