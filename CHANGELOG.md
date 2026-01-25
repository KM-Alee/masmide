# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/KM-Alee/masmide/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/KM-Alee/masmide/releases/tag/v0.1.0
