# Git Commit Analyzer

[中文](README_ZH.md) · [Français](README_FR.md) · [Español](README_ES.md)

Git Commit Analyzer is a Rust-based Git plugin that generates Git Flow–style commit messages from your staged diff using a local llama.cpp model. The CLI summarises large diffs, validates model output, and falls back to deterministic messages when needed.

## Key Features

- **Local inference**: Uses `llama_cpp_sys` to run GGUF models without any remote API.
- **Smart diff summarisation**: Large lockfiles and generated assets are reduced to concise summaries before prompting.
- **Git Flow enforcement**: Ensures responses match `<type>(<scope>): <subject>` and retries/falls back when they do not.
- **Interactive CLI**: Review, edit, or cancel the generated commit message.
- **Multi-language prompts**: English (default) and Simplified Chinese.
- **Configurable context**: Tune llama context length via Git configuration.
- **Multi-platform binaries**: Pre-built binaries for macOS (Intel & Apple Silicon).

## Requirements

- Git 2.30+
- A local GGUF model (the CLI can download the default `unsloth/gemma-3-270m-it-GGUF`)

## Installation

### Homebrew (Recommended) - Fast Binary Installation

**macOS users can install via Homebrew with pre-built binaries (no Rust compilation required):**

```bash
brew tap zh30/tap
brew install git-ca
```

This installs a pre-built binary for your platform:
- **macOS**: Apple Silicon (M1/M2/M3) and Intel (x86_64)

No Rust toolchain or compilation needed!

**Note**: Linux builds are temporarily disabled due to compilation issues. Windows builds are available via [GitHub Releases](https://github.com/zh30/git-commit-analyzer/releases).

### Manual Installation

Download the appropriate binary for your platform from [Releases](https://github.com/zh30/git-commit-analyzer/releases):

```bash
# macOS (Apple Silicon)
curl -L -o git-ca https://github.com/zh30/git-commit-analyzer/releases/download/v1.1.2/git-ca-1.1.2-apple-darwin-arm64.tar.gz
tar -xzf git-ca-1.1.2-apple-darwin-arm64.tar.gz
sudo mv git-ca /usr/local/bin/
chmod +x /usr/local/bin/git-ca

# macOS (Intel)
curl -L -o git-ca https://github.com/zh30/git-commit-analyzer/releases/download/v1.1.2/git-ca-1.1.2-apple-darwin-x86_64.tar.gz
tar -xzf git-ca-1.1.2-apple-darwin-x86_64.tar.gz
sudo mv git-ca /usr/local/bin/
chmod +x /usr/local/bin/git-ca
```
**Note**: Linux builds are temporarily disabled due to compilation issues. Windows builds are available via [GitHub Releases](https://github.com/zh30/git-commit-analyzer/releases).

### Build from Source

If you prefer to build from source:

```bash
git clone https://github.com/zh30/git-commit-analyzer.git
cd git-commit-analyzer
cargo build --release
mkdir -p ~/.git-plugins
cp target/release/git-ca ~/.git-plugins/
echo 'export PATH="$HOME/.git-plugins:$PATH"' >> ~/.bashrc   # adapt for your shell
source ~/.bashrc
```

### One-Line Bootstrap Script

```bash
bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)"
```

## First-Time Setup

On first run the CLI will:

1. **Scan for models** in common directories:
   - `./models` (project directory)
   - `~/.cache/git-ca/models` (Linux)
   - `~/.local/share/git-ca/models` (Linux alt)
   - `~/Library/Application Support/git-ca/models` (macOS)

2. **Download default model** automatically if none found:
   - Downloads `unsloth/gemma-3-270m-it-GGUF` from Hugging Face
   - Stores it in `~/.cache/git-ca/models/`

3. **Prompt for confirmation** if multiple models are found:
   ```bash
   git ca model  # Interactive model selector
   ```

## Usage

```bash
git add <files>
git ca
```

For each invocation:

1. The staged diff is summarised (lockfiles and large assets are listed but not inlined).
2. The llama.cpp model generates a commit message.
3. Invalid output triggers a stricter retry; if still invalid, a deterministic fallback is offered.
4. Choose to **use**, **edit**, or **cancel** the message.

### Configuration Commands

- `git ca model` — Interactive model selector
- `git ca language` — Choose English or Simplified Chinese prompts
- `git ca doctor` — Test model loading and inference
- `git ca --version` — Display version information

## Development

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo run -- git ca      # try against staged changes
```

Key modules:
- `src/main.rs` — CLI orchestration, diff summariser, fallback generator.
- `src/llama.rs` — llama.cpp session management.

## Release Process

Releases are automated via GitHub Actions:

1. Push a version tag: `git tag v1.1.2 && git push origin v1.1.2`
2. GitHub Actions builds binaries for macOS only (2 platforms: Intel & Apple Silicon)
3. Binaries are uploaded to GitHub Releases
4. Homebrew formula is automatically updated with bottle checksums
5. `homebrew-tap` repository receives the updated formula
   - **Note**: Linux builds are temporarily disabled due to compilation issues
   - Windows builds are available via GitHub Releases

See [DEPLOY.md](DEPLOY.md) for complete release documentation.

## Contributing

Pull requests are welcome. Please include:
- `cargo fmt` / `cargo clippy -- -D warnings` / `cargo test` outputs,
- Updates to documentation (`README*.md`, `AGENTS.md`, `DEPLOY.md`) when behaviour changes,
- A short description of manual `git ca` verification if applicable.

## License

Released under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

- The Rust community for providing excellent libraries and tools
- llama.cpp team for the efficient local inference engine
