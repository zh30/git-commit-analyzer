# Repository Guidelines

## Project Structure & Module Organization
`src/main.rs` hosts the Rust CLI that wires git2, MLX prompts, and interactive output; keep additional Rust modules in `src/` so `cargo` picks them up automatically. Helper assets live at the repository root: `generate_commit.py` streams tokens from mlx-lm, installer scripts (`install-git-ca.sh`, `git-ca.rb`) handle distribution, and language-specific READMEs document usage. Store new fixtures or resources beside the code they exercise and update the relevant README when behavior changes.

## Build, Test, and Development Commands
- `cargo check` — fast iteration pass that catches type errors.
- `cargo fmt --all` — formats Rust sources before committing.
- `cargo clippy --all-targets --all-features -D warnings` — enforces lint hygiene for the binary.
- `cargo test` — runs Rust unit and integration suites.
- `cargo run -- [args]` — exercises the CLI locally; add `--dry-run` for non-MLX flows.
- `cargo build --release` — produces the artifact used by the installer and Homebrew tap.

## Coding Style & Naming Conventions
Rely on Rust 2021 defaults: four-space indentation, snake_case for functions and variables, CamelCase for types, and SCREAMING_SNAKE_CASE for constants. Keep user-facing strings localized in English with Simplified Chinese counterparts when touching language prompts, and mirror PEP 8 conventions if editing Python helpers. Document any new environment variables or config keys in `README.md` and the install scripts.

## Testing Guidelines
Add targeted unit tests alongside new logic in `src/`, and place scenario-level checks under `tests/` when validating CLI behavior. Prefer deterministic git diff fixtures so MLX responses stay predictable; mock the model call when practical and assert on the rendered commit text. Run `cargo test` (and any manual `git ca` checks you rely on) before opening a PR.

## Commit & Pull Request Guidelines
Follow the practiced Conventional Commit pattern: `type(scope): summary`, using scopes such as `install-script` or `docs`. Keep summaries imperative, under 72 characters, and focused on a single change. Pull requests should outline intent, list validation commands (e.g., `cargo test`, manual `git ca` run), mention documentation updates, and link issues or release tasks that depend on the change.

## MLX & Environment Notes
Local development assumes Apple Silicon with `pip install mlx-lm` available; if MLX is unavailable, flag the limitation in your PR and provide alternative verification. When altering output formats or installer behavior, coordinate updates across `generate_commit.py`, `install-git-ca.sh`, and `git-ca.rb` so the release workflow remains consistent.
