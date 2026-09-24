mod llama;

use crate::llama::LlamaSession;
use git2::{Commit, Config, ErrorCode, Repository, Signature};
use hf_hub::api::sync::Api;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
const CONFIG_MODEL_TIER_KEY: &str = "commit-analyzer.model-tier";
const CONFIG_CONTEXT_KEY: &str = "commit-analyzer.context";
const COMMIT_TYPES: &[&str] = &["feat", "fix", "docs", "style", "refactor", "test", "chore"];
const DEFAULT_MODEL_REPO: &str = "marzoukbaig14/committed-gguf-0.6b";
const DEFAULT_MODEL_FILE: &str = "committed-0.6b-finetuned-Q4_K_M.gguf";

// Constrains generation to `type(scope)?: subject` so the commit line is
// well-formed by construction. `type` is limited to COMMIT_TYPES, which is
// narrower than the committed model's trained codebook (it also knows
// perf/build/ci) — the grammar resolves that mismatch at decode time.
const COMMIT_GRAMMAR: &str = r#"
root        ::= type scope? ": " description
type        ::= "feat" | "fix" | "docs" | "style" | "refactor" | "test" | "chore"
scope       ::= "(" [a-zA-Z0-9_./-]+ ")"
description ::= [^ \t\n.] ([^\n]* [^ \t\n.])?
"#;

/// Prompt style chosen by the model filename. `committed-*` GGUFs are the
/// QLoRA fine-tunes trained on `Diff:\n{diff}` with a fixed system instruction,
/// so they get their training recipe (ChatML + `/no_think`); everything else
/// gets the generic plain-text prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptKind {
    Committed,
    Legacy,
}

fn prompt_kind_for(model_path: &Path) -> PromptKind {
    model_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            if name.contains("committed") {
                PromptKind::Committed
            } else {
                PromptKind::Legacy
            }
        })
        .unwrap_or(PromptKind::Legacy)
}

const COMMITTED_SYSTEM_INSTRUCTION: &str = r#"You generate a single Conventional Commits subject line from a unified diff.
Output exactly one line of the form "<type>(<scope>): <description>" — scope optional — and nothing else: no prose, no code fences, no quotes.
Choose the type by what the change is, not by which files it touches:
feat — adds a capability; fix — corrects a bug; docs — documentation only; style — formatting with no change in logic; refactor — restructures code without changing behavior; perf — improves performance; test — adds or fixes tests; build — build system or dependencies; ci — CI configuration; chore — maintenance touching neither source nor tests.
Add a scope in parentheses only when a single file or area clearly owns the change; if the change is spread out or the owner is unclear, omit it.
Write the description so that:
- It reads correctly after "If applied, this commit will…" — imperative verb first ("add", never "adds" or "added").
- It states only what the diff shows. You can see what changed, not why, so never invent a reason, motivation, or outcome the diff doesn't contain; when unsure, say less rather than guess.
- It names the most significant change when the diff touches several things.
- It is specific: name the real function, file, flag, or endpoint, and skip filler verbs ("update", "change") and vague objects ("code", "stuff") when something precise fits."#;
/// Fallback when hardware probing is unavailable.
const MIN_CONTEXT_SIZE: i32 = 2048;
const MAX_CONTEXT_SIZE: i32 = 32768;

/// Model quality/size tiers for any-device deployments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelTier {
    /// ~0.6B — low-RAM / older machines.
    Small,
    /// ~1.7B — balanced default for most machines.
    Default,
    /// ~4B — higher quality when memory allows.
    Quality,
}

impl ModelTier {
    fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "small" | "s" | "low" | "0.6b" | "0.5b" | "tiny" => Some(Self::Small),
            "default" | "d" | "medium" | "med" | "1.7b" | "1.5b" | "balanced" => {
                Some(Self::Default)
            }
            "quality" | "q" | "high" | "4b" | "3b" | "large" => Some(Self::Quality),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Default => "default",
            Self::Quality => "quality",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Small => "small (Qwen3-0.6B)",
            Self::Default => "default (Qwen3-1.7B)",
            Self::Quality => "quality (Qwen3-4B)",
        }
    }

    fn repo_id(self) -> &'static str {
        match self {
            // Official Qwen3 GGUF repos (post-trained chat models; prefer Q4_K_M at download).
            Self::Small => "Qwen/Qwen3-0.6B-GGUF",
            Self::Default => "Qwen/Qwen3-1.7B-GGUF",
            Self::Quality => "Qwen/Qwen3-4B-GGUF",
        }
    }

    /// Recommended llama.cpp context length for this tier.
    fn recommended_context(self) -> i32 {
        match self {
            Self::Small => 4096,
            Self::Default => 8192,
            Self::Quality => 16384,
        }
    }

    /// Approximate Q4 weights size in MiB (for messaging / headroom checks).
    fn approx_weight_mib(self) -> u64 {
        match self {
            Self::Small => 450,
            Self::Default => 1200,
            Self::Quality => 2500,
        }
    }

    /// Pick a tier from total system RAM (MiB).
    fn from_total_ram_mib(total_ram_mib: u64) -> Self {
        if total_ram_mib < 8 * 1024 {
            Self::Small
        } else if total_ram_mib < 16 * 1024 {
            Self::Default
        } else {
            Self::Quality
        }
    }

    fn all() -> [Self; 3] {
        [Self::Small, Self::Default, Self::Quality]
    }
}

#[derive(Debug, Clone, Copy)]
struct HardwareProfile {
    total_ram_mib: Option<u64>,
    recommended_tier: ModelTier,
    recommended_context: i32,
}

impl HardwareProfile {
    fn detect() -> Self {
        let total_ram_mib = detect_total_memory_mib();
        let recommended_tier = total_ram_mib
            .map(ModelTier::from_total_ram_mib)
            .unwrap_or(ModelTier::Default);
        let recommended_context =
            clamp_context_size(recommended_tier.recommended_context(), total_ram_mib);
        Self {
            total_ram_mib,
            recommended_tier,
            recommended_context,
        }
    }
}

fn clamp_context_size(requested: i32, total_ram_mib: Option<u64>) -> i32 {
    let mut ctx = requested.clamp(MIN_CONTEXT_SIZE, MAX_CONTEXT_SIZE);
    if let Some(ram) = total_ram_mib {
        // Keep headroom for OS + weights + KV cache (very rough heuristic).
        let max_by_ram = if ram < 6 * 1024 {
            4096
        } else if ram < 12 * 1024 {
            8192
        } else if ram < 24 * 1024 {
            16384
        } else {
            MAX_CONTEXT_SIZE
        };
        ctx = ctx.min(max_by_ram);
    }
    ctx
}

fn detect_total_memory_mib() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let bytes: u64 = text.trim().parse().ok()?;
        Some(bytes / (1024 * 1024))
    }

    #[cfg(target_os = "linux")]
    {
        let contents = fs::read_to_string("/proc/meminfo").ok()?;
        for line in contents.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                let kib: u64 = rest
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse().ok())?;
                return Some(kib / 1024);
            }
        }
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
            ])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let bytes: u64 = text.trim().parse().ok()?;
        return Some(bytes / (1024 * 1024));
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

fn resolve_context_size(git_config: &GitConfig, hardware: &HardwareProfile) -> i32 {
    if let Ok(raw) = git_config.get(CONFIG_CONTEXT_KEY) {
        if let Ok(parsed) = raw.trim().parse::<i32>() {
            return clamp_context_size(parsed, hardware.total_ram_mib);
        }
    }
    hardware.recommended_context
}

fn resolve_model_tier(git_config: &GitConfig, hardware: &HardwareProfile) -> ModelTier {
    if let Ok(raw) = git_config.get(CONFIG_MODEL_TIER_KEY) {
        if let Some(tier) = ModelTier::from_str(&raw) {
            return tier;
        }
    }
    hardware.recommended_tier
}

struct Language;

