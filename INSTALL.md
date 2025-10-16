# Installation Guide

Git Commit Analyzer ships as a single Rust binary (`git-ca`) that integrates with Git as an external command. Choose the installation path that best fits your environment.

## 1. Requirements
- Git 2.30 or later
- Rust toolchain (stable channel) with `cargo`
- Build prerequisites for llama.cpp (`cmake`, `make`, C/C++ compiler, GPU drivers as needed)
- A local GGUF model (the CLI can download `unsloth/gemma-3-270m-it-GGUF` automatically)

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

On initial launch the CLI scans common directories (`./models`, `~/Library/Application Support/git-ca/models`, `~/.cache/git-ca/models`) for GGUF models. If none are found it can download the default model from Hugging Face and store the absolute path in:
- `commit-analyzer.model`

### Additional configuration

- `git ca model` — interactive model selector
- `git ca language` — choose English or Simplified Chinese prompts
- `git config --global commit-analyzer.context 1024` — adjust llama context window (512–8192)

## 6. Troubleshooting

### Model not found
- Verify the path returned by `git config commit-analyzer.model`.
- Ensure the GGUF file exists and is readable.
- Run `git ca model` to reselect the file.

### Build failures
- Check that `cmake`, `make`, and a C/C++ compiler are available (`cmake --version`, `cc --version`).
- On macOS install Xcode Command Line Tools (`xcode-select --install`).
- On Linux install build essentials (`apt install build-essential cmake` or distro equivalent).

### llama.cpp context errors
- Reduce context size: `git config --global commit-analyzer.context 768`.
- Verify available GPU/CPU memory; large models may exceed device limits.

### Command not found
- Ensure `~/.git-plugins` (or chosen directory) is in PATH.
- Reload your shell (`source ~/.bashrc`, `source ~/.zshrc`) or open a new terminal.

## 7. Uninstall

```bash
rm -f ~/.git-plugins/git-ca
sed -i '' '/git-plugins/d' ~/.bashrc   # adjust for your shell/OS
git config --global --unset commit-analyzer.model 2>/dev/null
git config --global --unset commit-analyzer.language 2>/dev/null
git config --global --unset commit-analyzer.context 2>/dev/null
```

## 8. Support
- Issues: <https://github.com/zh30/git-commit-analyzer/issues>
- Default model: <https://huggingface.co/unsloth/gemma-3-270m-it-GGUF>
- llama.cpp documentation: <https://github.com/ggerganov/llama.cpp>
