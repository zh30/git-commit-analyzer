# CLAUDE.md

Guidance for Claude Code (claude.ai/code) when working inside this repository.

## Project Overview
- **Purpose**: CLI helper that generates Git Flow–style commit messages from staged changes.
- **Runtime**: Pure Rust binary (`bin = "git-ca"`). No web services or VS Code extension.
- **AI Backend**: Local llama.cpp inference via the `llama_cpp_sys_2` crate. Models are GGUF files discovered in local directories, with the chosen path persisted for subsequent runs (non-interactive invocations reuse the stored path or the first match).
- **Prompt Workflow**: Staged diff is summarised, validated, and fed to the model; invalid output triggers retries or a deterministic fallback.

## Key Dependencies
- `git2` — Git plumbing (staged diff, repository metadata).
- `hf-hub` — Optional Hugging Face download helper for the default model.
- `llama_cpp_sys_2` — FFI bindings to llama.cpp.
- `clap` (built-in via `env::args`) not used; argument parsing is manual.

## Source Layout
- `src/main.rs` — CLI entrypoint, Git integration, diff summariser, fallback commit generator.
  - `build_diff_summary` / `build_diff_variants` — assemble prompt-friendly summaries and raw tails.
  - `generate_fallback_commit_message` — deterministic commit synthesis when the model fails.
  - `analyze_diff` — orchestrates prompting, retries, and validation.
  - `is_valid_commit_message` & `parse_commit_subject` — enforce Git Flow format.
- `src/llama.rs` — wrapper around llama.cpp session lifecycle (`LlamaSession::new` / `infer`).
- No additional crates, workspaces, or extensions.

## Configuration
- `commit-analyzer.language` — prompt language (`en`, `zh`).
- Llama context length is fixed to 1024 tokens; model paths are chosen interactively.

## Development Commands
```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo run -- git ca           # run against staged changes
```

## Testing Notes
- Unit tests live inline (`#[cfg(test)]`) and focus on parsing, fallback selection, and diff truncation (`tests::...` in `src/main.rs`).
- There are currently no integration tests under `tests/`.

## Common Tasks
- **Add a feature**: edit `src/main.rs`, add unit tests next to the affected functions, run the command suite above, and update README(s) plus `AGENTS.md`.
- **Adjust model handling**: update `src/llama.rs` or the `generate_fallback_commit_message` pipeline, and refresh documentation covering runtime model selection.
- **Modify prompts**: touch `build_diff_summary`, `build_commit_prompt`, or language strings in `Language` enum; update multilingual READMEs accordingly.

## Distribution
- Release binaries are produced with `cargo build --release`.
- Homebrew formula (`git-ca.rb`) and installer script (`install-git-ca.sh`) rely on that binary; keep `README.md` / `DEPLOY.md` / `INSTALL.md` in sync.

## Reminders
- Respect existing instructions in `AGENTS.md`.
- Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` before submitting patches.
