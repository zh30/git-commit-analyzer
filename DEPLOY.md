# Deploy & Release Guide

This document outlines the complete release process for Git Commit Analyzer, including multi-platform binary builds and Homebrew bottle distribution.

## Multi-Platform Binary Releases

Git Commit Analyzer now supports **multi-platform pre-built binaries** via GitHub Actions, enabling fast Homebrew installation without source compilation.

### Supported Platforms
- **macOS**: Apple Silicon (arm64), Intel (x86_64)
- **Linux**: Temporarily disabled due to compilation issues
- **Windows**: Builds available via GitHub Releases (not distributed via Homebrew)

## 1. Pre-release Checklist

Before creating a release:

- [ ] Update version in `Cargo.toml`
- [ ] Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`
- [ ] Smoke test `cargo run -- git ca` against staged changes
- [ ] Review and update `README*.md`, `INSTALL.md`, `AGENTS.md`, `CLAUDE.md`
- [ ] Update `CHANGELOG.md` or include release notes in PR

## 2. Automated Release Workflow

### Triggering the Build

Push a version tag to automatically build and release:

```bash
# Update version
vim Cargo.toml

# Commit changes
git commit -m "chore: bump version to v1.1.2"
git push origin main

# Create and push tag
git tag v1.1.2
git push origin v1.1.2
```

### GitHub Actions Workflows

#### Build Binaries (`.github/workflows/build-binaries.yml`)

Triggered on:
- Push to `main` branch (for testing)
- Push of version tags `v*.*.*` (for release)

**Build Matrix:**
- macOS 13 (Intel x86_64)
- macOS 14 (Apple Silicon ARM64)
- **Note**: Linux and Windows builds can be enabled if needed (see `.github/workflows/build-binaries.yml`)

**Process:**
1. Checks out repository
2. Installs Rust toolchain and platform-specific dependencies
3. Builds release binary for target platform
4. Strips binaries (macOS/Linux) to reduce size
5. Creates compressed archives:
   - `.tar.gz` for macOS/Linux
   - `.zip` for Windows
6. Uploads artifacts to GitHub Actions

#### Release & Homebrew Update (`.github/workflows/release.yml`)

Triggered on version tags only.

**Process:**
1. Creates GitHub Release with:
   - Auto-generated changelog from commit history
   - Download links for all platforms
   - Installation instructions
2. Downloads all release assets
3. Calculates SHA256 checksums for each platform
4. Updates `git-ca.rb` Homebrew formula with:
   - Version number
   - Bottle checksums for all platforms
5. Pushes updated formula to `homebrew-tap` repository

## 3. Manual Release (Alternative)

If automated workflow fails:

```bash
# 1. Build for each platform manually
rustup target add x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

# macOS ARM64
cargo build --release --target aarch64-apple-darwin
cd target/aarch64-apple-darwin/release && tar czf ../../../../git-ca-apple-darwin-arm64.tar.gz git-ca && cd ../../../../

# macOS Intel
cargo build --release --target x86_64-apple-darwin
cd target/x86_64-apple-darwin/release && tar czf ../../../../git-ca-apple-darwin-x86_64.tar.gz git-ca && cd ../../../../

# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu
cd target/x86_64-unknown-linux-gnu/release && tar czf ../../../../git-ca-unknown-linux-gnu-x86_64.tar.gz git-ca && cd ../../../../

# Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu
cd target/aarch64-unknown-linux-gnu/release && tar czf ../../../../git-ca-unknown-linux-gnu-arm64.tar.gz git-ca && cd ../../../../

# Windows (requires PowerShell)
# ... (or use cross compilation with mingw)

# 2. Calculate checksums
shasum -a 256 git-ca-*.tar.gz > checksums.txt

# 3. Create GitHub release
gh release create v1.1.2 \
  --title "git-ca v1.1.2" \
  --notes-file CHANGELOG.md \
  git-ca-apple-darwin-arm64.tar.gz \
  git-ca-apple-darwin-x86_64.tar.gz \
  git-ca-unknown-linux-gnu-x86_64.tar.gz \
  git-ca-unknown-linux-gnu-arm64.tar.gz \
  checksums.txt

# 4. Update Homebrew formula manually
vim git-ca.rb
# Update version and bottle checksums

