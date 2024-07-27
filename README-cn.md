# Git Commit Analyzer (git-ca)

[English Version](README.md) | [开发者文档](README-dev.md)

Git Commit Analyzer (git-ca) 是一个强大的 Git 插件，它利用 AI 技术根据你的暂存更改自动生成有意义的提交信息。无论你是编程新手还是经验丰富的开发者，这个工具都能让你的 Git 工作流程更加高效。

## 🌟 主要特性

- 🤖 自动生成符合 Git Flow 规范的提交信息
- 🔄 支持多种 AI 提供商（本地 Ollama 和远程 Groq）
- 🖥️ 交互式模式，让你可以选择 AI 提供商，使用、编辑或取消建议的提交信息
- 🌍 跨平台兼容（Linux、macOS、Windows）
- 🎨 可以使用你的个人 Git 签名进行自定义

## 📋 使用前准备

在开始之前，请确保你的系统已经安装了以下软件：

- [Git](https://git-scm.com/downloads) (版本 2.0 或更高)
- [Ollama](https://ollama.ai/download) (安装 llama3.1 模型) - 用于本地 AI 处理
- Groq API 密钥 (可选，用于使用 Groq 的远程 AI 模型)

## 🚀 安装指南

我们提供了一个方便的安装脚本，可以在 Linux、macOS 和 Windows (Git Bash) 上使用。

### Linux 和 macOS 用户

打开终端，运行以下命令：

```bash
bash <(curl -s https://scripts.zhanghe.dev/git_ca_install.sh)
```

### Windows 用户 - 待定（目前不可用）

1. 下载 [install.bat](https://scripts.zhanghe.dev/git_ca_install.bat) 文件。
2. 双击运行 `install.bat` 文件。

安装脚本会自动下载 `git-ca` 可执行文件，并将其放置在适当的位置。对于 Unix-like 系统，它还会自动添加到你的 PATH 中。

### 手动设置 Groq API 密钥（可选）

如果你计划使用 Groq 作为 AI 提供商，你需要设置 Groq API 密钥：

- **Linux 和 macOS**:
  ```bash
  echo 'export GROQ_API_KEY=你的_groq_api_密钥' >> ~/.bashrc
  source ~/.bashrc
  ```

- **Windows**:
  在命令提示符中运行：
  ```
  setx GROQ_API_KEY "你的_groq_api_密钥"
  ```

## 🎯 如何使用

安装完成后，你可以在任何 Git 仓库中使用 Git Commit Analyzer：

1. 在你的 Git 仓库中暂存更改（使用 `git add` 命令）。
2. 运行以下命令：

   ```
   git-ca
   ```

3. 按照提示选择 AI 提供商（Ollama 或 Groq）。
4. 程序会分析你的暂存更改并生成提交信息建议。
5. 你可以选择使用建议的信息、编辑它，或取消提交。

## 🤝 贡献

我们欢迎任何形式的贡献！如果你有好的想法或发现了 bug，请随时提交 Pull Request 或创建 Issue。

## 📄 许可证

本项目采用 MIT 许可证 - 详情请查看 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- Rust 社区提供的优秀库和工具
- Ollama 提供的 llama3.1 模型
- Groq 提供的 API 服务
