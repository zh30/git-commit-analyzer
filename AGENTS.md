# Repository Guidelines

## Project Structure & Module Organization
The CLI entrypoint, prompt workflow, and llama.cpp bindings live in `src/main.rs`, with engine-specific helpers in `src/llama.rs`. Keep new logic scoped to these files until the surface area justifies an extracted module. Inline unit tests belong beside the code they cover; multi-step workflows (Git staging, model selection) should be promoted to `tests/`. Generated assets remain under `target/` or another ignored directory—never check them into `src/` or `tests/`.

## Build, Test, and Development Commands
- `cargo build --release` — produce the optimized binary that installers copy to `~/.git-plugins`.
- `cargo run -- git ca` — run the analyzer against staged changes in the current repo to validate prompt flows.
- `cargo fmt` — enforce rustfmt defaults (4-space indent, 100-column width).
- `cargo clippy -- -D warnings` — lint with warnings treated as build failures.
- `cargo test` — execute all unit tests; run before every commit and PR.
- Llama.cpp context length is fixed to 1024 tokens.

## Coding Style & Naming Conventions
Use `snake_case` for functions/files, `CamelCase` for types/enums, and `SCREAMING_SNAKE_CASE` for constants such as `COMMIT_TYPES`. Let rustfmt manage alignment and spacing. Prefer error propagation with `?`, returning `AppError::Custom` only when you need a user-facing message. Comments should explain non-obvious Git plumbing or llama-specific constraints; avoid restating what the code already conveys.

## Testing Guidelines
Unit tests live in `#[cfg(test)]` modules with descriptive names like `handles_retry_backoff`. Integration flows that combine Git operations, prompt generation, and llama inference should move into `tests/`. Always run `cargo test` locally and note any manual `cargo run -- git ca` checks (e.g., staged fixture repos) in PR descriptions. Target meaningful branch coverage over exhaustive mocking.

## Commit & Pull Request Guidelines
Follow the existing Conventional Commit style—examples include `feat(cli): simplify prompt`, `fix(llama): handle kv cache reset`, `chore(deps): update dependencies`. Each PR must summarise behavior changes, list verification steps (tests, manual runs), and update affected docs (`README*.md`, `DEPLOY.md`, `CLAUDE.md`). Link relevant issues and include terminal captures when altering user-visible prompts or installer UX.

## Model & Configuration Tips
By default the tool scans `./models` and cache directories for llama.cpp-compatible GGUF files, persists the user's selection, and reuses it on subsequent runs. Non-interactive invocations reuse the stored model or fall back to the first discovered GGUF. Document any alternative endpoints or model defaults in `DEPLOY.md` before merging. Store credentials in ignored env files, not in tracked sources, and confirm large lockfiles remain ignored or summarized automatically by the diff truncation logic.