impl Language {
    fn generating_commit_message(&self) -> &'static str {
        "Generating commit message..."
    }

    fn this_may_take_moment(&self) -> &'static str {
        "This may take a moment depending on your model and system..."
    }

    fn processing_response(&self) -> &'static str {
        "Processing response..."
    }

    fn commit_message_generated(&self) -> &'static str {
        "\n\nCommit message generated."
    }

    fn invalid_selection(&self) -> &'static str {
        "Invalid selection. Please try again."
    }

    fn fetching_models(&self) -> &'static str {
        "Searching for local GGUF models..."
    }

    fn available_models(&self) -> &'static str {
        "\nDetected GGUF models:"
    }

    fn model_set_as_default(&self) -> &'static str {
        "Model path ready: {}"
    }

    fn no_default_model(&self) -> &'static str {
        "No model path available. Please select a GGUF file."
    }

    fn no_changes_staged(&self) -> &'static str {
        "No changes staged for commit."
    }

    fn use_edit_cancel_prompt(&self) -> &'static str {
        "\nDo you want to (u)se this message, (e)dit it, or (c)ancel? [u/e/c]: "
    }

    fn enter_commit_message(&self) -> &'static str {
        "Enter your commit message (use multiple lines if needed, end with an empty line):\n"
    }

    fn commit_cancelled(&self) -> &'static str {
        "Commit cancelled."
    }

    fn invalid_choice(&self) -> &'static str {
        "Invalid choice. Please try again."
    }

    fn enter_name_prompt(&self) -> &'static str {
        "Enter your name: "
    }

    fn enter_email_prompt(&self) -> &'static str {
        "Enter your email: "
    }

    fn changes_committed(&self) -> &'static str {
        "\nChanges committed successfully."
    }

    fn commit_message_label(&self) -> &'static str {
        "Commit message:\n{}"
    }

    fn model_retrying_invalid_output(&self) -> &'static str {
        "Model response was invalid. Retrying with stricter instructions..."
    }

    fn model_failed_generate(&self) -> &'static str {
        "Model could not produce a valid commit message. Please enter one manually."
    }

    fn fallback_commit_generated(&self) -> &'static str {
        "\n\nGenerated a fallback commit message."
    }

    fn truncated_diff_notice(&self) -> &'static str {
        "[Diff truncated to reduce context size.]"
    }

    fn changed_files_heading(&self) -> &'static str {
        "Changed files:"
    }

    fn file_omitted_notice(&self) -> &'static str {
        "(content omitted)"
    }

    fn file_snippet_heading(&self) -> &'static str {
        "File:"
    }

    fn no_models_found(&self) -> &'static str {
        "No GGUF models found in default locations. Download a model first or provide its path manually."
    }

    fn enter_model_path_hint(&self) -> &'static str {
        "Hint: place models under ./models or ~/Library/Application Support/git-ca/models (macOS) or ~/.cache/git-ca/models."
    }

    fn model_file_missing(&self) -> &'static str {
        "Model file missing: {}"
    }

    fn model_extension_warning(&self) -> &'static str {
        "The file must have a .gguf extension."
    }

    fn download_model_prompt(&self) -> &'static str {
        "Download a GGUF model (for example from https://huggingface.co/collections/ggml-org/gguf) and retry."
    }

    fn downloading_model(&self) -> &'static str {
        "Downloading model '{}' from Hugging Face..."
    }

    fn download_completed(&self) -> &'static str {
        "Model downloaded to: {}"
    }

    fn auto_downloading_default(&self) -> &'static str {
        "No local models found. Downloading default model '{}'..."
    }

    fn model_pull_hint(&self) -> &'static str {
        "Tip: run 'git ca model pull [small|default|quality|<repo>]' to download a tier or custom GGUF."
    }

    fn hardware_profile_label(&self) -> &'static str {
        "Hardware profile:"
    }

    fn recommended_tier_label(&self) -> &'static str {
        "Recommended model tier: {}"
    }

    fn using_context_label(&self) -> &'static str {
        "Using context length: {} tokens"
    }

    fn auto_selected_tier(&self) -> &'static str {
        "Auto-selected model tier '{}' based on system memory."
    }

    fn tier_persisted(&self) -> &'static str {
        "Model tier preference saved: {}"
    }

    fn available_tiers(&self) -> &'static str {
        "Model tiers:"
    }

    fn select_tier_prompt(&self) -> &'static str {
        "\nSelect a tier number, enter a GGUF path, or press Enter for the recommended tier: "
    }

    fn key_changes_heading(&self) -> &'static str {
        "Key changes:"
    }

    fn additional_snippets_heading(&self) -> &'static str {
        "Additional snippets:"
    }

    fn hierarchical_budget_notice(&self) -> &'static str {
        "[Lower-priority diff details omitted to fit context.]"
    }

    fn not_in_git_repository(&self) -> &'static str {
        "Not in a git repository"
    }
}

