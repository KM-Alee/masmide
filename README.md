<div align="center">

# ⚙️ masmide

**A blazing-fast TUI IDE for x86 Assembly — write, build, and run MASM right in your terminal.**

[![Release](https://img.shields.io/github/v/release/KM-Alee/masmide?style=for-the-badge&color=blue&logo=github)](https://github.com/KM-Alee/masmide/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/KM-Alee/masmide/ci.yml?style=for-the-badge&label=CI&logo=githubactions)](https://github.com/KM-Alee/masmide/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)](LICENSE)

<br>

<img src="https://raw.githubusercontent.com/KM-Alee/masmide/main/assets/screenshot.png" alt="masmide screenshot" width="820">

<br>

*Powered by [JWasm](https://github.com/JWasm/JWasm) · [MinGW-w64](https://www.mingw-w64.org/) · [Wine](https://www.winehq.org/) · [Ratatui](https://github.com/ratatui-org/ratatui)*

</div>

---

## ✨ Why masmide?

Most assembly courses require Windows + Visual Studio. **masmide** lets you do everything from a Linux terminal:

- 🎨 **Syntax highlighting** — full MASM support with 4 themes (Dark, Light, Gruvbox, Dracula)
- ⚡ **One-key build & run** — press `F5` and see output instantly
- 💡 **Smart autocomplete** — instructions, registers, Irvine32 procedures
- 📖 **Inline docs** — hover any instruction for a description
- 📁 **File explorer** — keyboard-driven project navigation
- 🔍 **Search** — find text across your project
- ⌨️ **Vim keybindings** — `hjkl`, visual mode, `dd`, `yy`, `p`, the works
- 📋 **System clipboard** — copy/paste works with your desktop (Wayland & X11)
- 🔧 **Per-project config** — `.masmide.toml` for custom build settings

---

## 🚀 Install

### One-liner (recommended)

```bash
curl -sSL https://raw.githubusercontent.com/KM-Alee/masmide/main/scripts/install-remote.sh | bash
```

> Installs masmide + JWasm + MinGW-w64 + Wine + Irvine32 — everything you need.

<details>
<summary><b>📦 Other methods</b></summary>

#### From GitHub Releases

| Archive | Platform |
|---------|----------|
| `masmide-*-linux-x86_64.tar.gz` | Intel/AMD 64-bit |
| `masmide-*-linux-aarch64.tar.gz` | ARM64 (RPi 4, etc.) |
| `masmide-*-linux-x86_64-musl.tar.gz` | Static binary (Alpine) |

```bash
tar -xzf masmide-*.tar.gz && cd masmide-* && sudo ./install.sh
```

#### From crates.io

```bash
cargo install masmide
```

#### From source

```bash
git clone https://github.com/KM-Alee/masmide.git
cd masmide && ./install.sh
```

</details>

---

## 📦 Quick Start

```bash
masmide --new hello    # scaffold a new project
cd hello
masmide                # open the IDE
```

Press **`F5`** to build and run. That's it.

---

## ⌨️ Keybindings

### Core

| Key | Action |
|:---:|--------|
| `F5` | Build & Run |
| `F6` | Build only |
| `Ctrl+S` | Save |
| `:q` | Quit |
| `F1` | Help |
| `Tab` | Switch panel |

### Navigation (Normal mode)

| Key | Action |
|:---:|--------|
| `h` `j` `k` `l` | Move cursor |
| `gg` | First line |
| `G` | Last line |
| `:42` | Go to line 42 |
| `Ctrl+F` | Search |
| `w` / `b` | Next / prev word |

### Editing

| Key | Action |
|:---:|--------|
| `i` | Insert mode |
| `v` | Visual mode |
| `V` | Visual line mode |
| `Esc` | Back to Normal |
| `u` / `Ctrl+R` | Undo / Redo |
| `dd` | Delete line |
| `yy` | Yank (copy) line |
| `p` / `P` | Paste after / before |
| `Ctrl+V` | Paste from system clipboard (Insert mode) |

### File Tree

| Key | Action |
|:---:|--------|
| `Enter` | Open / Expand |
| `n` | New file |
| `N` | New folder |
| `r` | Rename |
| `d` | Delete |

---

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
theme = "gruvbox"    # dark | light | gruvbox | dracula
tab_width = 4
use_spaces = true

[run]
runner = "wine"
```

---

## 📝 Example

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

---

## 🔧 Troubleshooting

<details>
<summary><code>jwasm: command not found</code></summary>

```bash
git clone https://github.com/JWasm/JWasm.git
cd JWasm && make -f GccUnix.mak
sudo cp build/GccUnixR/jwasm /usr/local/bin/
```
</details>

<details>
<summary><code>cannot find -lIrvine32</code></summary>

```bash
sudo mkdir -p /usr/local/lib/irvine
sudo cp Irvine/*.lib /usr/local/lib/irvine/
```
</details>

<details>
<summary>Wine errors</summary>

```bash
winecfg   # configure Wine

# For 32-bit support (Debian/Ubuntu):
sudo dpkg --add-architecture i386
sudo apt install wine32
```
</details>

---

## 🗑️ Uninstall

```bash
./uninstall.sh
# or
cargo uninstall masmide
```

---

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

MIT © [KM-Alee](https://github.com/KM-Alee)

---

<div align="center">

**Built with** ❤️ **using [Ratatui](https://github.com/ratatui-org/ratatui)**

[Irvine32](http://asmirvine.com/) · [JWasm](https://github.com/JWasm/JWasm) · [MinGW-w64](https://www.mingw-w64.org/)

</div>
