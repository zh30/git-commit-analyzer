# Git Commit Analyzer 一键安装指南

## 快速安装

### 方法一：使用网络安装脚本（推荐）

将 `install-git-ca.sh` 上传到你的 CDN 服务器，然后用户可以使用以下命令安装：

```bash
bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)"
```

### 方法二：直接下载运行

```bash
curl -fsSL https://your-cdn-url.com/install-git-ca.sh -o install-git-ca.sh
chmod +x install-git-ca.sh
./install-git-ca.sh
```

## 系统要求

- **操作系统**: macOS, Linux (Debian/Ubuntu, Fedora/CentOS, Arch, openSUSE)
- **依赖**: Git, Rust, Ollama
- **内存**: 至少 1GB 可用内存
- **网络**: 需要网络连接下载依赖和项目代码

## 安装过程

脚本会自动执行以下步骤：

1. **系统检测**: 自动识别操作系统类型
2. **依赖安装**: 
   - macOS: 使用 Homebrew 安装 Git 和 Rust
   - Linux: 使用系统包管理器安装依赖
3. **Ollama 配置**: 检查并配置 Ollama 环境
4. **项目构建**: 下载源码并编译发布版本
5. **环境设置**: 配置 PATH 环境变量
6. **Git 配置**: 设置用户信息（如需要）
7. **验证安装**: 确保所有组件正常工作

## 使用方法

安装完成后，在任意 Git 仓库中：

```bash
# 1. 添加文件到暂存区
git add .

# 2. 生成提交信息
git ca

# 3. 根据提示选择使用、编辑或取消提交信息
```

## 配置选项

```bash
# 选择默认 Ollama 模型
git ca model

# 设置输出语言（英文/中文）
git ca language

# 查看版本
git ca --version
```

## 故障排除

### Ollama 相关问题

如果脚本提示 Ollama 未安装或未运行：

1. **安装 Ollama**:
   ```bash
   # macOS
   brew install ollama
   
   # Linux
   curl -fsSL https://ollama.com/install.sh | sh
   ```

2. **启动 Ollama 服务**:
   ```bash
   ollama serve
   ```

3. **下载模型**（可选）:
   ```bash
   ollama pull llama3.2
   ollama pull qwen2.5:7b
   ```

### 环境变量问题

如果 `git ca` 命令不可用：

1. **重新加载 shell**:
   ```bash
   # Bash
   source ~/.bashrc
   
   # Zsh
   source ~/.zshrc
   ```

2. **或重启终端**

3. **手动添加 PATH**:
   ```bash
   export PATH="$HOME/.git-plugins:$PATH"
   ```

### 权限问题

如果遇到权限错误：

```bash
# 确保脚本有执行权限
chmod +x install-git-ca.sh

# 如果需要，手动创建插件目录
mkdir -p ~/.git-plugins
```

### 编译问题

如果 Rust 编译失败：

1. **确保 Rust 已正确安装**:
   ```bash
   rustc --version
   cargo --version
   ```

2. **重新安装 Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
   source ~/.cargo/env
   ```

## 卸载

如需卸载 Git Commit Analyzer：

```bash
# 删除二进制文件
rm -f ~/.git-plugins/git-ca

# 从 shell 配置中移除 PATH 设置
# 编辑 ~/.bashrc, ~/.zshrc 等文件，删除相关行
```

## 技术支持

- **项目地址**: https://github.com/zh30/git-commit-analyzer
- **问题报告**: https://github.com/zh30/git-commit-analyzer/issues
- **Ollama 文档**: https://ollama.com

## 安全说明

- 脚本仅从官方 GitHub 仓库下载源码
- 所有下载都使用 HTTPS 加密连接
- 脚本不会收集或传输任何个人信息
- 建议在安装前检查脚本内容