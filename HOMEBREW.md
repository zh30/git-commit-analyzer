# 发布 git-ca 到 Homebrew

本文档描述了如何将 git-ca 发布到 Homebrew 的步骤。

## 创建发布

1. 确保代码已经准备好发布，包括：
   - 所有功能测试通过
   - 版本号已更新 (在 Cargo.toml 中)
   - CHANGELOG 已更新

2. 在 GitHub 上创建一个新的发布版本（Release）:
   - 标签应该是 `v1.0.0` 格式
   - 发布标题应该是 "git-ca v1.0.0"
   - 在描述中包含此版本的更新内容

3. 上传生成的 tar.gz 文件，或者让 GitHub 自动创建。

4. 计算发布压缩包的 SHA256 校验值：
   ```
   curl -L https://github.com/zh30/git-commit-analyzer/archive/refs/tags/v1.0.0.tar.gz | shasum -a 256
   ```

5. 复制得到的校验值，并更新 `git-ca.rb` 文件中的 `sha256` 值。

## 提交到 Homebrew

### 选项 1: 提交到 Homebrew Core

如果你想将 git-ca 作为官方的 Homebrew 包，请按照以下步骤操作：

1. Fork [Homebrew Core 仓库](https://github.com/Homebrew/homebrew-core)
2. 将更新后的 `git-ca.rb` 文件保存到 `Formula/g/git-ca.rb`
3. 提交一个 Pull Request

### 选项 2: 创建自己的 Tap

如果你想通过自己的 Tap 分发，这是更简单的方法：

1. 创建一个新的仓库，命名为 `homebrew-tap`
2. 将 `git-ca.rb` 文件添加到这个仓库
3. 用户可以通过以下命令安装：
   ```
   brew tap zh30/tap
   brew install git-ca
   ```

## 更新现有公式

当发布新版本时：

1. 更新 `url` 指向新版本
2. 更新 `sha256` 值
3. 提交更新后的公式

## 测试公式

在提交前进行测试：

```
brew install --build-from-source ./git-ca.rb
brew test git-ca
brew audit --strict git-ca
```

确保所有测试都通过，然后才能提交到 Homebrew。 