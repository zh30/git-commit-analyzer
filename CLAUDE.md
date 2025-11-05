# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Purpose**: CLI helper that generates Git Flow–style commit messages from staged changes using local llama.cpp inference.

**Runtime**: Pure Rust binary (`bin = "git-ca"`). No web services or VS Code extension.

**AI Backend**: Local llama.cpp inference via the `llama_cpp_sys_2` crate. Models are GGUF files discovered in local directories, with the chosen path persisted for subsequent runs (non-interactive invocations reuse the stored path or the first match).

**Prompt Workflow**: Staged diff is summarised, validated, and fed to the model; invalid output triggers retries with stricter instructions, then falls back to deterministic generation.

## High-Level Architecture

The application follows a pipeline architecture with clear separation of concerns:

```
CLI Args → Model Selection → Diff Retrieval → Diff Summarization
                                                               ↓
Commit Creation ← Message Validation ← Response Processing ← Model Inference
                                                               ↑
                                                      Prompt Generation
```

### Core Components

**1. CLI Orchestration (`main.rs:1867-2028`)**
- Parses command-line arguments (doctor, model, language commands)
- Orchestrates the entire workflow
- Handles user interactions for commit confirmation

**2. Model Management (`main.rs:1363-1718`)**
- Scans default directories for GGUF files:
  - `./models` (project directory)
  - `~/.cache/git-ca/models` (Linux)
  - `~/.local/share/git-ca/models` (Linux alt)
  - `~/Library/Application Support/git-ca/models` (macOS)
- Downloads default model (`unsloth/gemma-3-270m-it-GGUF`) from Hugging Face if none found
- Persists selection to `~/.cache/git-ca/default-model.path` or `.git-ca/default-model.path`

**3. Diff Processing (`main.rs:414-962`)**
- **Retrieval**: `get_diff()` - uses `git diff --cached` to get staged changes
- **Analysis**: `analyze_diff_summary()` - parses diff to extract file types, scope candidates, detect patterns
- **Summarization**: `build_diff_summary()` - reduces large diffs to concise summaries with snippets
- **Variants**: `build_diff_variants()` - creates summary and raw variants for retry attempts

**4. Prompt Engineering (`main.rs:421-488`)**
- Builds language-specific prompts (English/Chinese)
- Enforces Git Flow format: `<type>(<scope>): <subject>`
- Includes strict validation rules
- Stricter retry prompts on subsequent attempts

**5. Model Inference (`llama.rs:48-432`)**
- **Session Management**: `LlamaSession::new()` - loads GGUF model, initializes context
- **Tokenization**: Handles prompt encoding with buffer resizing
- **Generation**: Token-by-token sampling with temperature/top-k/top-p
- **Chunked Decoding**: Processes long prompts in 256-token chunks
- **Context Management**: Clears KV cache between runs, respects 1024-token limit

**6. Response Processing (`main.rs:546-667`)**
- Strips `<thinking>` blocks if present
- Extracts commit subject line matching Git Flow pattern
- Collects body text until instruction keywords detected
- Validates against `COMMIT_TYPES` array

**7. Fallback Generation (`main.rs:1141-1189`)**
- Deterministic commit synthesis when model fails
- Analyzes diff summary to determine:
  - **Type**: feat, fix, docs, chore, etc.
  - **Scope**: from file paths (src/main.rs → cli, docs files → docs, etc.)
  - **Subject**: from template enum based on context
- Handles special cases: dependency updates, runtime changes, retry patterns

**8. Validation (`main.rs:1190-1239`)**
- `is_valid_commit_message()` - enforces Git Flow format
- `parse_commit_subject()` - extracts type, optional scope, and subject
- English mode requires ASCII subject line
- Triggers retry loop on invalid output

## Key Dependencies

- `git2` — Git plumbing (staged diff, repository metadata, commit creation)
- `llama-cpp-sys-2` — FFI bindings to llama.cpp
- `hf-hub` — Optional Hugging Face download helper for the default model
- `rand` — Sampling randomness for token generation

## Source Layout

- `src/main.rs` — CLI entrypoint, Git integration, diff summarizer, fallback commit generator
  - **Lines 1-400**: Language enum with 40+ localized methods
  - **Lines 414-962**: Diff processing functions
  - **Lines 421-544**: Prompt building and model interaction
  - **Lines 1141-1189**: Fallback generation logic
  - **Lines 1720-1865**: Unit tests
  - **Lines 1867-2028**: main() and command routing

- `src/llama.rs` — llama.cpp session wrapper
  - **Lines 48-110**: `LlamaSession::new()` - model loading and context setup
  - **Lines 112-229**: `infer()` - prompt processing and text generation
  - **Lines 231-272**: `decode_sequence()` - chunked prompt decoding
  - **Lines 274-384**: `sample_next_token()` - sampling with temperature/top-k/top-p
  - **Lines 386-413**: `token_to_string()` - detokenization
  - **Lines 416-432**: Drop implementation for cleanup

## Configuration

- `commit-analyzer.language` — Prompt language (`en`, `zh`)
- **Llama context length**: Fixed to 1024 tokens (`DEFAULT_CONTEXT_SIZE`)
- **Model persistence**: Paths stored in `~/.cache/git-ca/default-model.path` or `.git-ca/default-model.path`
- **Sampling parameters**: Temperature 0.8, Top-K 40, Top-P 0.9, Min-P 0.0

## Common Development Tasks

### Add a Feature
1. **Architecture First**: Keep new logic scoped to `src/main.rs` or `src/llama.rs` until the surface area justifies extracting a module
2. **Unit Tests**: Add inline tests in `#[cfg(test)]` modules beside the code they cover
3. **Integration Tests**: Multi-step workflows combining Git operations + model inference should be promoted to a `tests/` directory
4. **Verify**: Run `cargo fmt && cargo clippy -- -D warnings && cargo test`
5. **Manual Testing**: Use `cargo run -- git ca` in a test repo with staged changes and document output in PR description
6. **Documentation**: Update `README*.md`, `DEPLOY.md`, and `CLAUDE.md` when behavior changes

