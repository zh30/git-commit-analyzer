# Homebrew 发布指南

本文档记录了将 `git-ca` 发布到 Homebrew tap 或 Homebrew Core 的流程。

## 1. 生成发布包

1. bump 版本号：更新 `Cargo.toml` 与 `Cargo.lock`。
2. 运行 `cargo fmt && cargo clippy -- -D warnings && cargo test`。
3. 构建发布包并获取校验值：
   ```bash
   cargo build --release
   tar -C target/release -czf git-ca-$VERSION-x86_64.tar.gz git-ca
   shasum -a 256 git-ca-$VERSION-x86_64.tar.gz
   ```
4. 在 GitHub 创建 `v$VERSION` 标签与 Release，上传上述 tar 包。

## 2. 更新 Homebrew 配方

无论提交到官方 Homebrew Core 还是自建 tap，都需要更新 `git-ca.rb`：

- 将 `url` 指向新发布的 tar 包。
- 将 `sha256` 替换为最新校验值。
- 调整 `version`。
- 如依赖/构建步骤有变化（例如新增 `cmake`、`libomp`），同步更新 `depends_on`。

### 方案 A：Homebrew Core
1. Fork [Homebrew/homebrew-core](https://github.com/Homebrew/homebrew-core)。
2. 更新 `Formula/g/git-ca.rb`。
3. 运行 `brew audit --new-formula git-ca`（或 `--strict git-ca`）。
4. 提交 PR，并在描述中附上`brew install --build-from-source git-ca`与`brew test git-ca`的输出。

### 方案 B：自建 Tap
1. 创建形如 `zh30/homebrew-tap` 的仓库。
2. 将配方放在 `Formula/git-ca.rb`。
3. 用户安装方式：
   ```bash
   brew tap zh30/tap
  brew install git-ca
   ```

## 3. 本地验证

在提交前务必执行：

```bash
brew install --build-from-source ./git-ca.rb
brew test git-ca
brew audit --strict git-ca
```

测试内容应至少覆盖：
- `git ca --version`
- `git ca model`（交互式选择模型）
- 运行一次 `git ca`，确认 llama.cpp 库能够被加载，且 fallback 行为正常。

## 4. 发布后维护

- 更新 `README.md`、`INSTALL.md` 中的 Homebrew 示例命令。
- 若默认模型或上下文配置有变更，请同步更新配方中的提示（`caveats`）。
- 监控问题反馈，重点关注模型下载/依赖变更导致的安装失败。