#[derive(Debug)]
enum AppError {
    Git(git2::Error),
    Io(io::Error),
    InputClosed,
    Custom(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Git(e) => write!(f, "Git error: {e}"),
            AppError::Io(e) => write!(f, "IO error: {e}"),
            AppError::InputClosed => write!(f, "Input stream closed"),
            AppError::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<git2::Error> for AppError {
    fn from(err: git2::Error) -> Self {
        AppError::Git(err)
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<hf_hub::api::sync::ApiError> for AppError {
    fn from(err: hf_hub::api::sync::ApiError) -> Self {
        AppError::Custom(format!("Hugging Face API error: {err}"))
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Custom(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Custom(msg.to_string())
    }
}

type Result<T> = std::result::Result<T, AppError>;

fn debug_model_response(label: &str, response: &str) {
    eprintln!("\n[git-ca] {label}\n~~~~\n{response}\n~~~~");
}

fn find_git_repository(start_path: &Path) -> Option<PathBuf> {
    let mut current_path = start_path.to_path_buf();
    loop {
        if current_path.join(".git").is_dir() {
            return Some(current_path);
        }
        if !current_path.pop() {
            return None;
        }
    }
}

fn get_diff() -> Result<String> {
    let output = Command::new("git").args(["diff", "--cached"]).output()?;
    let diff = String::from_utf8(output.stdout)
        .map_err(|e| AppError::Custom(format!("Invalid UTF-8 in diff: {e}")))?;
    Ok(diff)
}

fn build_commit_prompt(diff: &str, attempt: usize, kind: PromptKind) -> String {
    if kind == PromptKind::Committed {
        return format!(
            "<|im_start|>system\n{COMMITTED_SYSTEM_INSTRUCTION}<|im_end|>\n<|im_start|>user\nDiff:\n{diff}\n\n/no_think<|im_end|>\n<|im_start|>assistant\n"
        );
    }

    {
        // `/no_think` disables Qwen3 thinking mode so the model returns only the commit line.
        let mut prompt = format!(
            r#"/no_think
SYSTEM: You are a commit message generator. You must output ONLY a commit message, nothing else.
Do not reason out loud. Do not emit <think> blocks or chain-of-thought.

TASK: Analyze the git diff below and produce exactly ONE commit message in Conventional Commits format.

FORMAT: <type>(<scope>): <subject>

EXAMPLES:
- feat(api): add user authentication endpoint
- fix(cli): resolve model loading timeout
- docs: update installation instructions
- refactor(llama): simplify token sampling logic
- chore(deps): update dependencies
- test: add unit tests for diff parsing

RULES:
1. <type> MUST be one of: feat, fix, docs, style, refactor, test, chore
2. <scope> is optional, use kebab-case when needed (e.g., cli, api, docs)
3. <subject> is imperative, concise (<= 72 chars)
4. NO explanations, NO markdown fences, NO extra text, NO thinking
5. Output ONLY the commit message, nothing else

HERE IS THE DIFF:
{diff}

YOUR OUTPUT (commit message only):"#
        );

        if attempt > 0 {
            prompt.push_str(
                "\n\n/no_think\nCRITICAL: Previous output was invalid. You MUST output ONLY a commit message starting with '<type>(<scope>): <subject>'. NO other text, explanations, thinking, or formatting.",
            );
        }

        prompt
    }
}

fn analyze_diff(
    diff: &str,
    model_path: &Path,
    language: &Language,
    context_size: i32,
) -> Result<Option<String>> {
    println!("{}", language.generating_commit_message());
    eprintln!("\x1b[90m{}\x1b[0m", language.this_may_take_moment());

    let mut session = LlamaSession::new(model_path, context_size).map_err(AppError::from)?;
    const MAX_ATTEMPTS: usize = 2;

    let prompt_kind = prompt_kind_for(model_path);
    let diff_variants = build_diff_variants(diff, language, context_size, prompt_kind);
    let grammar = (prompt_kind == PromptKind::Committed).then_some(COMMIT_GRAMMAR);

    for attempt in 0..MAX_ATTEMPTS {
        let fragment = diff_variants
            .get(attempt)
            .or_else(|| diff_variants.last())
            .unwrap();
        let prompt = build_commit_prompt(fragment, attempt, prompt_kind);
        let response = match session.infer(&prompt, 256, grammar) {
            Ok(output) => output,
            Err(err) => {
                eprintln!("{err}");
                if attempt + 1 < MAX_ATTEMPTS {
                    println!("{}", language.model_retrying_invalid_output());
                    continue;
                } else {
                    println!("{}", language.model_failed_generate());
                    return Ok(None);
                }
            }
        };

        println!("{}", language.processing_response());

        if let Some(processed) = process_model_response(&response) {
            if is_valid_commit_message(&processed) {
                println!("{processed}");
                println!("{}", language.commit_message_generated());
                return Ok(Some(processed));
            } else {
                debug_model_response("model output failed validation", &response);
            }
        } else {
            debug_model_response("model output did not contain a commit subject", &response);
        }

        if attempt + 1 < MAX_ATTEMPTS {
            println!("{}", language.model_retrying_invalid_output());
        }
    }

    Ok(None)
}

fn process_model_response(response: &str) -> Option<String> {
    let response_without_thinking = if response.trim_start().starts_with("<think>") {
        response
            .find("</think>")
            .map(|end_index| response[(end_index + "</think>".len())..].trim_start())
            .unwrap_or(response)
    } else {
        response
    };

    let lines: Vec<&str> = response_without_thinking
        .lines()
        .filter(|line| !line.starts_with("Fixes #") && !line.starts_with("Closes #"))
        .collect();

    if let Some((index, subject_line)) = lines.iter().enumerate().find_map(|(i, line)| {
        let trimmed = line.trim();
        if is_commit_subject(trimmed) {
            Some((i, trimmed.to_string()))
        } else {
            None
        }
    }) {
        let mut message_lines = vec![subject_line];
        let mut j = index + 1;

        while j < lines.len() {
            let trimmed = lines[j].trim();

            if trimmed.is_empty() {
                let mut k = j + 1;
                let mut next_non_empty: Option<&str> = None;
                while k < lines.len() {
                    let candidate = lines[k].trim();
                    if !candidate.is_empty() {
                        next_non_empty = Some(candidate);
                        break;
                    }
                    k += 1;
                }

                if let Some(next_line) = next_non_empty {
                    if is_commit_subject(next_line) || looks_like_instruction(next_line) {
                        break;
                    }
                } else {
                    break;
                }

                if !message_lines.last().map(|s| s.is_empty()).unwrap_or(false) {
                    message_lines.push(String::new());
                }
            } else if is_commit_subject(trimmed) || looks_like_instruction(trimmed) {
                break;
            } else {
                message_lines.push(trimmed.to_string());
            }

            j += 1;
        }

        let message = message_lines.join("\n").trim().to_string();
        if !message.is_empty() {
            return Some(message);
        }
    }

    None
}

fn is_commit_subject(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }

    let lower = line.to_ascii_lowercase();
    COMMIT_TYPES.iter().any(|commit_type| {
        if !lower.starts_with(commit_type) || lower.len() <= commit_type.len() {
            return false;
        }

        matches!(
            lower.as_bytes().get(commit_type.len()),
            Some(b'(') | Some(b':')
        )
    })
}

fn looks_like_instruction(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }

    let lower = line.to_ascii_lowercase();
    const KEYWORDS: &[&str] = &[
        "your task:",
        "your task is",
        "your response",
        "respond with",
        "return only",
        "remember:",
        "guidelines:",
        "rules:",
        "important:",
        "ensure your response",
        "ensure that your response",
        "make sure your response",
        "do not include any",
        "do not add any",
        "commit message content must",
        "the commit message must",
    ];

    KEYWORDS.iter().any(|keyword| lower.contains(keyword))
}

#[derive(Default)]
struct DiffSummary {
    files: Vec<String>,
    scope_candidates: Vec<String>,
    has_docs: bool,
    has_code: bool,
    docs_only: bool,
    has_main: bool,
    has_llama: bool,
    has_retry: bool,
    has_kv_reset: bool,
    new_files: HashSet<String>,
    has_cargo_toml: bool,
    has_cargo_lock: bool,
    has_node_manifest: bool,
    has_node_lock: bool,
}

impl DiffSummary {
    fn has_docs_only(&self) -> bool {
        self.has_docs && !self.has_code && self.docs_only
    }
}

fn analyze_diff_summary(diff: &str) -> DiffSummary {
    let mut summary = DiffSummary {
        docs_only: true,
        ..Default::default()
    };

    let mut seen_files = HashSet::new();
    let mut current_file: Option<String> = None;

    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            let mut parts = rest.split_whitespace();
            let _a = parts.next();
            let b = parts.next();
            if let Some(b) = b {
                let path = b.strip_prefix("b/").unwrap_or(b).to_string();
                current_file = Some(path.clone());

                if seen_files.insert(path.clone()) {
                    summary.files.push(path.clone());

                    let scope = path_to_scope(&path);
                    if !scope.is_empty() && !summary.scope_candidates.contains(&scope) {
                        summary.scope_candidates.push(scope);
                    }

                    let ext = path.rsplit('.').next().unwrap_or("");
                    let is_doc = matches!(ext, "md" | "rst" | "adoc" | "txt");
                    if is_doc {
                        summary.has_docs = true;
                    } else {
                        summary.docs_only = false;
                    }
                    // Dependency manifests/locks are not treated as application code for type inference.
                    let is_deps_path = is_dependency_manifest_or_lock(&path);
                    if is_code_extension(ext) && !is_deps_path {
                        summary.has_code = true;
                    }

                    if path == "src/main.rs" {
                        summary.has_main = true;
                    }
                    if path == "src/llama.rs" {
                        summary.has_llama = true;
                    }
                    if path == "Cargo.toml" || path.ends_with("/Cargo.toml") {
                        summary.has_cargo_toml = true;
                        summary.docs_only = false;
                    }
                    if path == "Cargo.lock" || path.ends_with("/Cargo.lock") {
                        summary.has_cargo_lock = true;
                        summary.docs_only = false;
                    }
                    if path.ends_with("package.json") {
                        summary.has_node_manifest = true;
                        summary.docs_only = false;
                    }
                    if path.contains("pnpm-lock")
                        || path.contains("package-lock")
                        || path.contains("yarn.lock")
                    {
                        summary.has_node_lock = true;
                        summary.docs_only = false;
                    }
                }
            }
        } else if line.starts_with("new file mode") {
            if let Some(file) = current_file.clone() {
                summary.new_files.insert(file);
            }
        } else if line.starts_with('+') {
            let lower = line.to_ascii_lowercase();
            if lower.contains("retry") || lower.contains("stricter instructions") {
                summary.has_retry = true;
            }
            if lower.contains("kv_self_clear") || lower.contains("kv cache") {
                summary.has_kv_reset = true;
            }
        }
    }

    summary
}

#[derive(Default)]
struct FileSection {
    path: String,
    additions: usize,
    deletions: usize,
    /// L1: signatures, hunk headers, high-signal added/removed lines.
    key_lines: Vec<String>,
    /// L2: remaining diff body for secondary packing.
    extra_lines: Vec<String>,
    omitted: bool,
    is_new: bool,
}

fn prompt_diff_char_budget(context_size: i32) -> usize {
    // Reserve tokens for the system prompt wrapper + short generation.
    (context_size as usize)
        .saturating_sub(384)
        .saturating_mul(3)
        .max(2048)
}

fn is_code_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "py"
            | "go"
            | "java"
            | "kt"
            | "kts"
            | "swift"
            | "c"
            | "cc"
            | "cpp"
            | "cxx"
            | "h"
            | "hpp"
            | "cs"
            | "rb"
            | "php"
            | "vue"
            | "svelte"
            | "scala"
            | "rsx"
            | "zig"
            | "lua"
            | "sh"
            | "bash"
            | "zsh"
            | "sql"
            | "gradle"
            | "dart"
            | "r"
            | "jl"
    )
}

fn is_dependency_manifest_or_lock(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with("package.json")
        || lower.ends_with("package-lock.json")
        || lower.contains("pnpm-lock")
        || lower.contains("yarn.lock")
        || lower.ends_with("cargo.toml")
        || lower.ends_with("cargo.lock")
        || lower.ends_with("go.mod")
        || lower.ends_with("go.sum")
        || lower.ends_with("composer.json")
        || lower.ends_with("composer.lock")
        || lower.ends_with("pyproject.toml")
        || lower.ends_with("poetry.lock")
        || lower.ends_with("requirements.txt")
        || lower.ends_with("gemfile")
        || lower.ends_with("gemfile.lock")
}

fn is_key_diff_line(line: &str) -> bool {
    if line.starts_with("@@") {
        return true;
    }

    let is_change = (line.starts_with('+') && !line.starts_with("+++"))
        || (line.starts_with('-') && !line.starts_with("---"));
    if !is_change {
        return false;
    }

    let content = line.get(1..).unwrap_or("").trim();
    if content.is_empty() {
        return false;
    }

    // Skip pure noise / imports-only churn for L1 packing priority.
    if content.starts_with("//")
        || content.starts_with('#')
        || content.starts_with("/*")
        || content.starts_with('*')
        || content.starts_with("import ")
        || content.starts_with("from ")
        || content.starts_with("use ")
        || content.starts_with("package ")
    {
        return false;
    }

    let lower = content.to_ascii_lowercase();
    lower.contains("fn ")
        || lower.contains("function ")
        || lower.contains("def ")
        || lower.contains("class ")
        || lower.contains("struct ")
        || lower.contains("interface ")
        || lower.contains("impl ")
        || lower.contains("export ")
        || lower.contains("pub ")
        || lower.starts_with("func ")
        || lower.contains(" type ")
        || lower.starts_with("type ")
        || lower.contains("async ")
        || lower.contains("const ")
        || lower.contains("let ")
        || lower.contains("var ")
        || lower.contains("enum ")
        || lower.contains("trait ")
        || lower.contains("mod ")
        || lower.contains("return ")
        || lower.contains("throw ")
        || lower.contains("error")
        || lower.contains("fix")
        || lower.contains("todo")
        || lower.contains("config")
        || lower.contains("route")
        || lower.contains("api")
}

