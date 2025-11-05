# Git 提交分析器

[English](README.md) · [Français](README_FR.md) · [Español](README_ES.md)

Git 提交分析器是一个基于 Rust 的 Git 插件，利用本地 llama.cpp 模型分析已暂存的 diff，并生成符合 Git Flow 规范的提交说明。CLI 会在提示前压缩冗长 diff，校验模型输出格式，并在必要时提供确定性的兜底提交信息。

## 功能特性

- **本地推理**：通过 `llama_cpp_sys_2` 调用 GGUF 模型，无需远程 API。
- **智能 diff 摘要**：锁文件、生成物等大文件仅展示概要，避免浪费 Token。
- **Git Flow 校验**：严格要求 `<type>(<scope>): <subject>`，失败时自动重试或兜底。
- **交互式 CLI**：支持直接使用、编辑或取消生成的提交说明。
- **多语言提示**：提供英文（默认）和简体中文两种提示语言。
- **多平台支持**：macOS 预构建二进制包（Intel + Apple Silicon）。

## 环境要求

- Git ≥ 2.30
- 一个本地 GGUF 模型（CLI 可自动下载默认模型 `unsloth/gemma-3-270m-it-GGUF`）

## 安装方式

### Homebrew（推荐） - 快速二进制安装

**macOS 用户可通过 Homebrew 安装预构建二进制包（无需 Rust 编译）：**

```bash
brew tap zh30/tap
brew install git-ca
```

这将为您的平台安装预构建的二进制包：
- **macOS**：Apple Silicon (M1/M2/M3/M4) 和 Intel (x86_64)

**无需 Rust 工具链或编译！** 二进制文件会自动从 GitHub Releases 下载。

**注意**：Linux 构建因编译问题暂时禁用。Windows 构建可通过 [GitHub Releases](https://github.com/zh30/git-commit-analyzer/releases) 获取，但不通过 Homebrew 分发。

### 手动安装

从 [Releases](https://github.com/zh30/git-commit-analyzer/releases) 下载对应平台的二进制文件：

```bash
# macOS (Apple Silicon)
curl -L -o git-ca.tar.gz https://github.com/zh30/git-commit-analyzer/releases/download/v1.1.2/git-ca-1.1.2-apple-darwin-arm64.tar.gz
tar -xzf git-ca.tar.gz
sudo mv git-ca /usr/local/bin/
chmod +x /usr/local/bin/git-ca
```

**注意**：Linux 构建暂时禁用。Windows 构建可通过 [GitHub Releases](https://github.com/zh30/git-commit-analyzer/releases) 获取。

### 源码构建

如果您偏好从源码构建：

```bash
git clone https://github.com/zh30/git-commit-analyzer.git
cd git-commit-analyzer
cargo build --release
sudo cp target/release/git-ca /usr/local/bin/
```

### 一键脚本

```bash
bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)"
```

## 首次配置

首次运行 CLI 会执行以下步骤：

1. **扫描模型** 在常用目录：
   - `./models`（项目目录）
   - `~/.cache/git-ca/models`（Linux/macOS）
   - `~/.local/share/git-ca/models`（Linux 备用）
   - `~/Library/Application Support/git-ca/models`（macOS）

2. **自动下载默认模型**（如果未找到）：
   - 从 Hugging Face 下载 `unsloth/gemma-3-270m-it-GGUF`
   - 存储至 `~/.cache/git-ca/models/`

3. **交互式选择**（如果找到多个模型）：
   ```bash
   git ca model  # 交互式模型选择器
   ```

## 使用说明

```bash
git add <files>
git ca
```

每次调用的流程：

1. 对已暂存 diff 进行摘要（大型文件仅展示概要）。
2. llama.cpp 模型生成提交说明。
3. 若输出不符合规范，使用更严格提示重试；仍失败则给出兜底信息。
4. 交互式选择 **使用**、**编辑** 或 **取消**。

### 配置命令

- `git ca model` — 交互式模型选择器
- `git ca language` — 选择英文或简体中文提示
- `git ca doctor` — 测试模型加载和推理
- `git ca --version` — 显示版本信息

## 开发指引

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo run -- git ca  # 测试暂存的更改
```

核心代码：
- `src/main.rs` — CLI 主流程、diff 摘要、兜底提交逻辑
- `src/llama.rs` — llama.cpp 会话管理

## 发布流程

**完全自动化的 GitHub Actions 发布流程：**

1. 推送版本标签：`git tag v1.1.2 && git push origin v1.1.2`
2. GitHub Actions 自动执行：
   - 为 macOS 构建二进制包（Intel + Apple Silicon）
   - 创建带变更日志的 GitHub Release
   - 生成 SHA256 校验和
   - **自动更新 Homebrew formula** 并添加 bottle 校验和
   - 推送更新到 `homebrew-tap` 仓库
3. 用户可立即安装：`brew install git-ca`

**注意**：Linux 构建因编译问题暂时禁用。Windows 构建可通过 GitHub Releases 获取，但不通过 Homebrew 分发。

详见 [DEPLOY.md](DEPLOY.md) 获取完整发布文档。

## 支持平台

- **macOS**：✅ Apple Silicon (arm64) 和 Intel (x86_64) - 通过 Homebrew 提供预构建二进制包
- **Linux**：❌ 暂时禁用（编译问题）
- **Windows**：⚠️ 通过 GitHub Releases 提供（不通过 Homebrew）

## 贡献

欢迎提交 Pull Request！请在提交前完成：
- `cargo fmt` / `cargo clippy -- -D warnings` / `cargo test` 检查，
- 在行为变化时更新相关文档（`README*.md`、`AGENTS.md`、`DEPLOY.md`），
- 如适用，提供手动 `git ca` 验证的简短说明。

## 许可证

项目采用 MIT 许可证，详见 [LICENSE](LICENSE)。

## 致谢

- Rust 社区提供的优秀库和工具
- llama.cpp 团队的高效本地推理引擎
