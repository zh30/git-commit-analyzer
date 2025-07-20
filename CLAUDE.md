# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
Git Commit Analyzer is a Rust-based Git plugin that uses Ollama AI to generate meaningful commit messages from staged changes. It follows Git Flow format conventions and provides an interactive interface for users to accept, edit, or cancel AI-generated messages.

## Core Architecture
- **Language**: Rust (edition 2021)
- **Entry Point**: `src/main.rs:266` - main function handles CLI arguments and orchestrates the workflow
- **Key Dependencies**: 
  - `git2` for Git operations
  - `reqwest` for Ollama API communication
  - `serde_json` for JSON handling

## Development Commands

### Build & Run
```bash
cargo build --release          # Build release binary
cargo run                      # Run in debug mode
cargo run -- model            # Change default Ollama model
```

### Development Workflow
```bash
cargo check                    # Quick check for compilation errors
cargo clippy                  # Lint code
cargo fmt                     # Format code
cargo test                    # Run tests
```

### Manual Installation for Testing
```bash
# After building
cp target/release/git-ca ~/.git-plugins/
# Add ~/.git-plugins to PATH
```

## Key Components

### Core Functions
- `main()`: CLI entry point at `src/main.rs:266`
- `analyze_diff()`: AI message generation at `src/main.rs:30`
- `get_diff()`: Gets staged changes via `git diff --cached` at `src/main.rs:25`
- `find_git_repository()`: Locates repo from current directory at `src/main.rs:12`

### Configuration Management
- Git config integration via `git2::Config`
- Model selection stored in `commit-analyzer.model` key
- User info auto-configured from Git settings

### Ollama Integration
- API base URL: `http://localhost:11434/api`
- Model listing via `/tags` endpoint
- Streaming response handling for real-time generation

## Usage Patterns
- Primary command: `git ca` (after installation)
- Model management: `git ca model`
- Version check: `git ca --version`

## Testing Workflow
1. Stage changes with `git add`
2. Run `./target/release/git-ca` or `cargo run`
3. Interactive prompt allows using, editing, or canceling

## Error Handling
- Ollama connection validation before processing
- Git repository detection
- Staged changes validation
- Model selection fallback