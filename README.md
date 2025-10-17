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

## Requirements

- Git 2.30+
- Rust toolchain (stable) with `cargo`
- Build prerequisites for llama.cpp (`cmake`, C/C++ compiler, Metal/CUDA drivers as appropriate)
- A local GGUF model (the CLI can download the default `unsloth/gemma-3-270m-it-GGUF`)

## Installation

### Manual Install

```bash
git clone https://github.com/zh30/git-commit-analyzer.git
cd git-commit-analyzer
cargo build --release
mkdir -p ~/.git-plugins
cp target/release/git-ca ~/.git-plugins/
echo 'export PATH="$HOME/.git-plugins:$PATH"' >> ~/.bashrc   # adapt for your shell
source ~/.bashrc
```

On first run the CLI scans common model directories (`./models`, `~/Library/Application Support/git-ca/models`, `~/.cache/git-ca/models`) and can download the default model via Hugging Face if none are found.

### Homebrew Tap (macOS/Linux)

```bash
brew tap zh30/tap
brew install git-ca
```

### One-Line Bootstrap Script

An optional helper script (`install-git-ca.sh`) automates dependency checks, compilation, and PATH updates:

```bash
bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)"
```

Review the script before execution and ensure a GGUF model is available (or allow the automated download).

## Usage

```bash
git add <files>
git ca
```

During the first run you will be asked to choose a model path. For each invocation:

1. The staged diff is summarised (lockfiles and large assets are listed but not inlined).
2. The llama.cpp model generates a commit message.
3. Invalid output triggers a stricter retry; if still invalid, a deterministic fallback (e.g., `chore(deps): update dependencies`) is offered.
4. Choose to **use**, **edit**, or **cancel** the message.

### Configuration

- `git ca model` — interactive model selector; the chosen GGUF path is reused on future runs.
- Non-interactive runs reuse the persisted model or fall back to the first detected GGUF.
- `git ca language` — choose English or Simplified Chinese prompts; stored in `commit-analyzer.language`.
- Llama context length is fixed to 1024 tokens.

## Development

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo run -- git ca      # try against staged changes
```

Key modules:
- `src/main.rs` — CLI orchestration, diff summariser, fallback generator.
- `src/llama.rs` — thin wrapper around llama.cpp session management.

## Contributing

Pull requests are welcome. Please include:
- `cargo fmt` / `cargo clippy -- -D warnings` / `cargo test` outputs,
- Updates to documentation (`README*.md`, `AGENTS.md`, `DEPLOY.md`) when behaviour changes,
- A short description of manual `git ca` verification if applicable.

## License

Released under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

- The Rust community for providing excellent libraries and tools
- Ollama for providing local AI model support
