# 完全自动化 Homebrew 安装设置

本文档说明如何实现完全自动化，让用户可以通过 `brew install git-ca` 安装（注意：是 formula，不是 cask）。

## 重要说明：Formula vs Cask

- **git-ca 是命令行工具**，因此使用 `brew install git-ca`（Formula）
- **Cask** 用于 GUI 应用程序，例如：`brew install --cask firefox`
- **Formula** 用于命令行工具和库，例如：`brew install git-ca`

## 完全自动化流程

### 当前设置（已完成）

1. **GitHub Actions 工作流**：`.github/workflows/build-binaries.yml`
   - ✅ 自动构建 macOS 二进制包（Intel 和 Apple Silicon）
   - ✅ 自动创建 GitHub Release
   - ✅ 自动计算和上传 SHA256 校验和
   - ✅ **自动更新 Homebrew formula**
   - ✅ **自动推送到 homebrew-tap 仓库**

2. **触发条件**：
   - 只在推送版本标签时触发：`git tag v1.1.2 && git push origin v1.1.2`
   - 不在普通提交或 PR 上触发

3. **Homebrew Tap**：
   - `zh30/tap` tap 已配置
   - formula 自动更新

## 使用方法

### 用户安装（简单）

```bash
# 只需两步
brew tap zh30/tap
brew install git-ca
```

### 开发人员发布（完全自动化）

```bash
# 1. 更新版本号
vim Cargo.toml
git commit -m "chore: bump version to v1.1.2"
git push origin main

# 2. 创建版本标签（触发自动化）
git tag v1.1.2
git push origin v1.1.2

# 3. 等待 5-10 分钟，GitHub Actions 自动：
#    - 构建二进制包
#    - 创建 Release
#    - 更新 Homebrew formula
#    - 推送到 homebrew-tap

# 4. 完成！用户可以立即安装新版本
```

## 自动化流程详解

当您推送版本标签时，GitHub Actions 自动执行：

1. **构建阶段**
   - 在 macOS 13 上构建 Intel 版本
   - 在 macOS 14 上构建 Apple Silicon 版本
   - 创建压缩包

2. **发布阶段**
   - 创建 GitHub Release
   - 上传二进制包
   - 生成并上传校验和文件

3. **Homebrew 更新阶段**（新功能）
   - 下载校验和文件
   - 计算 source tarball 校验和
   - 生成新的 `git-ca.rb` 文件
   - 推送到 `zh30/homebrew-tap` 仓库

## 需要配置的密钥

在 GitHub 仓库设置中添加以下 Secrets：

1. **`GITHUB_TOKEN`**（自动提供）
   - 用于下载 Release 资产
   - 推送更新到 homebrew-tap

2. **`TARGET_REPO_PAT`**（需要创建）
   - Personal Access Token
   - 权限：`repo` (完全控制)
   - 用于推送到 homebrew-tap 仓库

### 创建 TARGET_REPO_PAT 步骤

1. 访问 GitHub → Settings → Developer settings → Personal access tokens
2. 点击 "Generate new token (classic)"
3. 设置权限：
   - ✅ repo (Full control of private repositories)
4. 复制生成的 token
5. 在当前仓库的 Settings → Secrets and variables → Actions 中添加：
   - Name: `TARGET_REPO_PAT`
   - Value: 粘贴生成的 token

## 验证设置

### 检查工作流

1. 推送一个测试标签：
   ```bash
   git tag v999.0.0
   git push origin v999.0.0
   ```

2. 访问 GitHub → Actions 查看工作流运行

3. 检查：
   - ✅ 构建成功
   - ✅ Release 创建成功
   - ✅ Homebrew 更新成功

### 测试安装

在另一台 macOS 机器上测试：

```bash
brew tap zh30/tap
brew install git-ca
git ca --version
```

## 故障排除

### 常见问题

1. **Homebrew 更新失败**
   - 检查 `TARGET_REPO_PAT` 是否正确
   - 确认 homebrew-tap 仓库存在且有写权限

2. **构建失败**
   - 检查 Rust 工具链
   - 查看 Actions 日志中的错误信息

3. **校验和不匹配**
   - 重新计算校验和
   - 检查二进制包是否正确构建

### 查看日志

```bash
# 查看工作流状态
gh run list

# 查看特定工作流日志
gh run view <run-id>
```

## 优势

✅ **完全自动化** - 只需推送标签，无需手动步骤
✅ **快速发布** - 5-10 分钟完成整个流程
✅ **用户友好** - 用户使用简单命令安装
✅ **可靠** - 自动化减少人为错误
✅ **节省时间** - 释放开发者时间专注于代码

## 总结

现在，当您推送版本标签时：
1. GitHub Actions 自动构建和发布
2. 自动更新 Homebrew formula
3. 用户可以立即使用 `brew install git-ca` 安装

完全自动化已实现！🎉
