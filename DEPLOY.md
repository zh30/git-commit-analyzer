# Deploy & Release Guide

This document outlines the steps for shipping a new version of Git Commit Analyzer and keeping distribution channels in sync.

## 1. Pre-release Checklist
- [ ] Update `Cargo.toml` / `Cargo.lock` version.
- [ ] Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.
- [ ] Smoke test `cargo run -- git ca` against a fixture repository (document output in the PR).
- [ ] Review `README*.md`, `INSTALL.md`, `AGENTS.md`, and `CLAUDE.md` for accuracy (context tuning, fallback behaviour, config keys).
- [ ] Update `CHANGELOG.md` (if maintained) or include release notes in the PR/Release description.

## 2. Build Artifacts
```bash
cargo build --release
tar -C target/release -czf git-ca-$VERSION-x86_64.tar.gz git-ca
shasum -a 256 git-ca-$VERSION-x86_64.tar.gz
```
Capture the SHA256 hash for Homebrew and installer updates.

## 3. GitHub Release
1. Tag the commit (`git tag -a v$VERSION -m "git-ca v$VERSION"`).
2. Push tags (`git push origin v$VERSION`).
3. Create a GitHub release:
   - Title `git-ca v$VERSION`.
   - Upload the tarball.
   - Paste release notes (highlights, breaking changes, upgrade instructions).

## 4. Installer Script
`install-git-ca.sh` bootstraps dependencies, builds the binary, and configures PATH.
- Update version references and checksums if the script pins artefacts.
- Verify the script installs the latest release on macOS and Linux.
- Host the script at `https://sh.zhanghe.dev/install-git-ca.sh` (or your CDN) and update README links if the URL changes.
- Optional: publish copy under a versioned path (e.g. `install-git-ca-v$VERSION.sh`) for deterministic installs.

## 5. Homebrew Formula (`git-ca.rb`)
1. Update `url` to the new GitHub release tarball.
2. Replace `sha256` with the fresh checksum.
3. Bump the `version`.
4. `brew install --build-from-source ./git-ca.rb` to validate.
5. `brew test git-ca` and `brew audit --strict git-ca`.
6. Publish through your tap (`brew tap zh30/tap`) or submit to Homebrew if appropriate.

## 6. Model Distribution Notes
The CLI defaults to downloading `unsloth/gemma-3-270m-it-GGUF` if no local model is configured.
- Confirm the Hugging Face repository is accessible and that rate limits are acceptable.
- Document any repository or checksum changes in `README.md` and `INSTALL.md`.
- If shipping a custom model, mirror it to a stable location and adjust `DEFAULT_MODEL_REPO` in code.

## 7. Post-release Verification
- Re-run the installer script on macOS and Linux (fresh machines or containers) to ensure all dependencies resolve.
- Install via Homebrew and execute `git ca --version`.
- Confirm fallbacks still operate:
  - Stage a dependency-only diff and run `git ca` (expect `chore(deps): ...`).
  - Stage a runtime change and ensure the model/fallback yields a `fix(...)` message.
- Monitor issues for feedback on model downloads, context limits, or installation regressions.

## 8. Communication
- Announce the release (GitHub, project page, changelog).
- Note any environment changes (e.g., new minimum Rust version, different default context).
- Provide upgrade instructions if manual steps are required (e.g., re-selecting the model).