### Modify Model Handling
- Update `LlamaSession` in `src/llama.rs` for inference logic
- Adjust sampling parameters (lines 19-22 in `llama.rs`)
- Modify `generate_fallback_commit_message` in `src/main.rs` for different deterministic logic
- Update `DEFAULT_MODEL_REPO` if changing defaults

### Adjust Prompts
- `build_commit_prompt()` (lines 421-488) - update language-specific instructions
- `build_diff_summary()` (lines 784-919) - change how diffs are summarized
- Add new language support via `Language` enum methods
- Update multilingual READMEs accordingly

### Debug Model Issues
- Run `git ca doctor` to test model loading and inference
- Use `debug_model_response()` (lines 398-400) to log model output
- Check `analyze_diff()` retry logic (lines 490-544)
- Verify context size handling (lines 157-163 in `llama.rs`)

## Testing Guidelines

**Unit Tests** (in `#[cfg(test)]` at bottom of `main.rs`):
- `handles_extracts_subject_line` - Response parsing
- `handles_includes_body_until_instruction` - Body extraction
- `validates_git_flow_subject` - Validation logic
- `fallback_generates_for_*` - Fallback behavior
- `truncates_diff_for_prompt` - Diff summarization

**Integration Testing**:
- No `tests/` directory currently
- Use `cargo run -- git ca` against real repositories
- Test edge cases: empty diffs, very large diffs, generated files
- Verify fallback triggers: model errors, invalid output, empty responses

## Error Handling

- **Model Loading**: Returns descriptive errors if GGUF file missing or invalid
- **Tokenization**: Buffer resizing handles oversized prompts
- **Inference**: KV cache clearing between runs, chunked decoding with fallback
- **Validation**: Retry loop (2 attempts) before falling back to deterministic generation
- **Git Operations**: Propagates `git2::Error` with context

## Development Commands

```bash
# Format, lint, and test
cargo fmt
cargo clippy -- -D warnings
cargo test

# Run against staged changes
cargo run -- git ca

# Test model loading and inference
cargo run -- git ca doctor

# Select or download model
cargo run -- git ca model
cargo run -- git ca model pull unsloth/gemma-3-270m-it-GGUF

# Change language
cargo run -- git ca language

# Release build
cargo build --release
```

## Distribution

- Release binaries: `cargo build --release` produces optimized binary
- **Homebrew**: Formula at `git-ca.rb` with version and SHA256
- **Installer**: `install-git-ca.sh` for automated setup
- Documentation: `README.md`, `README_ZH.md`, `README_FR.md`, `README_ES.md`
- Keep `README.md` / `DEPLOY.md` / `INSTALL.md` in sync with code changes

## Critical Implementation Details

**Diff Summarization Strategy** (`build_diff_summary`):
1. Identifies generated/large files (lockfiles, minified JS/CSS)
2. Extracts file metadata: additions, deletions, file type
3. Includes code snippets up to 120 lines or 1200 characters per file
4. Truncates when approaching context limit (3× context - 512 chars)
5. Marks omitted content with notices

**Model Sampling** (`sample_next_token`):
1. Retrieves logits from llama.cpp
2. Applies temperature scaling
3. Filters to top-K candidates
4. Applies top-p (nucleus) filtering
5. Samples using weighted random selection
6. Prevents EOS tokens until meaningful text generated

**Context Management**:
- Fixed 1024-token context window
- Prompts truncated if exceeding `n_ctx - 32`
- Raw diff tail used as fallback variant
- KV cache cleared between inferences

## Architecture Decisions

- **Single Binary**: No web service or extension - keeps deployment simple
- **Local Inference**: Privacy and offline capability using llama.cpp directly (via `llama-cpp-sys-2`)
- **Manual Args Parsing**: Avoids `clap` dependency bloat
- **Inline Tests**: Co-located with code for easy maintenance; integration flows go to `tests/`
- **Deterministic Fallback**: Ensures commits succeed even when model fails
- **No Async**: Simple synchronous execution pattern
- **Minimal Modules**: Resist premature abstraction - keep logic in `main.rs`/`llama.rs` until justified
- **Generated Assets**: Keep under `target/` or other ignored directories - never in `src/` or `tests/`

## Performance Considerations

- **Chunked Prompt Decoding**: Handles long prompts without memory spikes
- **KV Cache Clearing**: Prevents memory buildup across runs
- **Diff Truncation**: Reduces context size for faster inference
- **Sampling Parameters**: Tuned for creativity while maintaining coherence
- **Thread Auto-Detection**: Uses available parallelism from system

## Security Notes

- No remote API calls (except optional Hugging Face model download)
- No credential storage beyond Git config
- Validates model files (checks `.gguf` extension)
- Sanitizes file paths and model paths
- No code execution from model output

## Coding Conventions

### Naming
- Functions/files: `snake_case`
- Types/enums: `CamelCase`
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `COMMIT_TYPES`)

### Error Handling
- Prefer error propagation with `?` operator
- Return `AppError::Custom` only when you need user-facing messages
- Comments should explain non-obvious Git plumbing or llama-specific constraints

### Formatting
- Rustfmt defaults: 4-space indent, 100-column width
- Run `cargo fmt` before committing

## Reminders

- Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` before committing
- Test both model generation and fallback paths (stage deps-only diffs, runtime changes)
- Update `README*.md`, `DEPLOY.md`, and `CLAUDE.md` when behavior changes
- Document manual `git ca` verification steps in PR descriptions
