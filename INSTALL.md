# Installation Guide

Git Commit Analyzer ships as a single Rust binary (`git-ca`) that integrates with Git as an external command. Choose the installation path that best fits your environment.

## 1. Requirements
- Git 2.30 or later
- Rust toolchain (stable channel) with `cargo`
- Build prerequisites for llama.cpp (`cmake`, `make`, C/C++ compiler, GPU drivers as needed)
- A local GGUF model (the CLI can auto-download `marzoukbaig14/committed-gguf-0.6b` from Hugging Face)

## 2. Manual Installation

```bash
git clone https://github.com/zh30/git-commit-analyzer.git
cd git-commit-analyzer
cargo build --release
mkdir -p ~/.git-plugins
cp target/release/git-ca ~/.git-plugins/
echo 'export PATH="$HOME/.git-plugins:$PATH"' >> ~/.bashrc   # adapt to your shell
source ~/.bashrc
```

### Windows Notes
1. `cargo build --release`
2. Copy `target\release\git-ca.exe` to `%USERPROFILE%\.git-plugins\`
3. Add `%USERPROFILE%\.git-plugins` to the user PATH via *System Properties → Environment Variables*

## 3. Homebrew (macOS / Linux)

```bash
brew tap zh30/tap
brew install git-ca
```

## 4. Bootstrap Script (Optional)

The repository includes `install-git-ca.sh`, which:
- Detects the platform
- Installs Git/Rust if missing
- Builds the release binary
- Adds `~/.git-plugins` to PATH

```bash
bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)"
```

Read the script before executing and ensure you are comfortable with the actions it performs.

## 5. First Run

```bash
git add <files>
git ca
```

On initial launch the CLI probes system memory, recommends a model tier (`small` / `default` / `quality`), scans common directories for GGUF models, and downloads a Q4 model if none are found.

### Additional configuration

- `git ca model` — interactive tier + local model selector (persisted for future runs)
- `git ca model pull [small|default|quality|<repo>]` — download a tier or custom HF GGUF
- Non-interactive runs reuse the saved model or fall back to the first detected GGUF.
- Context length is adaptive (typically 4K–16K); override with `commit-analyzer.context`

## 6. Troubleshooting

### Model not found
- Ensure at least one GGUF file exists in the default search directories.
- Confirm the GGUF file is readable.
- Run `git ca model` to select the file interactively.

### Build failures
- Check that `cmake`, `make`, and a C/C++ compiler are available (`cmake --version`, `cc --version`).
- On macOS install Xcode Command Line Tools (`xcode-select --install`).
- On Linux install build essentials (`apt install build-essential cmake` or distro equivalent).

### llama.cpp context / memory errors
- Prefer a smaller tier (`git ca model pull small`) on low-RAM machines.
- Lower context via `git config commit-analyzer.context 4096`.
- Verify available GPU/CPU memory; larger tiers need more headroom.

### Command not found
- Ensure `~/.git-plugins` (or chosen directory) is in PATH.
- Reload your shell (`source ~/.bashrc`, `source ~/.zshrc`) or open a new terminal.

## 7. Uninstall

```bash
rm -f ~/.git-plugins/git-ca
sed -i '' '/git-plugins/d' ~/.bashrc   # adjust for your shell/OS
git config --global --unset commit-analyzer.language 2>/dev/null
```

## 8. Support
- Issues: <https://github.com/zh30/git-commit-analyzer/issues>
- Default model: <https://huggingface.co/marzoukbaig14/committed-gguf-0.6b> (optional Qwen3 tiers: 0.6B / 1.7B / 4B)
- llama.cpp documentation: <https://github.com/ggerganov/llama.cpp>