/// Hierarchical diff packing:
/// - L0: file inventory (+/- counts)
/// - L1: key signatures / high-signal hunks
/// - L2: extra snippets from highest-churn files when budget remains
fn build_diff_summary(diff: &str, language: &Language, context_size: i32) -> String {
    build_hierarchical_diff_summary(diff, language, context_size, true)
}

fn build_hierarchical_diff_summary(
    diff: &str,
    language: &Language,
    context_size: i32,
    include_l2: bool,
) -> String {
    const PER_FILE_KEY_LINES: usize = 24;
    const PER_FILE_EXTRA_LINES: usize = 80;
    const PER_FILE_KEY_CHARS: usize = 900;
    const PER_FILE_EXTRA_CHARS: usize = 1600;

    let max_chars = prompt_diff_char_budget(context_size);
    let mut sections = parse_diff_sections(diff);

    if sections.is_empty() {
        return diff.chars().take(diff.len().min(max_chars)).collect();
    }

    // Prefer high-churn, non-generated files when packing L1/L2.
    sections.sort_by(|a, b| {
        let score = |s: &FileSection| {
            if s.omitted {
                0usize
            } else {
                s.additions
                    .saturating_add(s.deletions)
                    .saturating_add(usize::from(s.is_new) * 3)
            }
        };
        score(b).cmp(&score(a))
    });

    // L0 — always emit a compact inventory first.
    let mut output = String::new();
    output.push_str(language.changed_files_heading());
    output.push('\n');
    for section in &sections {
        let mut notes = Vec::new();
        if section.is_new {
            notes.push("new");
        }
        if section.omitted {
            notes.push(language.file_omitted_notice());
        }
        let note = if notes.is_empty() {
            String::new()
        } else {
            format!(" ({})", notes.join(", "))
        };
        output.push_str(&format!(
            "- {} (+{} / -{}){}\n",
            section.path, section.additions, section.deletions, note
        ));
    }
    output.push('\n');

    let mut remaining = max_chars.saturating_sub(output.len());
    let mut omitted_details = false;

    // L1 — key changes.
    let mut l1_body = String::new();
    for section in &sections {
        if section.omitted || section.key_lines.is_empty() {
            continue;
        }
        if remaining < 64 {
            omitted_details = true;
            break;
        }

        let mut block = format!("{} {}\n", language.file_snippet_heading(), section.path);
        let mut used_chars = 0usize;
        let mut used_lines = 0usize;
        for line in &section.key_lines {
            if used_lines >= PER_FILE_KEY_LINES || used_chars + line.len() + 1 > PER_FILE_KEY_CHARS
            {
                break;
            }
            if block.len() + line.len() + 1 > remaining {
                omitted_details = true;
                break;
            }
            block.push_str(line);
            block.push('\n');
            used_chars += line.len() + 1;
            used_lines += 1;
        }
        if used_lines == 0 {
            omitted_details = true;
            continue;
        }
        block.push('\n');
        if block.len() > remaining {
            omitted_details = true;
            break;
        }
        remaining = remaining.saturating_sub(block.len());
        l1_body.push_str(&block);
    }

    if !l1_body.is_empty() {
        output.push_str(language.key_changes_heading());
        output.push('\n');
        output.push_str(&l1_body);
        remaining = max_chars.saturating_sub(output.len());
    }

    // L2 — additional body from top churn files.
    if include_l2 && remaining > 128 {
        let mut l2_body = String::new();
        for section in &sections {
            if section.omitted || section.extra_lines.is_empty() {
                continue;
            }
            if remaining < 64 {
                omitted_details = true;
                break;
            }

            let mut block = format!("{} {}\n", language.file_snippet_heading(), section.path);
            let mut used_chars = 0usize;
            let mut used_lines = 0usize;
            for line in &section.extra_lines {
                if used_lines >= PER_FILE_EXTRA_LINES
                    || used_chars + line.len() + 1 > PER_FILE_EXTRA_CHARS
                {
                    break;
                }
                if block.len() + line.len() + 1 > remaining {
                    omitted_details = true;
                    break;
                }
                block.push_str(line);
                block.push('\n');
                used_chars += line.len() + 1;
                used_lines += 1;
            }
            if used_lines == 0 {
                continue;
            }
            block.push('\n');
            if block.len() > remaining {
                omitted_details = true;
                break;
            }
            remaining = remaining.saturating_sub(block.len());
            l2_body.push_str(&block);
        }

        if !l2_body.is_empty() {
            output.push_str(language.additional_snippets_heading());
            output.push('\n');
            output.push_str(&l2_body);
        }
    }

    if omitted_details || diff.len() > max_chars {
        output.push_str(language.hierarchical_budget_notice());
        output.push('\n');
    }

    output
}

fn parse_diff_sections(diff: &str) -> Vec<FileSection> {
    let mut sections: Vec<FileSection> = Vec::new();
    let mut current: Option<FileSection> = None;

    for line in diff.lines() {
        if let Some(path) = line
            .strip_prefix("diff --git ")
            .and_then(|rest| rest.split_whitespace().nth(1))
            .map(|b_path| b_path.strip_prefix("b/").unwrap_or(b_path).to_string())
        {
            if let Some(section) = current.take() {
                sections.push(section);
            }
            let mut section = FileSection {
                path,
                ..Default::default()
            };
            if is_generated_or_large_file(&section.path) {
                section.omitted = true;
            }
            current = Some(section);
            continue;
        }

        let Some(section) = current.as_mut() else {
            continue;
        };

        if line.starts_with("new file mode") {
            section.is_new = true;
            continue;
        }

        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }

        if line.starts_with('+') && !line.starts_with("+++") {
            section.additions += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            section.deletions += 1;
        }

        if section.omitted {
            continue;
        }

        if is_key_diff_line(line) {
            if section.key_lines.len() < 40 {
                section.key_lines.push(line.to_string());
            } else if section.extra_lines.len() < 120 {
                section.extra_lines.push(line.to_string());
            }
        } else if ((line.starts_with('+') && !line.starts_with("+++"))
            || (line.starts_with('-') && !line.starts_with("---"))
            || line.starts_with(' '))
            && section.extra_lines.len() < 120
        {
            section.extra_lines.push(line.to_string());
        }
    }

    if let Some(section) = current.take() {
        sections.push(section);
    }

    sections
}

fn is_generated_or_large_file(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("pnpm-lock")
        || lower.contains("package-lock")
        || lower.contains("yarn.lock")
        || lower.contains("cargo.lock")
        || lower.contains("composer.lock")
        || lower.contains("poetry.lock")
        || lower.contains("go.sum")
        || lower.ends_with(".min.js")
        || lower.ends_with(".min.css")
        || lower.ends_with(".map")
        || lower.ends_with(".lock")
}

fn build_diff_raw_tail(diff: &str, language: &Language, context_size: i32) -> String {
    let max_chars = prompt_diff_char_budget(context_size);

    if diff.len() <= max_chars {
        return diff.to_string();
    }

    let mut chars: Vec<char> = diff.chars().collect();
    if chars.len() > max_chars {
        chars.drain(0..chars.len() - max_chars);
    }

    let mut trimmed: String = chars.into_iter().collect();
    if let Some(pos) = trimmed.find("diff --git ") {
        trimmed = trimmed[pos..].to_string();
    }

    format!("{}\n\n{}", language.truncated_diff_notice(), trimmed)
}

fn build_diff_raw_head(diff: &str, language: &Language, context_size: i32) -> String {
    let max_chars = prompt_diff_char_budget(context_size);

    if diff.len() <= max_chars {
        return diff.to_string();
    }

    let mut trimmed: String = diff.chars().take(max_chars).collect();
    if let Some(pos) = trimmed.rfind("diff --git ") {
        trimmed = trimmed[..pos].to_string();
    }

    format!("{trimmed}\n\n{}", language.truncated_diff_notice())
}

fn build_diff_variants(
    diff: &str,
    language: &Language,
    context_size: i32,
    kind: PromptKind,
) -> Vec<String> {
    if kind == PromptKind::Committed {
        // committed was trained on near-raw single-file diffs — feed the raw
        // head first, fall back to the hierarchical summary on retry.
        let raw = build_diff_raw_head(diff, language, context_size);
        let full = build_diff_summary(diff, language, context_size);
        if raw.trim() == full.trim() {
            vec![raw]
        } else {
            vec![raw, full]
        }
    } else {
        // Attempt 0: full hierarchical summary (L0+L1+L2).
        let full = build_diff_summary(diff, language, context_size);
        // Attempt 1: tighter view (L0+L1 only) for stricter retries.
        let tight = build_hierarchical_diff_summary(diff, language, context_size, false);
        if full.trim() == tight.trim() {
            let raw = build_diff_raw_tail(diff, language, context_size);
            if full.trim() == raw.trim() {
                vec![full]
            } else {
                vec![full, raw]
            }
        } else {
            vec![full, tight]
        }
    }
}

