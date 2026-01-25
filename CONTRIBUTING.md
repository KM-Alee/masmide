# Contributing to masmide

Thank you for your interest in contributing to masmide! This document provides guidelines and information for contributors.

## Code of Conduct

Please be respectful and constructive in all interactions. We welcome contributors of all experience levels.

## How to Contribute

### Reporting Bugs

1. Check existing [issues](https://github.com/KM-Alee/masmide/issues) to avoid duplicates
2. Use the bug report template
3. Include:
   - masmide version (`masmide --version`)
   - Operating system and version
   - Steps to reproduce
   - Expected vs actual behavior
   - Relevant error messages or screenshots

### Suggesting Features

1. Check existing issues for similar suggestions
2. Open a new issue with the "feature request" label
3. Describe the feature and its use case
4. Be open to discussion and feedback

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Run tests and lints (see below)
5. Commit with clear messages: `git commit -m "Add: description of change"`
6. Push to your fork: `git push origin feature/your-feature`
7. Open a Pull Request against `main`

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- JWasm, MinGW-w64, Wine (for testing builds)

### Building

```bash
git clone https://github.com/KM-Alee/masmide.git
cd masmide

# Debug build
cargo build

# Release build
cargo build --release

# Run
cargo run -- path/to/project
```

### Code Style

We use standard Rust tooling:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test
```

**Before submitting a PR, ensure:**
- `cargo fmt --check` passes
- `cargo clippy -- -D warnings` passes
- `cargo test` passes
- `cargo build --release` succeeds

### Project Structure

```
src/
├── main.rs          # Entry point, CLI args
├── app.rs           # Main application state
├── input.rs         # Input handling
├── config.rs        # Configuration loading
├── project.rs       # Project management
├── syntax.rs        # Syntax highlighting
├── autocomplete.rs  # Autocompletion logic
├── diagnostics.rs   # Error parsing
├── docs.rs          # Documentation provider
├── theme.rs         # Color themes
└── ui/
    ├── mod.rs       # UI module exports
    ├── layout.rs    # Layout management
    ├── editor.rs    # Editor widget
    ├── file_tree.rs # File tree widget
    ├── tabs.rs      # Tab bar
    ├── status_bar.rs
    ├── output.rs    # Build output panel
    └── ...
```

### Commit Message Format

Use clear, descriptive commit messages:

- `Add: new feature description`
- `Fix: bug description`
- `Refactor: what was refactored`
- `Docs: documentation changes`
- `Style: formatting, no code change`
- `Test: adding or updating tests`

### Testing

Currently, the project has limited automated tests. Contributions to improve test coverage are especially welcome!

When adding new features:
1. Add unit tests where applicable
2. Manually test the feature in the TUI
3. Test with actual MASM files if the feature relates to assembly

## Architecture Notes

### UI Framework

masmide uses [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI. Key concepts:
- Widgets are stateless - they render based on application state
- Input events are handled in `input.rs` and dispatch actions
- The main loop in `main.rs` orchestrates rendering and input

### Build System

The build process (in `app.rs`):
1. JWasm assembles `.asm` to `.obj`
2. MinGW-w64 linker creates Windows PE executable
3. Wine runs the executable

### Configuration

Config is loaded from `.masmide.toml` via `config.rs` using `serde` and `toml`.

## Getting Help

- Open an issue for questions
- Check existing issues and discussions
- Read the source code and comments

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
