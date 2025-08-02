# Git 提交分析器

[English](README.md)

Git 提交分析器是一个强大的 Git 插件，它利用人工智能根据您的暂存更改自动生成有意义的提交消息。它使用 Ollama 分析 git 差异并提出符合 Git Flow 格式的提交消息。

## 功能特点

- 自动生成符合 Git Flow 规范的提交消息
- 由 Ollama 提供支持，实现本地 AI 处理
- 交互模式允许用户使用、编辑或取消建议的提交消息
- 多语言支持（英文和简体中文）
- 跨平台兼容性（Linux、macOS、Windows）
- 可以使用您的个人 Git 签名进行自定义
- 支持模型选择和持久化

## 前提条件

- Git（2.0 或更高版本）
- 已安装并运行 Ollama（https://ollama.com/download）
- Ollama 中至少安装了一个语言模型

## 安装

### Homebrew（macOS 和 Linux）

安装 Git 提交分析器最简单的方法是通过 Homebrew：

```
brew tap zh30/tap
brew install git-ca
```

安装后，您可以立即使用 `git ca` 命令。

### 手动安装（Linux 和 macOS）

1. 克隆仓库：
   ```
   git clone https://github.com/zh30/git-commit-analyzer.git
   cd git-commit-analyzer
   ```

2. 构建项目：
   ```
   cargo build --release
   ```

3. 创建 Git 插件目录（如果不存在）：
   ```
   mkdir -p ~/.git-plugins
   ```

4. 将编译好的二进制文件复制到插件目录：
   ```
   cp target/release/git-ca ~/.git-plugins/
   ```

5. 将插件目录添加到您的 PATH。根据您使用的 shell，将以下行添加到 `~/.bashrc`、`~/.bash_profile` 或 `~/.zshrc`：
   ```
   export PATH="$HOME/.git-plugins:$PATH"
   ```

6. 重新加载您的 shell 配置：
   ```
   source ~/.bashrc  # 或 ~/.bash_profile, 或 ~/.zshrc
   ```

### Windows - 理论上可行

1. 克隆仓库：
   ```
   git clone https://github.com/zh30/git-commit-analyzer.git
   cd git-commit-analyzer
   ```

2. 构建项目：
   ```
   cargo build --release
   ```

3. 创建 Git 插件目录（如果不存在）：
   ```
   mkdir %USERPROFILE%\.git-plugins
   ```

4. 将编译好的二进制文件复制到插件目录：
   ```
   copy target\release\git-commit-analyzer.exe %USERPROFILE%\.git-plugins\
   ```

5. 将插件目录添加到您的 PATH：
   - 右键点击"此电脑"或"我的电脑"并选择"属性"
   - 点击"高级系统设置"
   - 点击"环境变量"
   - 在"系统变量"下，找到并选择"Path"，然后点击"编辑"
   - 点击"新建"并添加 `%USERPROFILE%\.git-plugins`
   - 点击"确定"关闭所有对话框

6. 重启所有打开的命令提示符，使更改生效。

## 使用方法

安装后，您可以在任何 Git 仓库中使用 Git 提交分析器：

1. 在您的 Git 仓库中暂存您的更改（使用 `git add` 命令）。
2. 运行以下命令：

   ```
   git ca
   ```

3. 如果是首次运行该命令，系统会提示您从已安装的 Ollama 模型中选择一个模型。
4. 程序将分析您的暂存更改并生成建议的提交消息。
5. 您可以选择使用建议的消息、编辑它或取消提交。

### 配置命令

要随时更改默认模型，请运行：

```
git ca model
```

要设置 AI 生成提交消息的输出语言，请运行：

```
git ca language
```

可用语言：
- 英文（默认）
- 简体中文

所选语言将决定 AI 模型生成的提交消息的语言。注意：这会影响 AI 的提示语言，而不是界面语言。

## 贡献

欢迎贡献！请随时提交拉取请求。

## 许可证

该项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。

## 致谢

- Rust 社区提供了优秀的库和工具
- Ollama 提供本地 AI 模型支持