fn path_to_scope(path: &str) -> String {
    let mut trimmed = path.trim_start_matches("./");
    if trimmed.starts_with("a/") || trimmed.starts_with("b/") {
        trimmed = &trimmed[2..];
    }
    if trimmed.is_empty() {
        return String::new();
    }
    let mut parts = trimmed.split('/');
    let first = parts.next().unwrap_or(trimmed);
    let candidate = if first == "src" {
        parts.next().unwrap_or(first)
    } else {
        first
    };
    let candidate = candidate.split('.').next().unwrap_or(candidate);
    slugify(candidate)
}

fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn humanize_slug(slug: &str) -> String {
    if slug.eq_ignore_ascii_case("cli") {
        return "CLI".to_string();
    }
    if slug.eq_ignore_ascii_case("kv") {
        return "KV".to_string();
    }
    if slug.eq_ignore_ascii_case("deps") {
        return "Dependencies".to_string();
    }
    let parts: Vec<String> = slug
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    chars.as_str().to_ascii_lowercase()
                )
            } else {
                String::new()
            }
        })
        .collect();
    if parts.is_empty() {
        "Project".to_string()
    } else {
        parts.join(" ")
    }
}

enum SubjectTemplate {
    StabilizeCommitGeneration,
    SyncDocsAndCode,
    UpdateDocs,
    IntroduceScope,
    RefineScope,
    UpdateScope,
    UpdateDeps,
}

fn build_subject(template: SubjectTemplate, scope: &str) -> String {
    match template {
        SubjectTemplate::StabilizeCommitGeneration => {
            "stabilize commit message generation".to_string()
        }
        SubjectTemplate::UpdateDeps => "update dependencies".to_string(),
        SubjectTemplate::SyncDocsAndCode => "align docs and code changes".to_string(),
        SubjectTemplate::UpdateDocs => {
            format!("update {} documentation", scope)
        }
        SubjectTemplate::IntroduceScope => format!("add {}", scope),
        SubjectTemplate::RefineScope => format!("refine {}", scope),
        SubjectTemplate::UpdateScope => format!("update {}", scope),
    }
}

fn build_scope_readable(scopes: &[String]) -> String {
    if scopes.is_empty() {
        return "project".to_string();
    }

    let words: Vec<String> = scopes.iter().map(|slug| humanize_slug(slug)).collect();
    match words.len() {
        1 => words[0].clone(),
        2 => format!("{} and {}", words[0], words[1]),
        _ => format!("{} and more", words[0]),
    }
}

fn build_scope_slug(scopes: &[String]) -> String {
    if scopes.is_empty() {
        return String::new();
    }
    if scopes.iter().any(|s| s == "deps") {
        return "deps".to_string();
    }
    if scopes.iter().any(|s| s == "docs") && scopes.len() == 1 {
        return "docs".to_string();
    }
    scopes.iter().take(2).cloned().collect::<Vec<_>>().join("-")
}

fn compute_scopes(summary: &DiffSummary) -> Vec<String> {
    fn push_unique(scopes: &mut Vec<String>, value: &str) {
        if !scopes.iter().any(|s| s == value) {
            scopes.push(value.to_string());
        }
    }

    let mut scopes = Vec::new();

    if summary.has_main {
        push_unique(&mut scopes, "cli");
    }
    if summary.has_llama {
        push_unique(&mut scopes, "llama");
    }
    if summary.has_docs_only() {
        push_unique(&mut scopes, "docs");
    }
    if summary.has_cargo_toml || summary.has_cargo_lock {
        push_unique(&mut scopes, "deps");
    }
    if summary.has_node_manifest || summary.has_node_lock {
        push_unique(&mut scopes, "deps");
    }

    for candidate in &summary.scope_candidates {
        if scopes.len() >= 3 {
            break;
        }
        push_unique(&mut scopes, candidate);
    }

    if scopes.is_empty() {
        push_unique(&mut scopes, "project");
    }

    scopes
}

fn generate_fallback_commit_message(diff: &str) -> Option<String> {
    let summary = analyze_diff_summary(diff);
    if summary.files.is_empty() {
        return None;
    }

    let mut scopes = compute_scopes(&summary);

    let has_deps_change = summary.has_cargo_lock
        || summary.has_cargo_toml
        || summary.has_node_lock
        || summary.has_node_manifest;
    let has_runtime_change = summary.has_main || summary.has_llama;

    let (commit_type, template) = if summary.has_retry || summary.has_kv_reset {
        ("fix", SubjectTemplate::StabilizeCommitGeneration)
    } else if has_runtime_change {
        ("fix", SubjectTemplate::RefineScope)
    } else if summary.has_docs && summary.has_code {
        ("fix", SubjectTemplate::SyncDocsAndCode)
    } else if summary.has_docs_only() {
        ("docs", SubjectTemplate::UpdateDocs)
    } else if has_deps_change && !summary.has_code {
        ("chore", SubjectTemplate::UpdateDeps)
    } else if summary.has_code {
        if !summary.new_files.is_empty() {
            ("feat", SubjectTemplate::IntroduceScope)
        } else {
            ("refactor", SubjectTemplate::RefineScope)
        }
    } else {
        ("chore", SubjectTemplate::UpdateScope)
    };

    if commit_type == "chore" && matches!(template, SubjectTemplate::UpdateDeps) {
        scopes.clear();
        scopes.push("deps".to_string());
    }

    let scope_slug = build_scope_slug(&scopes);
    let scope_readable = build_scope_readable(&scopes);
    let subject = build_subject(template, &scope_readable);

    Some(if scope_slug.is_empty() {
        format!("{commit_type}: {subject}")
    } else {
        format!("{commit_type}({scope_slug}): {subject}")
    })
}
fn is_valid_commit_message(message: &str) -> bool {
    let subject_line = message
        .lines()
        .map(|line| line.trim())
        .find(|line| !line.is_empty());

    let subject_line = match subject_line {
        Some(line) => line,
        None => return false,
    };

    if parse_commit_subject(subject_line).is_none() {
        return false;
    }

    subject_line.is_ascii()
}

fn parse_commit_subject(line: &str) -> Option<(&'static str, Option<&str>, &str)> {
    for commit_type in COMMIT_TYPES {
        if let Some(rest) = line.strip_prefix(commit_type) {
            if rest.starts_with('(') {
                let end = rest.find("):")?;
                let scope = rest[1..end].trim();
                if scope.is_empty() {
                    return None;
                }
                let subject = rest[end + 2..].trim();
                if subject.is_empty() {
                    return None;
                }
                return Some((commit_type, Some(scope), subject));
            } else if let Some(stripped) = rest.strip_prefix(':') {
                let subject = stripped.trim();
                if subject.is_empty() {
                    return None;
                }
                return Some((commit_type, None, subject));
            }
        }
    }
    None
}

fn get_user_input(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    let bytes = io::stdin().read_line(&mut input)?;
    if bytes == 0 {
        return Err(AppError::InputClosed);
    }
    Ok(input.trim().to_string())
}

struct GitConfig {
    config: Config,
}

impl GitConfig {
    fn new() -> Result<Self> {
        Ok(Self {
            config: Config::open_default()?,
        })
    }

    fn get(&self, key: &str) -> Result<String> {
        Ok(self.config.get_string(key)?)
    }

    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        Ok(self.config.set_str(key, value)?)
    }

    fn get_or_prompt(&mut self, key: &str, prompt: &str) -> Result<String> {
        match self.get(key) {
            Ok(value) => Ok(value),
            Err(_) => {
                let value = get_user_input(prompt)?;
                self.set(key, &value)?;
                Ok(value)
            }
        }
    }
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        env::var("HOME").ok().map(PathBuf::from)
    }
}

fn default_model_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(current) = env::current_dir() {
        dirs.push(current.join("models"));
    }

    if let Some(home) = home_dir() {
        dirs.push(home.join(".cache/git-ca/models"));
        dirs.push(home.join(".cache/git-ca"));
        dirs.push(home.join(".local/share/git-ca/models"));
        dirs.push(home.join("Library/Application Support/git-ca/models"));
    }

    dirs
}

fn models_root_dir() -> Result<PathBuf> {
    if let Some(home) = home_dir() {
        Ok(home.join(".cache/git-ca/models"))
    } else {
        Ok(env::current_dir()?.join("models"))
    }
}

fn model_record_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(home) = home_dir() {
        candidates.push(home.join(".cache/git-ca/default-model.path"));
    }
    if let Ok(current) = env::current_dir() {
        candidates.push(current.join(".git-ca/default-model.path"));
    }
    candidates
}

