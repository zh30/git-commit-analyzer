# Homebrew 发布指南

本文档记录了将 `git-ca` 发布到 Homebrew tap 的完整流程，支持多平台预构建二进制包（bottles）。

## 多平台预构建二进制包（推荐）

我们的 Homebrew formula 支持预构建的二进制包（bottles），用户无需从源码构建。

### 支持的平台
- **macOS**: Apple Silicon (arm64) 和 Intel (x86_64)
- **Linux**: x86_64 和 ARM64

### 发布流程

#### 自动发布（推荐）

发布流程通过 GitHub Actions 自动化完成：

1. **触发构建**：
   - 推送版本标签 `v*.*.*` 到 `main` 分支
   - GitHub Actions 会自动触发 `build-binaries.yml` 工作流

2. **构建阶段**：
   - 在多个平台上并行构建二进制包：
     - macOS 13 (x86_64)
     - macOS 14 (ARM64)
     - Ubuntu 22.04 (x86_64, ARM64)
     - Windows 2022 (x86_64, ARM64)
   - 构建完成后自动上传二进制包到 GitHub Release

3. **更新 Homebrew**：
   - `release.yml` 工作流自动：
     - 下载所有平台的二进制包
     - 计算 SHA256 校验和
     - 更新 `git-ca.rb` 公式中的 bottle 校验和
     - 推送到 `homebrew-tap` 仓库

4. **手动触发**（如需要）：
   ```bash
   # 更新版本号
   vim Cargo.toml

   # 提交并推送
   git commit -m "chore: bump version"
   git push origin main

   # 创建并推送标签
   git tag v1.1.2
   git push origin v1.1.2
   ```

#### 验证发布

在创建 PR 或推送标签前，验证 Homebrew 公式：

```bash
# 本地验证
brew install --build-from-source ./git-ca.rb
brew test git-ca
brew audit --strict ./git-ca.rb

# 验证 bottle 安装
brew uninstall git-ca
brew install zh30/tap/git-ca
git ca --version
```

### Homebrew Formula 结构

`git-ca.rb` 现在包含：

```ruby
class GitCa < Formula
  # ... 元数据 ...

  # Bottle 支持 - 预构建二进制包
  bottle do
    root_url "https://github.com/zh30/git-commit-analyzer/releases/download/v#{version}"
    sha256 cellar: :any_skip_relocate, arm64_sequoia: "SHA256_ARM64_MACOS"
    sha256 cellar: :any_skip_relocate, x86_64_sequoia: "SHA256_X86_64_MACOS"
    sha256 cellar: :any_skip_relocate, arm64_linux: "SHA256_ARM64_LINUX"
    sha256 cellar: :any_skip_relocate, x86_64_linux: "SHA256_X86_64_LINUX"
  end

  # 安装时直接使用预构建二进制
  def install
    bin.install "git-ca"
  end
end
```

### 用户安装

用户现在可以通过以下方式安装：

```bash
# 添加 tap
brew tap zh30/tap

# 安装（自动使用 bottle，无须从源码构建）
brew install git-ca

# 验证安装
git ca --version
```

## 故障排除

### 常见问题

1. **bottle 校验和不匹配**：
   - 检查二进制包是否正确构建
   - 重新计算 SHA256 校验和
   - 确保所有平台都已构建

2. **构建失败**：
   - 检查 `.github/workflows/build-binaries.yml` 中的依赖安装
   - 确认 Rust 工具链版本
   - 查看 GitHub Actions 日志

3. **Homebrew 安装慢**：
   - 检查 bottle URL 是否可访问
   - 确认 GitHub Release 已创建
   - 验证 `git-ca.rb` 中的 `root_url`

### 调试步骤

```bash
# 检查 bottle 是否可用
brew fetch --bottle-tag=arm64_sequoia zh30/tap/git-ca

# 强制从源码安装（用于调试）
HOMEBREW_NO_INSTALL_FROM_API=1 brew install --build-from-source zh30/tap/git-ca

# 查看详细安装日志
brew install -v zh30/tap/git-ca
```

## 最佳实践

1. **版本管理**：
   - 始终在 `Cargo.toml` 和 `git-ca.rb` 中保持版本一致
   - 使用语义化版本号 (semver)

2. **测试**：
   - 在不同平台上测试 bottle
   - 运行完整的 CI/CD 流程
   - 验证用户安装体验

3. **文档**：
   - 更新 README.md 中的安装说明
   - 保持 HOMEBREW.md 和 DEPLOY.md 最新
   - 记录所有依赖变更
