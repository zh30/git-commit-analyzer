# CI/CD Flow

This document outlines the current CI/CD pipeline for git-ca.

## Workflow Files

### `.github/workflows/build-binaries.yml`
**Purpose**: Build binaries, create release, and update Homebrew formula

**Trigger**:
- Push of version tags (`v*.*.*`)

**Actions**:
1. Builds binaries for macOS (Intel & Apple Silicon)
2. Creates compressed archives (.tar.gz)
3. Creates GitHub Release with changelog
4. Computes and uploads SHA256 checksums
5. **Automatically updates Homebrew formula** with new version and checksums
6. Pushes formula to `homebrew-tap` repository

**Outputs**:
- Release artifacts for download
- checksums.txt file
- Updated Homebrew formula

## Release Process (Fully Automated)

### Step 1: Create Version Tag
```bash
git tag v1.1.2
git push origin v1.1.2
```

### Step 2: Wait for CI
GitHub Actions will:
- Build binaries for both macOS platforms
- Create GitHub Release with changelog
- Upload checksums
- **Automatically update Homebrew formula**
- **Automatically push to homebrew-tap**

### Step 3: Done!
Users can install immediately with:
```bash
brew tap zh30/tap
brew install git-ca
```

## Platforms

**Supported**: macOS (Intel x86_64, Apple Silicon ARM64)
**Disabled**: Linux (compilation issues), Windows (not needed)

## Notes

- **Fully automated**: No manual steps required after tagging
- Only version tags trigger builds (not regular commits or PRs)
- Build artifacts are retained for 30 days
- Homebrew formula updates are completely automatic
