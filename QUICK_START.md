# 快速开始指南

## 🚀 完全自动化 Homebrew 发布

### 用户安装（1分钟）

```bash
brew tap zh30/tap
brew install git-ca
```

### 开发人员发布（5分钟）

```bash
# 1. 更新版本
vim Cargo.toml
git commit -m "chore: version bump"
git push

# 2. 发布（触发自动化）
git tag v1.1.2
git push origin v1.1.2

# 3. 等待 5-10 分钟，然后用户就可以安装新版本了！
```

## 📋 需要做的

### 在 GitHub 仓库设置中添加 Secret：

1. **名称**: `TARGET_REPO_PAT`
2. **值**: Personal Access Token（权限：`repo`）
3. **位置**: Settings → Secrets and variables → Actions

### 如何创建 Token：

```
GitHub → Settings → Developer settings → Personal access tokens → 
Generate new token (classic) → 勾选 repo → Generate → 复制
```

## ✅ 自动化流程

推送版本标签后，GitHub Actions 自动：

1. ✅ 构建 macOS 二进制包（Intel + Apple Silicon）
2. ✅ 创建 GitHub Release
3. ✅ 上传校验和
4. ✅ **自动更新 Homebrew formula**
5. ✅ **自动推送到 homebrew-tap**

## 📚 文档

- **自动化设置**: `AUTOMATION_SETUP.md`
- **CI/CD 流程**: `CI_CD_FLOW.md`
- **发布指南**: `DEPLOY.md`
- **Homebrew 指南**: `HOMEBREW.md`

## 🎯 关键点

- ✅ **Formula**（不是 Cask）: `brew install git-ca`
- ✅ **只支持 macOS**: Intel + Apple Silicon
- ✅ **仅标签触发**: 不在普通提交上运行
- ✅ **完全自动化**: 无需手动步骤

## 🆘 故障排除

**Homebrew 更新失败？**
- 检查 `TARGET_REPO_PAT` Secret
- 确认 homebrew-tap 仓库存在

**构建失败？**
- 查看 Actions 日志
- 检查 Rust 依赖

---

🎉 **现在就开始使用完全自动化的发布流程吧！**