# 5. Update homebrew-tap
git clone https://github.com/zh30/homebrew-tap.git
cp git-ca.rb homebrew-tap/
cd homebrew-tap
git commit -m "chore: update git-ca to v1.1.2"
git push
```

## 4. Homebrew Formula Update

The `git-ca.rb` formula automatically receives updates via GitHub Actions.

### Formula Structure

```ruby
class GitCa < Formula
  desc "AI-powered Git plugin for generating meaningful commit messages"
  homepage "https://github.com/zh30/git-commit-analyzer"
  url "https://github.com/zh30/git-commit-analyzer/archive/refs/tags/v1.1.2.tar.gz"
  sha256 "SOURCE_TARBALL_SHA256"
  license "MIT"

  # Bottle definitions - auto-updated by GitHub Actions
  bottle do
    root_url "https://github.com/zh30/git-commit-analyzer/releases/download/v1.1.2"
    sha256 cellar: :any_skip_relocate, arm64_sequoia: "ARM64_MACOS_SHA256"
    sha256 cellar: :any_skip_relocate, x86_64_sequoia: "X86_64_MACOS_SHA256"
    sha256 cellar: :any_skip_relocate, arm64_linux: "ARM64_LINUX_SHA256"
    sha256 cellar: :any_skip_relocate, x86_64_linux: "X86_64_LINUX_SHA256"
  end

  def install
    bin.install "git-ca"
  end

  def caveats
    # Updated messaging about llama.cpp
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/git-ca --version")
  end
end
```

### Required Secrets

Configure these secrets in GitHub repository settings:

- `TARGET_REPO_PAT`: Personal access token for pushing to `homebrew-tap` repository
  - Required permissions: `repo` (full control)
  - Alternative: Use GitHub App with repository access

## 5. Installer Script

`install-git-ca.sh` remains available but now serves as an alternative to Homebrew.

**Updates for multi-platform:**
- Detect OS and architecture
- Download appropriate binary from GitHub releases
- Extract and install to `/usr/local/bin`
- Set executable permissions

## 6. Model Distribution Notes

No changes to model distribution - the CLI still:
- Defaults to downloading `unsloth/gemma-3-270m-it-GGUF` from Hugging Face
- Supports local GGUF files in `./models` or `~/.cache/git-ca/models`
- Uses llama.cpp (via `llama-cpp-sys-2`) for local inference

## 7. Post-release Verification

After release completes:

### GitHub Release
- [ ] Verify all 6 platforms built successfully
- [ ] Check download links work for each platform
- [ ] Validate checksums.txt contains all checksums
- [ ] Test release notes render correctly

### Homebrew
- [ ] Verify `homebrew-tap` repository updated with new formula
- [ ] Test installation on macOS (both ARM64 and x86_64):
  ```bash
  brew tap zh30/tap
  brew install git-ca
  git ca --version
  ```
- [ ] Confirm bottle is used (no source compilation)

### Manual Installation
- [ ] Download and test binary for each platform
- [ ] Verify executable permissions
- [ ] Test basic functionality

### Model Functionality
- [ ] Run `git ca model` to test model selection
- [ ] Test with a real repository:
  ```bash
  cd /tmp/test-repo
  git init
  echo "test" > test.txt
  git add .
  git ca  # Should generate a commit message
  ```

## 8. Rollback Procedure

If a release fails:

1. **GitHub Release**: Delete the release and tag
2. **Homebrew**: Rollback to previous version in `homebrew-tap`
3. **Documentation**: Restore previous README/INSTALL versions

## 9. Communication

Announce the release with:
- GitHub Release notes
- Updated installation instructions in README.md
- Social media/blog post (optional)

Include:
- Platform support matrix
- Installation commands
- Link to changelog
- Any migration notes

## 10. Troubleshooting

### Build Failures
```bash
# Check Rust targets
rustup target list --installed

# Verify dependencies
cargo tree --depth 1

# Clean rebuild
cargo clean
cargo build --release
```

### Homebrew Issues
```bash
# Force source install for debugging
HOMEBREW_NO_INSTALL_FROM_API=1 brew install --build-from-source zh30/tap/git-ca

# Verbose output
brew install -v zh30/tap/git-ca

# Audit formula
brew audit --strict zh30/tap/git-ca
```

### Release Workflow Issues
- Check GitHub Actions logs
- Verify `TARGET_REPO_PAT` secret is valid
- Ensure `homebrew-tap` repository exists and is accessible
- Confirm `git-ca.rb` syntax is valid Ruby
