# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` hosts the CLI entrypoint, Git/Ollama integrations, and all user prompts; keep new modules small and import them from `main.rs` until a `src/` submodule is justified.
- `install-git-ca.sh` and `git-ca.rb` handle installer automation (shell and Homebrew); update both when distribution steps change.
- `README*.md`, `CLAUDE.md`, and `DEPLOY.md` are user-facing references—mirror any behavior changes here.
- Build artifacts land in `target/`; never commit that directory. Generated assets belong under `target/` or a new ignored path, not under `src/`.

## Build, Test, and Development Commands
- `cargo build --release` compiles the binary that installers copy into `~/.git-plugins`.
- `cargo run -- git ca` runs the CLI with local changes; use staged diffs in a sample repo to validate prompts.
- `cargo fmt` and `cargo clippy -- -D warnings` enforce Rust style and catch regressions before review.

## Coding Style & Naming Conventions
- Follow rustfmt defaults (4-space indent, 100-column wrap) via `cargo fmt`; do not hand-format.
- Use `snake_case` for functions/files, `SCREAMING_SNAKE_CASE` for constants like `COMMIT_TYPES`, and `CamelCase` for types/enums.
- Prefer descriptive error messages and `?` propagation; add short comments only around non-obvious Git/Ollama logic.

## Testing Guidelines
- Add unit tests in `#[cfg(test)]` modules next to the code under test; name functions `fn handles_*` to reflect behavior.
- Place integration tests under `tests/` when flows require multiple Git operations.
- Run `cargo test` locally; target meaningful branch coverage for new logic and document any manual verification (`git ca` run with staged fixtures) in the PR.

## Commit & Pull Request Guidelines
- Use Conventional Commit prefixes observed in history (e.g., `feat(client): ...`, `chore: ...`); scopes and descriptions can be English or Simplified Chinese.
- Squash work into logically complete commits with passing builds/tests.
- PRs must include: summary of behavior change, testing notes (`cargo test`, manual `git ca` checks), and updates to impacted docs.
- Link related issues and add screenshots or terminal captures when altering user prompts or install UX.

## Ollama & Model Configuration Tips
- Keep a local Ollama instance running at `localhost:11434`; document any alternative endpoints in `DEPLOY.md` before merging.
- When introducing new model flows, ensure defaults are persisted via `git config` keys `commit-analyzer.model` and `commit-analyzer.language`.