fn load_persisted_model_path() -> Option<String> {
    for record in model_record_candidates() {
        if !record.is_file() {
            continue;
        }
        match fs::read_to_string(&record) {
            Ok(contents) => {
                let trimmed = contents.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
            Err(err) => {
                eprintln!(
                    "[git-ca] warning: could not read persisted model path ({}): {err}",
                    record.display()
                );
            }
        }
    }
    None
}

fn persist_model_path(path: &Path) {
    let mut last_error: Option<String> = None;
    let serialized = path.to_string_lossy();
    for record in model_record_candidates() {
        if let Some(parent) = record.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                last_error = Some(err.to_string());
                continue;
            }
        }
        match fs::write(&record, serialized.as_ref()) {
            Ok(_) => return,
            Err(err) => {
                last_error = Some(err.to_string());
            }
        }
    }
    if let Some(err) = last_error {
        eprintln!(
            "[git-ca] warning: could not persist model path ({}): {err}",
            path.display()
        );
    }
}

fn clear_persisted_model_path() {
    for record in model_record_candidates() {
        if record.is_file() {
            if let Err(err) = fs::remove_file(&record) {
                eprintln!(
                    "[git-ca] warning: could not clear cached model path ({}): {err}",
                    record.display()
                );
            }
        }
    }
}

fn is_gguf(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("gguf"))
        .unwrap_or(false)
}

fn expand_model_path(input: &str) -> PathBuf {
    let trimmed = input.trim();

    if trimmed == "~" {
        if let Some(home) = home_dir() {
            return home;
        }
    }

    if let Some(stripped) = trimmed.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(stripped);
        }
    }

    if let Some(stripped) = trimmed.strip_prefix("~\\") {
        if let Some(home) = home_dir() {
            return home.join(stripped);
        }
    }

    PathBuf::from(trimmed)
}

fn find_local_models() -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut found = Vec::new();

    for dir in default_model_dirs() {
        if !dir.is_dir() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && is_gguf(&path) && seen.insert(path.clone()) {
                    found.push(path);
                }
            }
        }
    }

    found.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    found
}

fn pick_gguf_filename<'a>(siblings: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let mut fallback: Option<&str> = None;
    let mut q4: Option<&str> = None;
    let mut q4_k_m: Option<&str> = None;

    for name in siblings {
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".gguf") {
            continue;
        }
        // Prefer instruction / it variants when multiple GGUFs exist.
        let is_instruct =
            lower.contains("instruct") || lower.contains("-it-") || lower.contains("_it_");
        if fallback.is_none() || is_instruct {
            fallback = Some(name);
        }
        if lower.contains("q4_k_m") {
            q4_k_m = Some(name);
            if is_instruct {
                break;
            }
        } else if q4.is_none() && lower.contains("q4") {
            q4 = Some(name);
        }
    }

    q4_k_m.or(q4).or(fallback)
}

fn download_model_from_hub(
    repo_id: &str,
    language: &Language,
    preferred_file: Option<&str>,
) -> Result<PathBuf> {
    let api = Api::new()
        .map_err(|e| AppError::Custom(format!("Failed to initialize Hugging Face client: {e}")))?;
    let repo = api.model(repo_id.to_string());
    let info = repo.info().map_err(|e| {
        AppError::Custom(format!(
            "Failed to fetch repository '{repo_id}' metadata: {e}"
        ))
    })?;

    let filename = match preferred_file {
        // The default repo ships a baseline GGUF alongside the fine-tuned one —
        // pin the exact file so the heuristic can't pick the wrong sibling.
        Some(want) => info
            .siblings
            .iter()
            .map(|s| s.rfilename.as_str())
            .find(|name| *name == want)
            .ok_or_else(|| {
                AppError::Custom(format!(
                    "Expected file '{want}' not found in repository '{repo_id}'"
                ))
            })?,
        None => pick_gguf_filename(info.siblings.iter().map(|s| s.rfilename.as_str())).ok_or_else(
            || AppError::Custom(format!("No GGUF files found in repository '{repo_id}'")),
        )?,
    };

    println!("{}", language.downloading_model().replace("{}", repo_id));
    let source_path = repo.get(filename).map_err(|e| {
        AppError::Custom(format!(
            "Failed to download '{}' from '{}': {e}",
            filename, repo_id
        ))
    })?;

    let dest_dir = models_root_dir()?;
    fs::create_dir_all(&dest_dir)?;

    let base_name = Path::new(filename)
        .file_name()
        .and_then(|os| os.to_str())
        .unwrap_or(filename);
    let sanitized_repo = repo_id.replace(['/', '\\'], "__");
    let dest_file_name = format!("{sanitized_repo}__{base_name}");
    let dest_path = dest_dir.join(dest_file_name);

    if !dest_path.exists() {
        fs::copy(&source_path, &dest_path)?;
    }

    let canonical = fs::canonicalize(&dest_path).unwrap_or(dest_path.clone());
    println!(
        "{}",
        language
            .download_completed()
            .replace("{}", &canonical.to_string_lossy())
    );
    Ok(canonical)
}

fn print_hardware_profile(language: &Language, hardware: &HardwareProfile) {
    println!("{}", language.hardware_profile_label());
    match hardware.total_ram_mib {
        Some(mib) => {
            let gib = mib as f64 / 1024.0;
            println!("  RAM: ~{gib:.1} GiB ({mib} MiB)");
        }
        None => println!("  RAM: unknown"),
    }
    println!(
        "  {}",
        language
            .recommended_tier_label()
            .replace("{}", hardware.recommended_tier.display_name())
    );
    println!(
        "  {}",
        language
            .using_context_label()
            .replace("{}", &hardware.recommended_context.to_string())
    );
}

fn ensure_default_model(language: &Language) -> Result<Option<PathBuf>> {
    if find_local_models().is_empty() {
        println!(
            "{}",
            language
                .auto_downloading_default()
                .replace("{}", DEFAULT_MODEL_REPO)
        );
        let downloaded =
            download_model_from_hub(DEFAULT_MODEL_REPO, language, Some(DEFAULT_MODEL_FILE))?;
        let canonical = fs::canonicalize(&downloaded).unwrap_or(downloaded);
        persist_model_path(&canonical);
        println!(
            "{}",
            language
                .model_set_as_default()
                .replace("{}", &canonical.to_string_lossy())
        );
        return Ok(Some(canonical));
    }

    Ok(None)
}

fn get_model_path(language: &Language, tier: ModelTier) -> Result<PathBuf> {
    if let Some(stored) = load_persisted_model_path() {
        let expanded = expand_model_path(&stored);
        if expanded.is_file() && is_gguf(&expanded) {
            let canonical = fs::canonicalize(&expanded).unwrap_or(expanded);
            println!(
                "{}",
                language
                    .model_set_as_default()
                    .replace("{}", &canonical.to_string_lossy())
            );
            return Ok(canonical);
        } else {
            println!(
                "{}",
                language
                    .model_file_missing()
                    .replace("{}", &expanded.to_string_lossy())
            );
            clear_persisted_model_path();
        }
    }

    if let Some(downloaded) = ensure_default_model(language)? {
        return Ok(downloaded);
    }

    let models = find_local_models();
    if models.is_empty() {
        println!("{}", language.no_default_model());
        println!("{}", language.model_pull_hint());
        return select_model_path(language, tier);
    }

    if models.len() == 1 {
        let canonical = fs::canonicalize(&models[0]).unwrap_or_else(|_| models[0].clone());
        persist_model_path(&canonical);
        println!(
            "{}",
            language
                .model_set_as_default()
                .replace("{}", &canonical.to_string_lossy())
        );
        return Ok(canonical);
    }

    select_model_path(language, tier)
}

