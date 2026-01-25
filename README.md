# masmide

[![Crates.io](https://img.shields.io/crates/v/masmide.svg)](https://crates.io/crates/masmide)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/KM-Alee/masmide/actions/workflows/ci.yml/badge.svg)](https://github.com/KM-Alee/masmide/actions/workflows/ci.yml)

A Terminal User Interface (TUI) IDE for MASM (Microsoft Macro Assembler) development on Linux. Write, build, and run x86 assembly programs using JWasm, MinGW-w64, and Wine.

![masmide screenshot](https://raw.githubusercontent.com/KM-Alee/masmide/main/assets/screenshot.png)

## Features

- 📝 **Syntax Highlighting** - Full MASM syntax support with customizable themes
- 🔧 **Build Integration** - One-key build with JWasm and MinGW-w64 linker
- ▶️ **Run with Wine** - Execute Windows PE binaries directly
- 📁 **File Tree** - Navigate projects with keyboard-driven file browser
- 🔍 **Search** - Find and replace across files
- 💡 **Autocomplete** - Context-aware completion for MASM instructions and Irvine32 procedures
- 📚 **Inline Documentation** - Hover docs for instructions and library procedures
- 🎨 **Themes** - Dark, light, and Dracula themes included
- ⌨️ **Vim-like Navigation** - Optional vim keybindings for editor

## Installation

### Quick Install (Recommended)

The easiest way to install masmide with all dependencies:

```bash
curl -sSL https://raw.githubusercontent.com/KM-Alee/masmide/main/scripts/install-remote.sh | bash
```

This will automatically:
- Download the latest prebuilt binary
- Install dependencies (mingw-w64, wine, jwasm)
- Set up the Irvine32 library
- Create default configuration

### Download Prebuilt Binary

Download from [GitHub Releases](https://github.com/KM-Alee/masmide/releases/latest):

| Architecture | Description |
|-------------|-------------|
| `x86_64` | Standard Linux (Intel/AMD 64-bit) |
| `aarch64` | ARM 64-bit (Raspberry Pi 4, etc.) |
| `x86_64-musl` | Static binary (Alpine Linux, etc.) |

```bash
# Download and extract
tar -xzf masmide-v*-linux-x86_64.tar.gz
cd masmide-v*-linux-x86_64

# Run the included installer
sudo ./install.sh
```

### From crates.io

If you have Rust installed:

```bash
cargo install masmide
```

### From Source

```bash
git clone https://github.com/KM-Alee/masmide.git
cd masmide
./install.sh  # Installs deps + builds + installs
```

### Manual Prerequisites

If you prefer to install dependencies manually:

**Arch Linux:**
```bash
sudo pacman -S mingw-w64-gcc wine
yay -S jwasm  # or paru -S jwasm
```

**Ubuntu/Debian:**
```bash
sudo apt install mingw-w64 wine
# JWasm: build from https://github.com/JWasm/JWasm
```

### Uninstall

```bash
# If installed via install script
./uninstall.sh

# If installed via cargo
cargo uninstall masmide
```

### Irvine32 Library Setup

This repository includes the Irvine32 library files in the `Irvine/` directory. The install script automatically sets these up, or you can manually copy them:

```bash
# Create library directories
sudo mkdir -p /usr/local/lib/irvine /usr/local/include/irvine

# Copy library files
sudo cp Irvine/*.lib /usr/local/lib/irvine/
sudo cp Irvine/*.inc /usr/local/include/irvine/
```

## Quick Start

**Create a new project:**
```bash
masmide --new myproject
cd myproject
masmide
```

**Open an existing directory:**
```bash
cd your-asm-project
masmide
```

**Open a specific file:**
```bash
masmide path/to/file.asm
```

## Keybindings

### Global
| Key | Action |
|-----|--------|
| `Ctrl+Q` | Quit |
| `Ctrl+S` | Save file |
| `Ctrl+B` | Build project |
| `Ctrl+R` | Run executable |
| `Ctrl+Shift+B` | Build and run |
| `Ctrl+P` | Command palette |
| `Ctrl+F` | Find in file |
| `Ctrl+Shift+F` | Find in project |
| `Ctrl+G` | Go to line |
| `Tab` | Switch focus (editor/file tree/output) |
| `F1` | Help |

### Editor
| Key | Action |
|-----|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+C` | Copy |
| `Ctrl+X` | Cut |
| `Ctrl+V` | Paste |
| `Ctrl+A` | Select all |
| `Ctrl+D` | Duplicate line |
| `Ctrl+/` | Toggle comment |
| `Home/End` | Line start/end |
| `Ctrl+Home/End` | File start/end |

### File Tree
| Key | Action |
|-----|--------|
| `Enter` | Open file/toggle folder |
| `n` | New file |
| `N` | New folder |
| `d` | Delete |
| `r` | Rename |

## Configuration

masmide looks for a `.masmide.toml` file in your project root. Create one to customize build settings:

```toml
[build]
# Path to JWasm executable (default: "jwasm")
assembler = "jwasm"

# Assembler flags
asm_flags = ["-coff"]

# Path to MinGW linker (default: "x86_64-w64-mingw32-ld")
linker = "x86_64-w64-mingw32-ld"

# Linker flags
link_flags = ["-mi386pe", "--subsystem", "console"]

# Library search paths (Irvine32 libs)
lib_paths = ["/usr/local/lib/irvine", "./Irvine"]

# Libraries to link
libs = ["Irvine32", "Kernel32", "User32"]

# Include paths for assembler
include_paths = ["/usr/local/include/irvine", "./Irvine"]

[editor]
# Tab width (default: 4)
tab_width = 4

# Use spaces instead of tabs
use_spaces = true

# Show line numbers
line_numbers = true

# Theme: "dark", "light", "dracula"
theme = "dark"

[run]
# Command to run the executable (default: "wine")
runner = "wine"
```

## Project Structure

A typical masmide project:

```
myproject/
├── .masmide.toml    # Project configuration
├── main.asm         # Main assembly file
├── Irvine/          # (optional) Local Irvine library copy
│   ├── Irvine32.inc
│   ├── Irvine32.lib
│   └── ...
└── README.md
```

## Example Program

```asm
; Hello World - MASM with Irvine32
INCLUDE Irvine32.inc

.data
    message BYTE "Hello, World!", 0

.code
main PROC
    mov  edx, OFFSET message
    call WriteString
    call Crlf
    exit
main ENDP
END main
```

## Troubleshooting

### "jwasm: command not found"
Install JWasm from your package manager or build from source:
```bash
git clone https://github.com/JWasm/JWasm.git
cd JWasm
make -f GccUnix.mak
sudo cp jwasm /usr/local/bin/
```

### "cannot find -lIrvine32"
Ensure the Irvine library is installed:
```bash
sudo cp Irvine/*.lib /usr/local/lib/irvine/
```
And add the path to your `.masmide.toml`:
```toml
[build]
lib_paths = ["/usr/local/lib/irvine"]
```

### Wine errors
Ensure Wine is properly configured:
```bash
winecfg  # Configure Wine
wine --version  # Verify installation
```

### Build fails with linker errors
Check that MinGW-w64 is installed for 32-bit targets:
```bash
# Arch Linux
sudo pacman -S mingw-w64-gcc

# Ubuntu/Debian  
sudo apt install gcc-mingw-w64-i686
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Irvine32 Library](http://asmirvine.com/) by Kip Irvine
- [JWasm](https://github.com/JWasm/JWasm) - Open source MASM-compatible assembler
- [Ratatui](https://github.com/ratatui-org/ratatui) - TUI framework for Rust
