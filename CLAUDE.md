# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
Git Commit Analyzer is a Rust-based Git plugin that uses MLX AI to generate meaningful commit messages from staged changes. It follows Git Flow format conventions, supports multiple languages, and provides both a CLI tool and a VS Code extension for enhanced developer experience. The application uses a Python bridge to integrate with MLX for local AI processing on Apple Silicon devices.

## Core Architecture
- **Primary Language**: Rust (edition 2021) for CLI tool
- **Extension Language**: TypeScript for VS Code integration
- **Python Bridge**: `generate_commit.py` for MLX integration
- **Entry Point**: `src/main.rs:674` - main function handles CLI arguments and orchestrates the workflow
- **Key Dependencies**:
  - `git2` for Git operations
  - MLX-LM Python package for AI model inference
  - Apple Silicon optimization with Metal acceleration
- **Project Structure**: Single binary crate with Python bridge script and separate VS Code extension in `vscode-extension/`
- **One-Click Installation**: Automated installation script at `install-git-ca.sh` for cross-platform deployment

## Development Commands

### Rust CLI Tool
```bash
cargo build --release          # Build release binary
cargo run                      # Run in debug mode
cargo run -- model             # Change default MLX model
cargo run -- language          # Change default output language
cargo check                    # Quick check for compilation errors
cargo clippy                   # Lint code
cargo fmt                      # Format code
cargo test                     # Run tests (no test framework currently configured)
```

### Python MLX Bridge
```bash
python3 generate_commit.py --list-models  # Show available MLX models
python3 generate_commit.py --help         # Show Python script options
```

### Testing Commands
```bash
# Manual testing workflow
git add .                      # Stage changes
cargo run                      # Test commit generation
./target/release/git-ca        # Test compiled binary
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
- `main()`: CLI entry point at line 674 - handles argument parsing, model/language selection, and commit workflow
- `check_python_mlx()`: Validates Python and MLX-LM installation at line 645
- `analyze_diff()`: AI message generation via Python bridge at line 394
- `find_git_repository()`: Locates repo from current directory at line 297
- `get_diff()`: Gets staged changes via `git diff --cached` at line 309
- `select_language()`: Interactive language selection at line 576
- `get_language()`: Gets configured language with English default at line 596
- `select_default_model()`: Interactive MLX model selection at line 604
- **Key Constants**: `MLX_MODELS`, `DEFAULT_MLX_MODEL`, `COMMIT_TYPES`, `CONFIG_MODEL_KEY`, `CONFIG_LANGUAGE_KEY`

### Python MLX Bridge (`generate_commit.py`)
- `generate_commit_message()`: Main MLX inference function
- `build_commit_prompt()`: Language-specific prompt generation for MLX
- `AVAILABLE_MODELS`: Dictionary mapping model names to MLX Hub paths
- Uses `mlx_lm.load()` and `mlx_lm.stream_generate()` for inference
- Handles model loading, text streaming, and error recovery
- Supports both English and Chinese prompt generation

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
- MLX model selection stored in `commit-analyzer.model` key
- Language selection stored in `commit-analyzer.language` key (English default)
- User info auto-configured from Git settings
- Support for English and Simplified Chinese output languages
- **Git Config Keys**: `commit-analyzer.model`, `commit-analyzer.language`, `user.name`, `user.email`
- **Key Constants**: `MLX_MODELS`, `DEFAULT_MLX_MODEL`, `CONFIG_MODEL_KEY`, `CONFIG_LANGUAGE_KEY`, `COMMIT_TYPES`

### MLX Integration
- **Python Bridge**: Subprocess communication with `generate_commit.py`
- **Model Loading**: Uses `mlx_lm.load()` with Apple Silicon optimization
- **Streaming Generation**: Real-time text generation via `mlx_lm.stream_generate()`
- **Dependency Validation**: Checks Python3 and MLX-LM availability before processing
- **Default Model**: `gemma-3-270m-it-6bit` (optimized for most Apple Silicon devices)
- **Available Models**: Gemma-3-270M, Gemma-2B, Mistral-7B, Llama-3-8B, Phi-3-mini
- **Threading**: Thread-safe subprocess I/O handling with Arc<Mutex<Vec<u8>>>
- **Error Handling**: Comprehensive error recovery for subprocess failures
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

## Important Architecture Patterns
- **Error Handling**: Uses `AppError` enum for unified error management with `GitError`, `NetworkError`, `ConfigError`, `Custom` variants
- **Language System**: `Language` enum with `English` and `Chinese` variants, supporting conversion and display methods
- **Interactive Selection**: Model and language selection via interactive CLI prompts with fallback to defaults
- **Git Operations**: Uses `git2` crate for repository detection, config management, and diff generation
- **Subprocess Communication**: Thread-safe I/O handling with Arc<Mutex<Vec<u8>>> for concurrent stdout/stderr capture
- **Python Bridge Integration**: Clean separation between Rust CLI and Python MLX processing via subprocess calls
- **Streaming Output**: Real-time text generation from MLX models with immediate user feedback

## Installation Script Details
The `install-git-ca.sh` script provides automated cross-platform installation:
- **OS Detection**: Automatically identifies macOS, Debian/Ubuntu, Fedora/CentOS, Arch, openSUSE
- **Dependency Management**: Installs Git, Rust, Python, and MLX-LM
- **Environment Setup**: Configures PATH and shell integration
- **Interactive Configuration**: Guides users through Git and Python/MLX setup
- **Error Recovery**: Provides fallbacks and troubleshooting guidance
- **CDN Hosted**: Available at `https://sh.zhanghe.dev/install-git-ca.sh` for one-click installation

## Error Handling
- Python and MLX-LM dependency validation before processing
- Git repository detection
- Staged changes validation
- MLX model selection fallback
- Language selection with English default
- Custom error types with unified handling (AppError enum)
- Binary path discovery for VS Code extension
- **Error Types**: `AppError` enum with `GitError`, `NetworkError`, `ConfigError`, `Custom` variants
- **Installation Script Robustness**: Cross-platform OS detection, dependency auto-installation, interactive fallbacks
- **Subprocess Error Recovery**: Comprehensive handling of Python script failures and MLX-LM import errors

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.