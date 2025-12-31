# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-XX

### Added

#### Core Features
- Initial Rust port of [Streamdown](https://github.com/kristopolous/Streamdown)
- Streaming markdown parsing - renders content as it arrives
- Full markdown support:
  - Headings (h1-h6) with centered rendering
  - Code blocks with syntax highlighting
  - Inline code with backtick formatting
  - Bold (`**text**`) and italic (`*text*`)
  - Strikethrough (`~~text~~`)
  - Tables with Unicode box drawing
  - Ordered and unordered lists (nested)
  - Blockquotes (nested)
  - Horizontal rules
  - Links with OSC 8 hyperlinks
- Think blocks (`<think>...</think>`) for LLM reasoning output

#### Syntax Highlighting
- Powered by [syntect](https://github.com/trishume/syntect)
- 100+ language support
- Language aliases (e.g., `py` → `python`)
- Automatic language detection

#### Terminal Features
- ANSI escape code rendering
- True color (24-bit) support
- OSC 52 clipboard integration
- OSC 8 hyperlinks
- Unicode box drawing characters
- Proper CJK character width handling

#### PTY Exec Mode
- Execute commands in a pseudo-terminal
- Render markdown output in real-time
- Keyboard passthrough
- Prompt detection for output flushing

#### Savebrace
- Save code blocks to temp file (`/tmp/savebrace`)
- Null-byte separated for easy parsing
- Shell integration for accessing code blocks

#### LaTeX Plugin
- Convert LaTeX math to Unicode symbols
- Greek letters (`\alpha` → α)
- Math operators (`\times` → ×)
- Superscripts and subscripts
- Fractions

#### Configuration
- TOML configuration file support
- XDG config directory support
- HSV-based color theme generation
- Customizable margins and styling
- Feature toggles (clipboard, savebrace)

#### CLI
- File input or stdin streaming
- `--exec` mode for running commands
- `--width` override for terminal width
- `--config` for custom config file
- `--scrape` to save code blocks to directory
- `--debug` for troubleshooting

#### Security
- Terminal escape sequence sanitization
- URL validation for OSC 8 hyperlinks
- Symlink attack prevention for savebrace
- Path traversal prevention in code scraping

#### Testing
- 370+ unit tests
- Integration tests with Python Streamdown test files
- Snapshot tests for output verification
- Property-based tests with proptest
- Doc tests for all public APIs

#### Documentation
- Comprehensive README with examples
- API documentation with `cargo doc`
- Example programs

### Architecture

- **streamdown-core**: Core types, traits, parse state
- **streamdown-ansi**: ANSI codes, colors, sanitization
- **streamdown-config**: TOML config, style computation
- **streamdown-parser**: Streaming markdown parser
- **streamdown-syntax**: Syntax highlighting via syntect
- **streamdown-render**: Terminal renderer
- **streamdown-plugin**: Plugin system (LaTeX)

### Platform Support

| Platform | Status |
|----------|--------|
| Linux    | ✅ Full support |
| macOS    | ✅ Full support |
| Windows  | ⚠️ Partial (no PTY) |

### Dependencies

- `syntect` 5.2 - Syntax highlighting
- `clap` 4.5 - CLI argument parsing
- `toml` 0.8 - Configuration parsing
- `serde` 1.0 - Serialization
- `crossterm` 0.28 - Terminal handling
- `regex` 1.10 - Pattern matching
- `thiserror` 2.0 - Error handling
- `unicode-width` 0.2 - Character width
- `directories` 5.0 - XDG paths
- `nix` 0.29 - Unix PTY (Unix only)

[Unreleased]: https://github.com/fed-stew/streamdown-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/fed-stew/streamdown-rs/releases/tag/v0.1.0
