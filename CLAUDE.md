# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
Git Commit Analyzer is a Rust-based Git plugin that uses Ollama AI to generate meaningful commit messages from staged changes. It follows Git Flow format conventions, supports multiple languages, and provides both a CLI tool and a VS Code extension for enhanced developer experience.

## Core Architecture
- **Primary Language**: Rust (edition 2021) for CLI tool
- **Extension Language**: TypeScript for VS Code integration
- **Entry Point**: `src/main.rs:645` - main function handles CLI arguments and orchestrates the workflow
- **Key Dependencies**: 
  - `git2` for Git operations
  - `reqwest` for Ollama API communication (with blocking and json features)
  - `serde_json` for JSON handling
- **Project Structure**: Single binary crate with separate VS Code extension in `vscode-extension/`
- **One-Click Installation**: Automated installation script at `install-git-ca.sh` for cross-platform deployment

## Development Commands

### Rust CLI Tool
```bash
cargo build --release          # Build release binary
cargo run                      # Run in debug mode
cargo run -- model             # Change default Ollama model
cargo run -- language          # Change default output language
cargo check                    # Quick check for compilation errors
cargo clippy                   # Lint code
cargo fmt                      # Format code
cargo test                     # Run tests (no test framework currently configured)
```

### VS Code Extension
```bash
cd vscode-extension
npm run compile                 # Compile TypeScript
npm run watch                   # Watch mode for development
npm run package                 # Package as .vsix file
npm run publish                 # Publish to marketplace
npm run vscode:prepublish       # Prepare for publishing
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
- `main()`: CLI entry point at line 645 - handles argument parsing, model/language selection, and commit workflow
- `find_git_repository()`: Locates repo from current directory at line 297
- `get_diff()`: Gets staged changes via `git diff --cached` at line 309
- `build_commit_prompt()`: Language-specific prompt generation at line 316
- `analyze_diff()`: AI message generation at line 404 (now supports language parameter)
- `process_ollama_response()`: Post-processes AI output at line 471
- `select_language()`: Interactive language selection at line 576
- `get_language()`: Gets configured language with English default at line 596
- `select_default_model()`: Interactive model selection at line 604
- **Key Constants**: `OLLAMA_API_BASE`, `COMMIT_TYPES`, `CONFIG_MODEL_KEY`, `CONFIG_LANGUAGE_KEY`

### VS Code Extension (`vscode-extension/src/extension.ts`)
- Command registration: `gitCommitAnalyzer.generateMessage`
- Binary discovery with fallback paths
- SCM integration with buttons and context menus
- Progress indication during AI generation
- **Extension Activation**: `onStartupFinished` and command-based activation
- **UI Integration**: SCM/title and scm/resourceGroup/context menus
- **Dependencies**: VS Code API >= 1.74.0, TypeScript 4.9.5

### Configuration Management
- Git config integration via `git2::Config`
- Model selection stored in `commit-analyzer.model` key
- Language selection stored in `commit-analyzer.language` key (English default)
- User info auto-configured from Git settings
- Support for English and Simplified Chinese output languages
- **Git Config Keys**: `commit-analyzer.model`, `commit-analyzer.language`, `user.name`, `user.email`

### Ollama Integration
- API base URL: `http://localhost:11434/api`
- Model listing via `/tags` endpoint
- Streaming response handling for real-time generation
- Connection validation before processing
- Enforces Git Flow commit message format: `<type>(<scope>): <subject>` with optional body
- Supported commit types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
- Generates single commit message per invocation without issue numbers or footers

## Distribution Methods
- **One-Click Installation**: `bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)" - automated cross-platform installer
- **Homebrew**: `brew tap zh30/tap && brew install git-ca`
- **Manual**: Build and install to `~/.git-plugins/`
- **VS Code Extension**: Package as `.vsix` and install
- **Multi-language docs**: README files in EN, ZH, FR, ES
- **CDN Distribution**: Installation script hosted at `https://sh.zhanghe.dev/install-git-ca.sh`

## Usage Patterns
- Primary command: `git ca` (after installation)
- Model management: `git ca model`
- Language selection: `git ca language` (English/Chinese)
- Version check: `git ca --version`
- VS Code: Use wand icon in SCM panel or context menu

## Testing Workflow
1. Stage changes with `git add`
2. Run `./target/release/git-ca` or `cargo run`
3. Interactive prompt allows using, editing, or canceling
4. VS Code: Click generate button and approve in input box

## Installation Script Details
The `install-git-ca.sh` script provides automated cross-platform installation:
- **OS Detection**: Automatically identifies macOS, Debian/Ubuntu, Fedora/CentOS, Arch, openSUSE
- **Dependency Management**: Installs Git, Rust, and configures Ollama
- **Environment Setup**: Configures PATH and shell integration
- **Interactive Configuration**: Guides users through Git and Ollama setup
- **Error Recovery**: Provides fallbacks and troubleshooting guidance
- **CDN Hosted**: Available at `https://sh.zhanghe.dev/install-git-ca.sh` for one-click installation

## Error Handling
- Ollama connection validation before processing
- Git repository detection
- Staged changes validation
- Model selection fallback
- Language selection with English default
- Custom error types with unified handling (AppError enum)
- Binary path discovery for VS Code extension
- **Error Types**: `AppError` enum with `GitError`, `NetworkError`, `ConfigError`, `Custom` variants
- **Installation Script Robustness**: Cross-platform OS detection, dependency auto-installation, interactive fallbacks

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.