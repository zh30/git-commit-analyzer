# Homebrew Installation Fix

## 问题说明

你遇到的错误是因为 Homebrew formula (`git-ca.rb`) 中有语法错误：
```
sha256 cellar: :any, : ""
```

这是一个不正确的语法格式，正确的应该是：
```ruby
sha256 cellar: :any_skip_relocate, "HASH_VALUE"
```

## 解决方案

### 方案 1：从源码安装（推荐，立即可用）

由于 Homebrew bottles 需要从 GitHub release 获取正确的哈希值，最快的解决方案是从源码构建：

```bash
brew install --build-from-source git-ca
```

这会使用本地已修复的 formula 编译安装。

### 方案 2：等待 Homebrew 自动更新

GitHub Actions 工作流会自动更新 homebrew-tap 仓库。几小时后 Homebrew 就会使用修复后的 formula。

### 方案 3：手动下载二进制文件

如果你不想编译，可以直接下载预编译的二进制文件：

```bash
# macOS (Apple Silicon - M1/M2/M3/M4)
curl -L -o git-ca.tar.gz https://github.com/zh30/git-commit-analyzer/releases/download/v2.0.12/git-ca-2.0.12-apple-darwin-arm64.tar.gz
tar -xzf git-ca.tar.gz
sudo mv git-ca /usr/local/bin/
chmod +x /usr/local/bin/git-ca
```

### 验证安装

安装完成后，验证版本：
```bash
git-ca --version
```

## 修复内容

已修复的问题：
1. ✅ 更新版本号从 v2.0.9 到 v2.0.12
2. ✅ 修正了 bottle `sha256` 行的语法错误
3. ✅ 更新了源码 tarball 的 SHA256 校验和

## 备注

- bottle 的预编译二进制哈希值需要从 GitHub release 中获取
- 如果你需要预编译的 binary，使用方案 3
- 从源码构建只需要几分钟时间
