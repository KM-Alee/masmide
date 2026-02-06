# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-02-06

### Changed
- **Clipboard system rewrite** — single `Clipboard` struct replaces scattered `Option<arboard::Clipboard>` + `yank_buffer` fields across the codebase
- Explicit `YankType` enum (`Line` / `Char`) replaces fragile `text.ends_with('\n')` heuristic for detecting line-wise vs character-wise paste
- Clipboard now uses `wl-copy`/`wl-paste` (Wayland) or `xclip` (X11) as primary mechanism, with `arboard` as fallback — fixes system clipboard not persisting in TUI mode

### Fixed
- System clipboard copy not working on Wayland — text yanked in the editor now correctly appears in external applications
- Line-wise vs character-wise paste detection was unreliable when system clipboard and internal buffer desynced

## [0.1.0] - 2024-01-24

### Added
- Initial release of masmide
- TUI-based code editor with syntax highlighting for MASM
- File tree navigation with keyboard controls
- Build integration with JWasm assembler
- Linking with MinGW-w64 for Windows PE executables
- Run executables using Wine
- Autocomplete for MASM instructions and directives
- Autocomplete for Irvine32 library procedures
- Inline documentation on hover for instructions and procedures
- Search and replace functionality
- Multiple file tabs
- Project configuration via `.masmide.toml`
- Three built-in themes: dark, light, and Dracula
- Command palette for quick actions
- Status bar with cursor position and file info
- Output panel for build results and errors
- Error diagnostics with line highlighting
- New project scaffolding with `--new` flag
- Bundled Irvine32 library for easy setup
- Clipboard support (copy, cut, paste)
- Undo/redo support

### Dependencies
- JWasm (MASM-compatible assembler)
- MinGW-w64 (cross-compiler/linker)
- Wine (Windows executable runner)

[Unreleased]: https://github.com/KM-Alee/masmide/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/KM-Alee/masmide/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/KM-Alee/masmide/releases/tag/v0.1.0
