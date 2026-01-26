<h1 align="center">masmide</h1>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/KM-Alee/masmide/actions/workflows/ci.yml/badge.svg)](https://github.com/KM-Alee/masmide/actions/workflows/ci.yml)
=======
<p align="center">
  <strong>A modern TUI IDE for x86 Assembly development on Linux</strong>
</p>


<p align="center">
  <a href="https://github.com/KM-Alee/masmide/releases/latest"><img src="https://img.shields.io/github/v/release/KM-Alee/masmide?style=flat-square&color=blue" alt="Release"></a>
  <a href="https://crates.io/crates/masmide"><img src="https://img.shields.io/crates/v/masmide?style=flat-square&color=orange" alt="Crates.io"></a>
  <a href="https://github.com/KM-Alee/masmide/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/KM-Alee/masmide/ci.yml?style=flat-square" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="License"></a>
</p>

<p align="center">
  Write, build, and run MASM programs without leaving the terminal.<br>
  Powered by JWasm, MinGW-w64, and Wine.
</p>

---

<p align="center">
  <img src="https://raw.githubusercontent.com/KM-Alee/masmide/main/assets/screenshot.png" alt="Screenshot" width="800">
</p>

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🎨 **Syntax Highlighting** | Full MASM syntax support with multiple themes (Dark, Light, Gruvbox, Dracula) |
| ⚡ **One-Key Build** | Press `F5` to build and run instantly |
| 💡 **Smart Autocomplete** | Context-aware completion for instructions, registers, and Irvine32 procedures |
| 📖 **Inline Documentation** | Hover over any instruction to see its description |
| 📁 **File Explorer** | Navigate your project with a keyboard-driven file tree |
| 🔍 **Search & Replace** | Find text across files with regex support |
| ⌨️ **Vim Keybindings** | Navigate and edit like a pro |
| 🔧 **Configurable** | Per-project settings via `.masmide.toml` |

## 🚀 Quick Install

```bash
curl -sSL https://raw.githubusercontent.com/KM-Alee/masmide/main/scripts/install-remote.sh | bash
```

This installs everything: masmide, JWasm, MinGW-w64, Wine, and the Irvine32 library.

<details>
<summary><strong>Other installation methods</strong></summary>

### From GitHub Releases

Download the [latest release](https://github.com/KM-Alee/masmide/releases/latest) for your architecture:

| Binary | Platform |
|--------|----------|
| `masmide-*-linux-x86_64.tar.gz` | Intel/AMD 64-bit |
| `masmide-*-linux-aarch64.tar.gz` | ARM64 (Raspberry Pi 4, etc.) |
| `masmide-*-linux-x86_64-musl.tar.gz` | Static binary (Alpine, etc.) |

```bash
tar -xzf masmide-*.tar.gz && cd masmide-* && sudo ./install.sh
```

### From crates.io

```bash
cargo install masmide
```

### From Source

```bash
git clone https://github.com/KM-Alee/masmide.git
cd masmide && ./install.sh
```

</details>

## 📦 Quick Start

```bash
# Create a new project
masmide --new hello
cd hello

# Open the IDE
masmide
```

Press `F5` to build and run your program.

## ⌨️ Keybindings

### Essential

| Key | Action |
|-----|--------|
| `F5` | Build & Run |
| `F6` | Build only |
| `Ctrl+S` | Save |
| `:q` | Quit |
| `F1` | Help |
| `Tab` | Switch panel |

### Navigation

| Key | Action |
|-----|--------|
| `h` `j` `k` `l` | Move cursor (Vim-style) |
| `gg` | Go to first line |
| `G` | Go to last line |
| `:42` | Go to line 42 |
| `Ctrl+F` | Search |

### Editing

| Key | Action |
|-----|--------|
| `i` | Insert mode |
| `v` | Visual mode |
| `Esc` | Normal mode |
| `u` | Undo |
| `Ctrl+R` | Redo |
| `dd` | Delete line |
| `yy` | Copy line |
| `p` | Paste |

### File Tree

| Key | Action |
|-----|--------|
| `Enter` | Open / Expand |
| `n` | New file |
| `N` | New folder |
| `r` | Rename |
| `d` | Delete |

## ⚙️ Configuration

Create `.masmide.toml` in your project root:

```toml
[build]
assembler = "jwasm"
linker = "i686-w64-mingw32-ld"
lib_paths = ["/usr/local/lib/irvine"]
include_paths = ["/usr/local/include/irvine"]
libs = ["Irvine32", "Kernel32", "User32"]

[editor]
theme = "gruvbox"    # dark, light, gruvbox, dracula
tab_width = 4
use_spaces = true

[run]
runner = "wine"
```

## 📝 Example Program

```asm
INCLUDE Irvine32.inc

.data
    msg BYTE "Hello from masmide!", 0

.code
main PROC
    mov  edx, OFFSET msg
    call WriteString
    call Crlf
    exit
main ENDP
END main
```

## 🔧 Troubleshooting

<details>
<summary><strong>jwasm: command not found</strong></summary>

Build from source:
```bash
git clone https://github.com/JWasm/JWasm.git
cd JWasm && make -f GccUnix.mak
sudo cp build/GccUnixR/jwasm /usr/local/bin/
```
</details>

<details>
<summary><strong>cannot find -lIrvine32</strong></summary>

Run the installer or manually copy libraries:
```bash
sudo mkdir -p /usr/local/lib/irvine
sudo cp Irvine/*.lib /usr/local/lib/irvine/
```
</details>

<details>
<summary><strong>Wine errors</strong></summary>

Configure Wine:
```bash
winecfg
```

For 32-bit executables, ensure wine32 is installed:
```bash
# Debian/Ubuntu
sudo dpkg --add-architecture i386
sudo apt install wine32
```
</details>

## 🗑️ Uninstall

```bash
./uninstall.sh
# or
cargo uninstall masmide
```

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

MIT © [KM-Alee](https://github.com/KM-Alee)

---

<p align="center">
  <strong>Acknowledgments</strong><br>
  <a href="http://asmirvine.com/">Irvine32</a> •
  <a href="https://github.com/JWasm/JWasm">JWasm</a> •
  <a href="https://github.com/ratatui-org/ratatui">Ratatui</a>
</p>
