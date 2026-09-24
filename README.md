# Git Commit Analyzer

[中文](README_ZH.md) · [Français](README_FR.md) · [Español](README_ES.md)

- Official website: https://zhanghe.dev/products/git-commit-analyzer
- Releases: https://github.com/zh30/git-commit-analyzer/releases

Git Commit Analyzer is a Rust-based Git plugin that generates Conventional
Commits messages from your staged diff using a local llama.cpp model. The CLI
summarises large diffs, validates model output, and falls back to deterministic
messages when needed.

## Key Features

- **Local inference**: Uses `llama_cpp_sys_2` to run GGUF models without any remote API calls.
- **Task-tuned default model**: Auto-downloads `committed-0.6b` (a Qwen3-0.6B fine-tune trained for Conventional Commits) on first run; official Qwen3 tiers stay available via `git ca model pull`.
- **Adaptive context**: Sizes the llama.cpp context from system memory (4K–16K tokens).
- **Hierarchical diff packing**: Multi-file commits keep an inventory + key signatures before filling leftover budget with snippets.
- **Conventional Commits validation**: Grammar-constrained decoding keeps output at `<type>(<scope>): <subject>`, with retry/fallback when needed.
- **Interactive CLI**: Review, edit, or cancel the generated commit message.
- **Multi-platform support**: Pre-built binaries for macOS (Intel & Apple Silicon).

## Requirements

- Git 2.30+
- A local GGUF model (the CLI can auto-download `marzoukbaig14/committed-gguf-0.6b` from Hugging Face)

## Installation

### Homebrew (Recommended) - Fast Binary Installation

**macOS users can install via Homebrew with pre-built binaries (no Rust compilation required):**

```bash
brew tap zh30/tap
brew install git-ca
```

This installs a pre-built binary for your platform:
- **macOS**: Apple Silicon (M1/M2/M3/M4) and Intel (x86_64)

**No Rust toolchain or compilation needed!** The binary is automatically downloaded from GitHub Releases.

**Note**: If you encounter a version mismatch error, try:
```bash
brew update
brew upgrade git-ca
```

Or install from source:
```bash
brew install --build-from-source git-ca
```

Linux builds are temporarily disabled due to compilation issues. Windows builds are available via [GitHub Releases](https://github.com/zh30/git-commit-analyzer/releases) but not distributed via Homebrew.

### Manual Installation

Download the appropriate binary for your platform from [Releases](https://github.com/zh30/git-commit-analyzer/releases):

```bash
# macOS (Apple Silicon)
curl -L -o git-ca.tar.gz https://github.com/zh30/git-commit-analyzer/releases/download/v2.0.12/git-ca-2.0.12-apple-darwin-arm64.tar.gz
tar -xzf git-ca.tar.gz
sudo mv git-ca /usr/local/bin/
chmod +x /usr/local/bin/git-ca
```

**Note**: Linux builds are temporarily disabled. Windows builds are available via [GitHub Releases](https://github.com/zh30/git-commit-analyzer/releases).

### Build from Source

If you prefer to build from source:

```bash
git clone https://github.com/zh30/git-commit-analyzer.git
cd git-commit-analyzer
cargo build --release
sudo cp target/release/git-ca /usr/local/bin/
```

### One-Line Bootstrap Script

```bash
bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)"
```

## First-Time Setup

On first run the CLI will:

1. **Probe system memory** to size the context window (4K–16K tokens).

2. **Scan for models** in common directories:
   - `./models` (project directory)
   - `~/.cache/git-ca/models` (Linux/macOS)
   - `~/.local/share/git-ca/models` (Linux alt)
   - `~/Library/Application Support/git-ca/models` (macOS)

3. **Download the default model** automatically if none found: `marzoukbaig14/committed-gguf-0.6b` (Q4_K_M GGUF, ~397 MB, a Qwen3-0.6B fine-tune for Conventional Commits) into `~/.cache/git-ca/models/`.

4. **Prompt interactively** when multiple models/tiers are available:
   ```bash
   git ca model              # Interactive selector (tiers + local GGUFs)
   git ca model pull         # Download the recommended Qwen3 tier
   git ca model pull quality # Force a specific tier (small|default|quality)
   git ca model pull <repo>  # Pull a custom HF GGUF repo
   ```

## Usage

```bash
git add <files>
git ca
```

For each invocation:

1. The staged diff is packed hierarchically (file inventory → key signatures → extra snippets) to fit the adaptive context window.
2. The llama.cpp model generates a commit message.
3. Invalid output triggers a stricter retry with a tighter diff view; if still invalid, a deterministic fallback is offered.
4. Choose to **use**, **edit**, or **cancel** the message.

### Configuration Commands

- `git ca model` — Interactive model / tier selector
- `git ca model pull [small|default|quality|<repo>]` — Download a tier or custom HF GGUF repo
- `git ca doctor` — Hardware profile + model loading smoke test
- `git ca --version` — Display version information

Optional git config overrides:

```bash
git config --global commit-analyzer.model-tier default   # small | default | quality
git config --global commit-analyzer.context 8192         # token context length
```

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

**Fully automated release via GitHub Actions:**

1. Push a version tag: `git tag v1.1.2 && git push origin v1.1.2`
2. GitHub Actions automatically:
   - Builds binaries for macOS (Intel & Apple Silicon)
   - Creates GitHub Release with changelog
   - Generates SHA256 checksums
   - **Automatically updates Homebrew formula** with bottle checksums
   - Pushes updates to `homebrew-tap` repository
3. Users can immediately install with: `brew install git-ca`

**Note**: Linux builds are temporarily disabled due to compilation issues. Windows builds are available via GitHub Releases but not distributed via Homebrew.

See [DEPLOY.md](DEPLOY.md) for complete release documentation.

## Supported Platforms

- **macOS**: ✅ Apple Silicon (arm64) and Intel (x86_64) - Pre-built binaries via Homebrew
- **Linux**: ❌ Temporarily disabled (compilation issues)
- **Windows**: ⚠️ Available via GitHub Releases (not Homebrew)

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
