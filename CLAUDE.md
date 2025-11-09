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

**1. CLI Orchestration (src/main.rs)**
- Parses command-line arguments (doctor, model, language commands)
- Orchestrates the entire workflow
- Handles user interactions for commit confirmation

**2. Model Management (src/main.rs)**
- Scans default directories for GGUF files:
  - `./models` (project directory)
  - `~/.cache/git-ca/models` (Linux)
  - `~/.local/share/git-ca/models` (Linux alt)
  - `~/Library/Application Support/git-ca/models` (macOS)
- Downloads default model (`unsloth/gemma-3-270m-it-GGUF`) from Hugging Face if none found
- Persists selection to `~/.cache/git-ca/default-model.path` or `.git-ca/default-model.path`

**3. Diff Processing (src/main.rs)**
- **Retrieval**: `get_diff()` - uses `git diff --cached` to get staged changes
- **Analysis**: `analyze_diff_summary()` - parses diff to extract file types, scope candidates, detect patterns
- **Summarization**: `build_diff_summary()` - reduces large diffs to concise summaries with snippets
- **Variants**: `build_diff_variants()` - creates summary and raw variants for retry attempts

**4. Prompt Engineering (src/main.rs)**
- Builds language-specific prompts (English/Chinese)
- Enforces Git Flow format: `<type>(<scope>): <subject>`
- Includes strict validation rules
- Stricter retry prompts on subsequent attempts

**5. Model Inference (src/llama.rs)**
- **Session Management**: `LlamaSession::new()` - loads GGUF model, initializes context
- **Tokenization**: Handles prompt encoding with buffer resizing
- **Generation**: Token-by-token sampling with temperature/top-k/top-p
- **Chunked Decoding**: Processes long prompts in 256-token chunks
- **Context Management**: Clears KV cache between runs, respects 1024-token limit

**6. Response Processing (src/main.rs)**
- Strips `<thinking>` blocks if present
- Extracts commit subject line matching Git Flow pattern
- Collects body text until instruction keywords detected
- Validates against `COMMIT_TYPES` array

**7. Fallback Generation (src/main.rs)**
- Deterministic commit synthesis when model fails
- Analyzes diff summary to determine:
  - **Type**: feat, fix, docs, chore, etc.
  - **Scope**: from file paths (src/main.rs → cli, docs files → docs, etc.)
  - **Subject**: from template enum based on context
- Handles special cases: dependency updates, runtime changes, retry patterns

**8. Validation (src/main.rs)**
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
  - Language enum with 40+ localized methods
  - Diff processing functions
  - Prompt building and model interaction
  - Fallback generation logic
  - Unit tests
  - main() and command routing

- `src/llama.rs` — llama.cpp session wrapper
  - Model loading and context setup
  - Prompt processing and text generation
  - Chunked prompt decoding
  - Token sampling with temperature/top-k/top-p
  - Detokenization

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

# Run a single test
cargo test handles_extracts_subject_line
```

## Configuration

- `commit-analyzer.language` — Prompt language (`en`, `zh`)
- **Llama context length**: Fixed to 1024 tokens (`DEFAULT_CONTEXT_SIZE`)
- **Model persistence**: Paths stored in `~/.cache/git-ca/default-model.path` or `.git-ca/default-model.path`
- **Sampling parameters**: Temperature 0.8, Top-K 40, Top-P 0.9, Min-P 0.0

## Critical Implementation Details

**Diff Summarization Strategy**:
1. Identifies generated/large files (lockfiles, minified JS/CSS)
2. Extracts file metadata: additions, deletions, file type
3. Includes code snippets up to 120 lines or 1200 characters per file
4. Truncates when approaching context limit (3× context - 512 chars)
5. Marks omitted content with notices

**Model Sampling**:
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

## Related Documentation

- `README.md` - Project overview and installation
- `AGENTS.md` - Repository guidelines for contributors
- `DEPLOY.md` - Release process and distribution
- `HOMEBREW.md` - Homebrew formula management
- `INSTALL.md` - Installation methods
- `QUICK_START.md` - Quick start guide
