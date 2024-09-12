# Git 提交分析器

[English Version](README.md)

Git 提交分析器是一个强大的 Git 插件，利用 AI 技术根据您的暂存更改自动生成有意义的提交信息。它支持本地（Ollama）和远程 AI 模型来分析 git 差异并提出符合 Git Flow 格式的提交信息。

## 功能特性

- 自动生成符合 Git Flow 规范的提交信息
- 支持多个 AI 提供商（Ollama 和 Groq 和 Cerebras）
- 交互模式允许用户选择 AI 提供商，使用、编辑或取消建议的提交信息
- 跨平台兼容性（Linux、macOS、Windows）
- 可使用您的个人 Git 签名进行自定义

## 前置条件

- Git（2.0 版本或更高）
- Rust（1.54 版本或更高）
- Cargo（通常随 Rust 一起安装）
- Ollama（安装了 llama3.1 模型）用于本地 AI 处理
- Groq API 密钥（可选，用于使用 Groq 的远程 AI 模型）
- Cerebras API 密钥（可选，用于使用 Cerebras 的远程 AI 模型）
## 安装

### Linux 和 macOS

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

5. 将插件目录添加到您的 PATH。将以下行添加到您的 `~/.bashrc`、`~/.bash_profile` 或 `~/.zshrc`（取决于您的 shell）：
   ```
   export PATH="$HOME/.git-plugins:$PATH"
   ```

6. 重新加载您的 shell 配置：
   ```
   source ~/.bashrc  # 或 ~/.bash_profile, 或 ~/.zshrc
   ```

7. 如果您计划使用 Groq，请将 API 密钥设置为环境变量：
   ```
   echo 'export GROQ_API_KEY=your_groq_api_key_here' >> ~/.bashrc  # 或 ~/.bash_profile, 或 ~/.zshrc
   echo 'export CEREBRAS_API_KEY=your_cerebras_api_key_here' >> ~/.bashrc  # 或 ~/.bash_profile, 或 ~/.zshrc
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
   - 右键点击"此电脑"或"我的电脑"，选择"属性"
   - 点击"高级系统设置"
   - 点击"环境变量"
   - 在"系统变量"下，找到并选择"Path"，然后点击"编辑"
   - 点击"新建"并添加 `%USERPROFILE%\.git-plugins`
   - 点击"确定"关闭所有对话框

6. 如果您计划使用 Groq，请将 API 密钥设置为环境变量：
   - 在同一个"环境变量"对话框中，在"用户变量"下，点击"新建"
   - 将变量名设置为 `GROQ_API_KEY`，变量值设置为您的 Groq API 密钥
   - 将变量名设置为 `CEREBRAS_API_KEY`，变量值设置为您的 Cerebras API 密钥
   - 点击"确定"关闭所有对话框

7. 重启所有打开的命令提示符，使更改生效。

## 使用方法

安装完成后，您可以在任何 Git 仓库中使用 Git 提交分析器：

1. 在您的 Git 仓库中暂存更改（使用 `git add` 命令）。
2. 运行以下命令：

   ```
   git ca
   ```

3. 按照提示选择 AI 提供商（Ollama 或 Groq）。
4. 程序将分析您的暂存更改并生成建议的提交信息。
5. 您可以选择使用建议的信息、编辑它或取消提交。

## 贡献

欢迎贡献！请随时提交 Pull Request。

## 许可证

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。

## 致谢

- Rust 社区提供的优秀库和工具
- Ollama 提供 llama3.1 模型
- Groq 提供的 API 服务
