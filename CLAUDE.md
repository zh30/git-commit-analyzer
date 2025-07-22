# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
Git Commit Analyzer is a Rust-based Git plugin that uses Ollama AI to generate meaningful commit messages from staged changes. It follows Git Flow format conventions and provides both a CLI tool and a VS Code extension for enhanced developer experience.

## Core Architecture
- **Primary Language**: Rust (edition 2021) for CLI tool
- **Extension Language**: TypeScript for VS Code integration
- **Entry Point**: `src/main.rs:266` - main function handles CLI arguments and orchestrates the workflow
- **Key Dependencies**: 
  - `git2` for Git operations
  - `reqwest` for Ollama API communication
  - `serde_json` for JSON handling

## Development Commands

### Rust CLI Tool
```bash
cargo build --release          # Build release binary
cargo run                      # Run in debug mode
cargo run -- model            # Change default Ollama model
cargo check                    # Quick check for compilation errors
cargo clippy                   # Lint code
cargo fmt                      # Format code
cargo test                     # Run tests
```

### VS Code Extension
```bash
cd vscode-extension
npm run compile                 # Compile TypeScript
npm run watch                   # Watch mode for development
npm run package                 # Package as .vsix file
npm run publish                 # Publish to marketplace
```

### Manual Installation for Testing
```bash
# CLI tool
cargo build --release
cp target/release/git-ca ~/.git-plugins/
# Add ~/.git-plugins to PATH

# VS Code extension
cd vscode-extension && npm run package
# Install .vsix file in VS Code
```

## Key Components

### Core Functions (`src/main.rs`)
- `main()`: CLI entry point at line 266
- `analyze_diff()`: AI message generation at line 30
- `get_diff()`: Gets staged changes via `git diff --cached` at line 25
- `find_git_repository()`: Locates repo from current directory at line 12
- `process_ollama_response()`: Post-processes AI output at line 126
- `select_default_model()`: Interactive model selection at line 230

### VS Code Extension (`vscode-extension/src/extension.ts`)
- Command registration: `gitCommitAnalyzer.generateMessage`
- Binary discovery with fallback paths
- SCM integration with buttons and context menus
- Progress indication during AI generation

### Configuration Management
- Git config integration via `git2::Config`
- Model selection stored in `commit-analyzer.model` key
- User info auto-configured from Git settings

### Ollama Integration
- API base URL: `http://localhost:11434/api`
- Model listing via `/tags` endpoint
- Streaming response handling for real-time generation
- Connection validation before processing

## Distribution Methods
- **Homebrew**: `brew tap zh30/tap && brew install git-ca`
- **Manual**: Build and install to `~/.git-plugins/`
- **VS Code Extension**: Package as `.vsix` and install
- **Multi-language docs**: README files in EN, ZH, FR, ES

## Usage Patterns
- Primary command: `git ca` (after installation)
- Model management: `git ca model`
- Version check: `git ca --version`
- VS Code: Use wand icon in SCM panel or context menu

## Testing Workflow
1. Stage changes with `git add`
2. Run `./target/release/git-ca` or `cargo run`
3. Interactive prompt allows using, editing, or canceling
4. VS Code: Click generate button and approve in input box

## Error Handling
- Ollama connection validation before processing
- Git repository detection
- Staged changes validation
- Model selection fallback
- Binary path discovery for VS Code extension