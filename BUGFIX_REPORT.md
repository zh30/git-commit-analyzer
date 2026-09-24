# Bug Fix Report: Homebrew Formula Syntax Error

## 问题分析

**用户报告的错误：**
```bash
brew install git-ca
Error: zh30/tap/git-ca: /opt/homebrew/Library/Taps/zh30/homebrew-tap/Formula/git-ca.rb:12: syntax errors found
  10 |   bottle do
  11 |     root_url "https://github.com/zh30/git-commit-analyzer/releases/download/v2.0.14"
> 12 |     sha256 cellar: :any, : ""
     |                        ^ unexpected ':'; expected an argument
```

**根本原因：**

GitHub Actions 工作流 (`.github/workflows/build-binaries.yml`) 在第272-273行生成 Homebrew formula 时：

1. **语法错误**：使用了 `cellar: :any` 而不是 `cellar: :any_skip_relocate`
2. **空值错误**：当 SHA256 变量为空时，生成的 formula 包含空字符串 `sha256 cellar: :any, arm64_sequoia: ""`
3. **缺少验证**：没有检查变量是否为空就使用

## 修复内容

### 修改的文件：`.github/workflows/build-binaries.yml`

**修改前（第272-273行）：**
```yaml
echo "    sha256 cellar: :any, arm64_sequoia: \"${ARM64_MACOS}\""
echo "    sha256 cellar: :any, x86_64_sequoia: \"${X86_64_MACOS}\""
```

**修改后（第273-279行）：**
```yaml
# Only add SHA256 lines if the values are not empty
if [ -n "${ARM64_MACOS}" ]; then
  echo "    sha256 cellar: :any_skip_relocate, arm64_sequoia: \"${ARM64_MACOS}\""
fi
if [ -n "${X86_64_MACOS}" ]; then
  echo "    sha256 cellar: :any_skip_relocate, x86_64_sequoia: \"${X86_64_MACOS}\""
fi
```

### 修复的关键点：

1. ✅ **修正语法**：从 `cellar: :any` 改为 `cellar: :any_skip_relocate`
2. ✅ **添加空值检查**：只有当变量非空时才生成对应的 SHA256 行
3. ✅ **保持向后兼容**：只影响生成流程，不影响已有的 formula

## 用户临时解决方案

在修复发布前，用户可以通过以下方式安装：

### 方案 1：从源码安装
```bash
brew install --build-from-source git-ca
```

### 方案 2：手动下载
```bash
# macOS Apple Silicon
curl -L -o git-ca.tar.gz https://github.com/zh30/git-commit-analyzer/releases/download/v2.0.12/git-ca-2.0.12-apple-darwin-arm64.tar.gz
tar -xzf git-ca.tar.gz
sudo mv git-ca /usr/local/bin/
chmod +x /usr/local/bin/git-ca
```

## 长期解决方案

修复后的工作流将在下次发布 v2.0.13+ 时生效：

1. 推送新版本标签：`git tag v2.0.13 && git push origin v2.0.13`
2. GitHub Actions 自动：
   - 构建 macOS 二进制文件
   - 生成正确的 Homebrew formula
   - 更新 homebrew-tap 仓库
3. 用户执行 `brew update && brew upgrade git-ca` 即可安装修复版本

## 验证修复

修复后的 formula 将生成正确的格式：
```ruby
class GitCa < Formula
  desc "AI-powered Git plugin for generating meaningful commit messages"
  homepage "https://github.com/zh30/git-commit-analyzer"
  url "https://github.com/zh30/git-commit-analyzer/archive/refs/tags/v2.0.13.tar.gz"
  sha256 "CORRECT_SHA256_HASH"
  license "MIT"
  head "https://github.com/zh30/git-commit-analyzer.git", branch: "main"

  bottle do
    root_url "https://github.com/zh30/git-commit-analyzer/releases/download/v2.0.13"
    sha256 cellar: :any_skip_relocate, arm64_sequoia: "ARM64_SHA256"
    sha256 cellar: :any_skip_relocate, x86_64_sequoia: "X64_SHA256"
  end
  # ...
end
```

## 相关文件

- `.github/workflows/build-binaries.yml` - 已修复
- `git-ca.rb` - 已更新版本号
- `README.md` - 已添加故障排除说明