fn select_model_path(language: &Language, recommended_tier: ModelTier) -> Result<PathBuf> {
    println!("{}", language.fetching_models());
    print_hardware_profile(language, &HardwareProfile::detect());

    println!("\n{}", language.available_tiers());
    for (idx, tier) in ModelTier::all().iter().enumerate() {
        let marker = if *tier == recommended_tier {
            " ← recommended"
        } else {
            ""
        };
        println!(
            "  {}. {} — {} (~{} MiB Q4){marker}",
            idx + 1,
            tier.as_str(),
            tier.repo_id(),
            tier.approx_weight_mib()
        );
    }

    let models = find_local_models();
    if models.is_empty() {
        println!("{}", language.no_models_found());
        println!("{}", language.model_pull_hint());
    } else {
        println!("{}", language.available_models());
        for (i, model) in models.iter().enumerate() {
            println!("  L{}. {}", i + 1, model.display());
        }
    }

    if !io::stdin().is_terminal() {
        // Non-interactive: prefer an existing local model, else download recommended tier.
        if let Some(first) = models.first() {
            let canonical = fs::canonicalize(first).unwrap_or_else(|_| first.clone());
            persist_model_path(&canonical);
            println!(
                "{}",
                language
                    .model_set_as_default()
                    .replace("{}", &canonical.to_string_lossy())
            );
            return Ok(canonical);
        }
        let downloaded = download_model_from_hub(recommended_tier.repo_id(), language, None)?;
        let canonical = fs::canonicalize(&downloaded).unwrap_or(downloaded);
        persist_model_path(&canonical);
        return Ok(canonical);
    }

    println!("{}", language.enter_model_path_hint());

    loop {
        let input = match get_user_input(language.select_tier_prompt()) {
            Ok(value) => value,
            Err(AppError::InputClosed) => {
                if let Some(first) = models.first() {
                    let canonical = fs::canonicalize(first).unwrap_or_else(|_| first.clone());
                    persist_model_path(&canonical);
                    println!(
                        "{}",
                        language
                            .model_set_as_default()
                            .replace("{}", &canonical.to_string_lossy())
                    );
                    return Ok(canonical);
                }
                return Err(AppError::InputClosed);
            }
            Err(err) => return Err(err),
        };
        let trimmed = input.trim();

        // Empty / Enter → download recommended tier.
        if trimmed.is_empty() {
            let downloaded = download_model_from_hub(recommended_tier.repo_id(), language, None)?;
            let canonical = fs::canonicalize(&downloaded).unwrap_or(downloaded);
            persist_model_path(&canonical);
            println!(
                "{}",
                language
                    .model_set_as_default()
                    .replace("{}", &canonical.to_string_lossy())
            );
            return Ok(canonical);
        }

        // Tier by name or 1–3.
        if let Some(tier) = ModelTier::from_str(trimmed) {
            let downloaded = download_model_from_hub(tier.repo_id(), language, None)?;
            let canonical = fs::canonicalize(&downloaded).unwrap_or(downloaded);
            persist_model_path(&canonical);
            println!(
                "{}",
                language
                    .model_set_as_default()
                    .replace("{}", &canonical.to_string_lossy())
            );
            return Ok(canonical);
        }
        if let Ok(index) = trimmed.parse::<usize>() {
            if (1..=3).contains(&index) {
                let tier = ModelTier::all()[index - 1];
                let downloaded = download_model_from_hub(tier.repo_id(), language, None)?;
                let canonical = fs::canonicalize(&downloaded).unwrap_or(downloaded);
                persist_model_path(&canonical);
                println!(
                    "{}",
                    language
                        .model_set_as_default()
                        .replace("{}", &canonical.to_string_lossy())
                );
                return Ok(canonical);
            }
        }

        // Local model: "L1", "l2", or bare index only when it matches local list uniquely after tiers.
        let local_index = trimmed
            .strip_prefix('L')
            .or_else(|| trimmed.strip_prefix('l'))
            .and_then(|rest| rest.parse::<usize>().ok());
        if let Some(index) = local_index {
            if index > 0 && index <= models.len() {
                let selected = fs::canonicalize(&models[index - 1])
                    .unwrap_or_else(|_| models[index - 1].clone());
                persist_model_path(&selected);
                println!(
                    "{}",
                    language
                        .model_set_as_default()
                        .replace("{}", &selected.to_string_lossy())
                );
                return Ok(selected);
            }
            println!("{}", language.invalid_selection());
            continue;
        }

        let candidate = expand_model_path(trimmed);
        if !is_gguf(&candidate) {
            println!("{}", language.model_extension_warning());
            continue;
        }
        if !candidate.is_file() {
            println!(
                "{}",
                language
                    .model_file_missing()
                    .replace("{}", &candidate.to_string_lossy())
            );
            println!("{}", language.download_model_prompt());
            continue;
        }

        let canonical = fs::canonicalize(&candidate).unwrap_or(candidate);
        persist_model_path(&canonical);
        println!(
            "{}",
            language
                .model_set_as_default()
                .replace("{}", &canonical.to_string_lossy())
        );
        return Ok(canonical);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_extracts_subject_line() {
        let response = "Processing response...\nThe commit message content must be written in English.\n\nfeat(cli): improve diff summary\n";
        assert_eq!(
            process_model_response(response),
            Some("feat(cli): improve diff summary".to_string())
        );
    }

    #[test]
    fn handles_includes_body_until_instruction() {
        let response = "feat(cli): improve diff summary\n\nAdd staged file summary for clarity.\nGuidelines: avoid printing instructions.\n";
        assert_eq!(
            process_model_response(response),
            Some(
                "feat(cli): improve diff summary\n\nAdd staged file summary for clarity."
                    .to_string()
            )
        );
    }

    #[test]
    fn handles_instruction_only_fallback() {
        let response = "The commit message content must be written in English.";
        assert_eq!(process_model_response(response), None);
    }

    #[test]
    fn validates_conventional_commit_subject() {
        assert!(is_valid_commit_message("feat(cli): improve prompts"));
        assert!(is_valid_commit_message("docs: update contribution guide"));
    }

    #[test]
    fn rejects_invalid_commit_messages() {
        assert!(!is_valid_commit_message("Implement new feature"));
        assert!(!is_valid_commit_message("feat(): missing subject"));
        assert!(!is_valid_commit_message("feat(cli) missing colon"));
        assert!(!is_valid_commit_message("docs: 更新贡献指南"));
    }

    #[test]
    fn fallback_generates_for_retry_flow() {
        let diff = "\
diff --git a/src/main.rs b/src/main.rs
index 1111111..2222222 100644
--- a/src/main.rs
+++ b/src/main.rs
@@
+ println!(\"Model response was invalid. Retrying with stricter instructions...\");
";
        let message = generate_fallback_commit_message(diff).expect("fallback");
        assert!(message.starts_with("fix("));
        assert!(message.contains("stabilize commit message generation"));
    }

    #[test]
    fn fallback_generates_for_docs_only() {
        let diff = "\
diff --git a/AGENTS.md b/AGENTS.md
new file mode 100644
index 0000000..3333333
--- /dev/null
+++ b/AGENTS.md
@@
+# Repository Guidelines
";
        let message = generate_fallback_commit_message(diff).expect("fallback docs");
        assert!(message.starts_with("docs("));
        assert!(message.contains("documentation"));
    }

    #[test]
    fn fallback_prefers_runtime_scope() {
        let diff = "\
diff --git a/src/main.rs b/src/main.rs
index 1111111..2222222 100644
--- a/src/main.rs
+++ b/src/main.rs
@@
+ println!(\"Processing response...\");
diff --git a/src/llama.rs b/src/llama.rs
new file mode 100644
index 0000000..3333333
--- /dev/null
+++ b/src/llama.rs
@@
+ llama_kv_self_clear(ctx);
";
        let message = generate_fallback_commit_message(diff).expect("fallback runtime");
        assert!(message.starts_with("fix("));
        assert!(
            message.contains("stabilize commit message generation") || message.contains("refine")
        );
    }

    #[test]
    fn fallback_handles_dependency_updates() {
        let diff = concat!(
            "diff --git a/package.json b/package.json\n",
            "index 1111111..2222222 100644\n",
            "--- a/package.json\n",
            "+++ b/package.json\n",
            "@@\n",
            "+  \"llama-kit\": \"^2.0.0\"\n",
            "diff --git a/pnpm-lock.yaml b/pnpm-lock.yaml\n",
            "index 1111111..3333333 100644\n",
            "--- a/pnpm-lock.yaml\n",
            "+++ b/pnpm-lock.yaml\n",
            "@@\n",
            "+packages:\n",
        );
        let message = generate_fallback_commit_message(diff).expect("fallback deps");
        assert_eq!(message, "chore(deps): update dependencies");
    }

    #[test]
    fn truncates_diff_for_prompt() {
        let language = Language;
        let long_diff = format!(
            "diff --git a/file b/file\n--- a/file\n+++ b/file\n@@\n+{}\n",
            "a".repeat(5000)
        );
        let prepared = build_diff_summary(&long_diff, &language, 512);
        assert!(
            prepared.contains(language.hierarchical_budget_notice())
                || prepared.contains(language.truncated_diff_notice())
                || prepared.contains(language.changed_files_heading())
        );
        assert!(prepared.len() < long_diff.len());
    }

    #[test]
    fn hierarchical_summary_includes_l0_inventory() {
        let language = Language;
        let diff = "\
diff --git a/src/api.ts b/src/api.ts
--- a/src/api.ts
+++ b/src/api.ts
@@
+export function login() { return true; }
diff --git a/src/ui.tsx b/src/ui.tsx
--- a/src/ui.tsx
+++ b/src/ui.tsx
@@
+export const Button = () => null;
";
        let summary = build_diff_summary(diff, &language, 4096);
        assert!(summary.contains(language.changed_files_heading()));
        assert!(summary.contains("src/api.ts"));
        assert!(summary.contains("src/ui.tsx"));
        assert!(summary.contains(language.key_changes_heading()) || summary.contains("export"));
    }

    #[test]
    fn hierarchical_retry_variant_can_drop_l2() {
        let language = Language;
        let mut body = String::new();
        for i in 0..40 {
            body.push_str(&format!("+const value_{i} = {i};\n"));
        }
        let diff = format!(
            "diff --git a/src/lib.ts b/src/lib.ts\n--- a/src/lib.ts\n+++ b/src/lib.ts\n@@\n+export function main() {{}}\n{body}"
        );
        let variants = build_diff_variants(&diff, &language, 2048, PromptKind::Legacy);
        assert!(!variants.is_empty());
        assert!(variants[0].contains(language.changed_files_heading()));
    }

    #[test]
    fn model_tier_from_ram_thresholds() {
        assert_eq!(ModelTier::from_total_ram_mib(4 * 1024), ModelTier::Small);
        assert_eq!(ModelTier::from_total_ram_mib(8 * 1024), ModelTier::Default);
        assert_eq!(ModelTier::from_total_ram_mib(32 * 1024), ModelTier::Quality);
    }

    #[test]
    fn model_tier_parses_aliases() {
        assert_eq!(ModelTier::from_str("small"), Some(ModelTier::Small));
        assert_eq!(ModelTier::from_str("DEFAULT"), Some(ModelTier::Default));
        assert_eq!(ModelTier::from_str("quality"), Some(ModelTier::Quality));
        assert_eq!(ModelTier::from_str("0.6b"), Some(ModelTier::Small));
        assert_eq!(ModelTier::from_str("1.7b"), Some(ModelTier::Default));
        assert_eq!(ModelTier::from_str("4b"), Some(ModelTier::Quality));
        assert_eq!(ModelTier::from_str("nope"), None);
    }

    #[test]
    fn model_tier_uses_qwen3_repos() {
        assert_eq!(ModelTier::Small.repo_id(), "Qwen/Qwen3-0.6B-GGUF");
        assert_eq!(ModelTier::Default.repo_id(), "Qwen/Qwen3-1.7B-GGUF");
        assert_eq!(ModelTier::Quality.repo_id(), "Qwen/Qwen3-4B-GGUF");
    }

    #[test]
    fn commit_prompt_disables_qwen3_thinking() {
        let prompt = build_commit_prompt("diff --git a/x b/x\n", 0, PromptKind::Legacy);
        assert!(prompt.starts_with("/no_think") || prompt.contains("/no_think"));
        assert!(prompt.contains("NO thinking") || prompt.contains("Do not emit <think>"));

        let retry = build_commit_prompt("diff --git a/x b/x\n", 1, PromptKind::Legacy);
        assert!(retry.matches("/no_think").count() >= 2);

        let committed = build_commit_prompt("diff --git a/x b/x\n", 0, PromptKind::Committed);
        assert!(committed.contains("<|im_start|>system"));
        assert!(committed.contains("/no_think"));
        assert!(committed.contains("<|im_start|>assistant\n"));
    }

    #[test]
    fn detects_code_extensions_beyond_rust() {
        assert!(is_code_extension("ts"));
        assert!(is_code_extension("py"));
        assert!(is_code_extension("go"));
        assert!(!is_code_extension("md"));
    }

    #[test]
    fn key_diff_line_prefers_signatures() {
        assert!(is_key_diff_line("@@ -1,2 +1,3 @@"));
        assert!(is_key_diff_line("+export function login() {}"));
        assert!(is_key_diff_line("+pub fn analyze_diff() {}"));
        assert!(!is_key_diff_line("+import foo from 'bar';"));
        assert!(!is_key_diff_line(" context only"));
    }

    #[test]
    fn pick_gguf_prefers_q4_k_m() {
        let names = [
            "model-q8_0.gguf",
            "model-q4_0.gguf",
            "model-q4_k_m.gguf",
            "readme.md",
        ];
        assert_eq!(
            pick_gguf_filename(names.into_iter()),
            Some("model-q4_k_m.gguf")
        );
    }

    #[test]
    fn clamp_context_respects_low_ram() {
        assert_eq!(clamp_context_size(16384, Some(4 * 1024)), 4096);
        assert_eq!(clamp_context_size(16384, Some(10 * 1024)), 8192);
        assert!(clamp_context_size(8192, Some(32 * 1024)) >= 8192);
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("git-ca version {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut git_config = GitConfig::new()?;
    let language = Language;
    let hardware = HardwareProfile::detect();
    let model_tier = resolve_model_tier(&git_config, &hardware);
    let context_size = resolve_context_size(&git_config, &hardware);

    if args.len() > 1 {
        match args[1].as_str() {
            "doctor" => {
                run_doctor(&language, &hardware, model_tier, context_size)?;
                return Ok(());
            }
            "model" => {
                if args.len() > 2 && args[2] == "pull" {
                    let (repo_id, tier_for_config) = match args.get(3).map(|s| s.as_str()) {
                        None => {
                            // Auto-select from hardware.
                            println!(
                                "{}",
                                language
                                    .auto_selected_tier()
                                    .replace("{}", hardware.recommended_tier.display_name())
                            );
                            (
                                hardware.recommended_tier.repo_id().to_string(),
                                Some(hardware.recommended_tier),
                            )
                        }
                        Some(arg) => {
                            if let Some(tier) = ModelTier::from_str(arg) {
                                (tier.repo_id().to_string(), Some(tier))
                            } else {
                                (arg.to_string(), None)
                            }
                        }
                    };
                    let downloaded = download_model_from_hub(&repo_id, &language, None)?;
                    persist_model_path(&downloaded);
                    if let Some(tier) = tier_for_config {
                        let _ = git_config.set(CONFIG_MODEL_TIER_KEY, tier.as_str());
                        println!(
                            "{}",
                            language.tier_persisted().replace("{}", tier.display_name())
                        );
                    }
                    println!(
                        "{}",
                        language
                            .model_set_as_default()
                            .replace("{}", &downloaded.to_string_lossy())
                    );
                    return Ok(());
                } else {
                    select_model_path(&language, model_tier)?;
                    return Ok(());
                }
            }
            _ => {}
        }
    }

    let model_path = get_model_path(&language, model_tier)?;

    let current_dir = env::current_dir()?;
    let repo_path = find_git_repository(&current_dir)
        .ok_or_else(|| AppError::Custom(language.not_in_git_repository().to_string()))?;

    let repo = Repository::open(&repo_path)?;
    let mut index = repo.index()?;

    env::set_current_dir(&repo_path)?;
    index.read(true)?;

    let diff = get_diff()?;
    if diff.trim().is_empty() {
        println!("{}", language.no_changes_staged());
        return Ok(());
    }

    println!(
        "{}",
        language
            .using_context_label()
            .replace("{}", &context_size.to_string())
    );
    let mut commit_msg = match analyze_diff(&diff, &model_path, &language, context_size)? {
        Some(msg) => msg,
        None => {
            if let Some(fallback) = generate_fallback_commit_message(&diff) {
                println!("{}", language.fallback_commit_generated());
                println!("{fallback}");
                fallback
            } else {
                println!("{}", language.model_failed_generate());
                get_user_input(language.enter_commit_message())?
            }
        }
    };

    if io::stdin().is_terminal() {
        loop {
            let choice = get_user_input(language.use_edit_cancel_prompt())?;

            match choice.to_lowercase().as_str() {
                "u" => break,
                "e" => {
                    commit_msg = get_user_input(language.enter_commit_message())?;
                    break;
                }
                "c" => {
                    println!("{}", language.commit_cancelled());
                    return Ok(());
                }
                _ => println!("{}", language.invalid_choice()),
            }
        }
    } else {
        // Non-interactive mode: automatically use the generated message
        println!("\n[git-ca] Non-interactive mode detected. Using generated commit message.");
    }

    let name = git_config.get_or_prompt("user.name", language.enter_name_prompt())?;
    let email = git_config.get_or_prompt("user.email", language.enter_email_prompt())?;

    let signature = Signature::now(&name, &email)?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let parents = match repo.head() {
        Ok(head) => match head.peel_to_commit() {
            Ok(commit) => vec![commit],
            Err(err) if matches!(err.code(), ErrorCode::NotFound | ErrorCode::UnbornBranch) => {
                Vec::new()
            }
            Err(err) => return Err(err.into()),
        },
        Err(err) if matches!(err.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound) => {
            Vec::new()
        }
        Err(err) => return Err(err.into()),
    };
    let parent_refs: Vec<&Commit> = parents.iter().collect();

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_msg,
        &tree,
        &parent_refs,
    )?;

    println!("{}", language.changes_committed());
    println!(
        "{}",
        language.commit_message_label().replace("{}", &commit_msg)
    );

    Ok(())
}

fn run_doctor(
    language: &Language,
    hardware: &HardwareProfile,
    model_tier: ModelTier,
    context_size: i32,
) -> Result<()> {
    println!("Running llama.cpp smoke test…");
    print_hardware_profile(language, hardware);
    println!(
        "Active model tier: {} ({})",
        model_tier.display_name(),
        model_tier.repo_id()
    );

    let model_path = get_model_path(language, model_tier)?;

    println!("Using model: {}", model_path.to_string_lossy());
    println!(
        "{}",
        language
            .using_context_label()
            .replace("{}", &context_size.to_string())
    );

    let mut session = LlamaSession::new(&model_path, context_size).map_err(AppError::from)?;

    let prompt =
        "You are a helpful assistant. Reply with a short greeting that confirms the model is working, e.g. \"Model ok\".".to_string();

    println!("\nPrompt:\n{}\n", prompt);

    let response = session.infer(&prompt, 64, None).map_err(AppError::from)?;
    println!("Model response:\n{}\n", response.trim());

    Ok(())
}
