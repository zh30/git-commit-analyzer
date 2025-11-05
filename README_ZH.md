# Git 提交分析器

[English](README.md) · [Français](README_FR.md) · [Español](README_ES.md)

Git 提交分析器是一个基于 Rust 的 Git 插件，利用本地 llama.cpp 模型分析已暂存的 diff，并生成符合 Git Flow 规范的提交说明。CLI 会在提示前压缩冗长 diff，校验模型输出格式，并在必要时提供确定性的兜底提交信息。

## 功能特性

- **本地推理**：通过 `llama_cpp_sys` 调用 GGUF 模型，无需远程 API。
- **智能 diff 摘要**：锁文件、生成物等大文件仅展示概要，避免浪费 Token。
- **Git Flow 校验**：严格要求 `<type>(<scope>): <subject>`，失败时自动重试或兜底。
- **交互式 CLI**：支持直接使用、编辑或取消生成的提交说明。
- **多语言提示**：提供英文（默认）和简体中文两种提示语言。
- **上下文可调**：可通过 Git 配置调整 llama 上下文长度。

## 环境要求

- Git ≥ 2.30
- Rust 稳定版工具链
- 构建 llama.cpp 所需依赖（cmake、C/C++ 编译器、Metal/CUDA 等）
- 本地 GGUF 模型（首次运行可自动下载 `unsloth/gemma-3-270m-it-GGUF`）

## 安装方式

### 手动安装

```bash
git clone https://github.com/zh30/git-commit-analyzer.git
cd git-commit-analyzer
cargo build --release
mkdir -p ~/.git-plugins
cp target/release/git-ca ~/.git-plugins/
echo 'export PATH="$HOME/.git-plugins:$PATH"' >> ~/.bashrc   # 根据使用的 shell 调整
source ~/.bashrc
```

CLI 会在常用目录（`./models`、`~/Library/Application Support/git-ca/models`、`~/.cache/git-ca/models`）中查找 GGUF 模型，若未找到会提示自动下载默认模型。

### Homebrew（macOS / Linux）

```bash
brew tap zh30/tap
brew install git-ca
```

### 一键脚本

可选的 `install-git-ca.sh` 会检测依赖、编译二进制并更新 PATH：

```bash
bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)"
```

运行前建议审阅脚本，并确认本地可访问目标 GGUF 模型。

## 使用说明

```bash
git add <files>
git ca
```

首次运行需选择模型路径。后续流程包括：

1. 对已暂存 diff 进行摘要（大型文件仅展示概要）。
2. llama.cpp 模型生成提交说明。
3. 若输出不符合规范，使用更严格提示重试；仍失败则给出兜底信息（如 `chore(deps): update dependencies`）。
4. 交互式选择 **使用**、**编辑** 或 **取消**。

### 配置命令

- `git ca model` — 交互式选择模型路径，所选 GGUF 会在后续运行中复用。
- 非交互模式优先使用已保存的模型，否则自动使用检测到的第一个 GGUF。
- `git ca language` — 切换提示语言（英文/中文），写入 `commit-analyzer.language`。
- llama 上下文长度固定为 1024 tokens。

## 开发指引

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo run -- git ca
```

核心代码：
- `src/main.rs` — CLI 主流程、diff 摘要、兜底提交逻辑。
- `src/llama.rs` — llama.cpp 会话封装。

## 贡献

欢迎提交 Pull Request！请在提交前完成 `cargo fmt` / `cargo clippy -- -D warnings` / `cargo test`，并在行为变化时更新相关文档（`README*.md`、`AGENTS.md`、`DEPLOY.md`）。

## 许可证

项目采用 MIT 许可证，详见 [LICENSE](LICENSE)。
