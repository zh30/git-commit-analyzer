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
const CONFIG_LANGUAGE_KEY: &str = "commit-analyzer.language";
const COMMIT_TYPES: &[&str] = &["feat", "fix", "docs", "style", "refactor", "test", "chore"];
const DEFAULT_MODEL_REPO: &str = "unsloth/gemma-3-270m-it-GGUF";
const DEFAULT_CONTEXT_SIZE: i32 = 1024;

#[derive(Debug, Clone, PartialEq)]
enum Language {
    English,
    Chinese,
}

impl Language {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(Language::English),
            "zh" | "chinese" | "中文" => Some(Language::Chinese),
            _ => None,
        }
    }

    fn to_string(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Chinese => "zh",
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Chinese => "简体中文",
        }
    }

    fn generating_commit_message(&self) -> &'static str {
        match self {
            Language::English => "Generating commit message...",
            Language::Chinese => "正在生成提交信息...",
        }
    }

    fn this_may_take_moment(&self) -> &'static str {
        match self {
            Language::English => "This may take a moment depending on your model and system...",
            Language::Chinese => "这可能需要一些时间，取决于您的模型和系统配置...",
        }
    }

    fn processing_response(&self) -> &'static str {
        match self {
            Language::English => "Processing response...",
            Language::Chinese => "正在处理响应...",
        }
    }

    fn commit_message_generated(&self) -> &'static str {
        match self {
            Language::English => "\n\nCommit message generated.",
            Language::Chinese => "\n\n提交信息已生成。",
        }
    }

    fn available_languages(&self) -> &'static str {
        match self {
            Language::English => "Available languages:",
            Language::Chinese => "可选语言：",
        }
    }

    fn select_language_prompt(&self) -> &'static str {
        match self {
            Language::English => "\nSelect a language by number: ",
            Language::Chinese => "\n请输入语言编号：",
        }
    }

    fn invalid_selection(&self) -> &'static str {
        match self {
            Language::English => "Invalid selection. Please try again.",
            Language::Chinese => "无效选择，请重试。",
        }
    }

    fn language_set_to(&self) -> &'static str {
        match self {
            Language::English => "Language set to: {}",
            Language::Chinese => "语言已设置为：{}",
        }
    }

    fn fetching_models(&self) -> &'static str {
        match self {
            Language::English => "Searching for local GGUF models...",
            Language::Chinese => "正在搜索本地 GGUF 模型...",
        }
    }

    fn available_models(&self) -> &'static str {
        match self {
            Language::English => "\nDetected GGUF models:",
            Language::Chinese => "\n检测到的 GGUF 模型：",
        }
    }

    fn select_model_prompt(&self) -> &'static str {
        match self {
            Language::English => "\nEnter a model number or provide a full GGUF path: ",
            Language::Chinese => "\n输入模型编号或直接提供 GGUF 文件路径：",
        }
    }

    fn model_set_as_default(&self) -> &'static str {
        match self {
            Language::English => "Model path ready: {}",
            Language::Chinese => "模型路径已就绪：{}",
        }
    }

    fn no_default_model(&self) -> &'static str {
        match self {
            Language::English => "No model path available. Please select a GGUF file.",
            Language::Chinese => "当前没有可用的模型路径，请选择一个 GGUF 文件。",
        }
    }

    fn no_changes_staged(&self) -> &'static str {
        match self {
            Language::English => "No changes staged for commit.",
            Language::Chinese => "没有暂存的更改可提交。",
        }
    }

    fn use_edit_cancel_prompt(&self) -> &'static str {
        match self {
            Language::English => {
                "\nDo you want to (u)se this message, (e)dit it, or (c)ancel? [u/e/c]: "
            }
            Language::Chinese => "\n您想要 (u) 使用此信息，(e) 编辑它，还是 (c) 取消？[u/e/c]：",
        }
    }

    fn enter_commit_message(&self) -> &'static str {
        match self {
            Language::English => "Enter your commit message (use multiple lines if needed, end with an empty line):\n",
            Language::Chinese => "请输入您的提交信息（如需要可使用多行，以空行结束）：\n",
        }
    }

    fn commit_cancelled(&self) -> &'static str {
        match self {
            Language::English => "Commit cancelled.",
            Language::Chinese => "提交已取消。",
        }
    }

    fn invalid_choice(&self) -> &'static str {
        match self {
            Language::English => "Invalid choice. Please try again.",
            Language::Chinese => "无效选择，请重试。",
        }
    }

    fn enter_name_prompt(&self) -> &'static str {
        match self {
            Language::English => "Enter your name: ",
            Language::Chinese => "请输入您的姓名：",
        }
    }

    fn enter_email_prompt(&self) -> &'static str {
        match self {
            Language::English => "Enter your email: ",
            Language::Chinese => "请输入您的邮箱：",
        }
    }

    fn changes_committed(&self) -> &'static str {
        match self {
            Language::English => "\nChanges committed successfully.",
            Language::Chinese => "\n更改已成功提交。",
        }
    }

    fn commit_message_label(&self) -> &'static str {
        match self {
            Language::English => "Commit message:\n{}",
            Language::Chinese => "提交信息：\n{}",
        }
    }

    fn model_retrying_invalid_output(&self) -> &'static str {
        match self {
            Language::English => {
                "Model response was invalid. Retrying with stricter instructions..."
            }
            Language::Chinese => "模型输出无效，正在使用更严格的提示重试...",
        }
    }

    fn model_failed_generate(&self) -> &'static str {
        match self {
            Language::English => {
                "Model could not produce a valid commit message. Please enter one manually."
            }
            Language::Chinese => "模型未能生成有效的提交信息，请手动输入。",
        }
    }

    fn fallback_commit_generated(&self) -> &'static str {
        match self {
            Language::English => "\n\nGenerated a fallback commit message.",
            Language::Chinese => "\n\n已生成备用提交信息。",
        }
    }

    fn truncated_diff_notice(&self) -> &'static str {
        match self {
            Language::English => "[Diff truncated to reduce context size.]",
            Language::Chinese => "[为控制上下文长度，diff 已被截断。]",
        }
    }

    fn changed_files_heading(&self) -> &'static str {
        match self {
            Language::English => "Changed files:",
            Language::Chinese => "变更文件：",
        }
    }

    fn file_omitted_notice(&self) -> &'static str {
        match self {
            Language::English => "(content omitted)",
            Language::Chinese => "（内容已省略）",
        }
    }

    fn file_snippet_heading(&self) -> &'static str {
        match self {
            Language::English => "File:",
            Language::Chinese => "文件：",
        }
    }

    fn truncated_body_notice(&self) -> &'static str {
        match self {
            Language::English => "[Additional hunks truncated]",
            Language::Chinese => "[更多变更已截断]",
        }
    }

    fn no_models_found(&self) -> &'static str {
        match self {
            Language::English => "No GGUF models found in default locations. Download a model first or provide its path manually.",
            Language::Chinese => "在默认位置未找到 GGUF 模型。请先下载模型或手动提供其路径。",
        }
    }

    fn enter_model_path_hint(&self) -> &'static str {
        match self {
            Language::English => "Hint: place models under ./models or ~/Library/Application Support/git-ca/models (macOS) or ~/.cache/git-ca/models.",
            Language::Chinese => "提示：可将模型放在 ./models、~/Library/Application Support/git-ca/models（macOS）或 ~/.cache/git-ca/models 等目录。",
        }
    }

    fn model_file_missing(&self) -> &'static str {
        match self {
            Language::English => "Model file missing: {}",
            Language::Chinese => "模型文件缺失：{}",
        }
    }

    fn model_extension_warning(&self) -> &'static str {
        match self {
            Language::English => "The file must have a .gguf extension.",
            Language::Chinese => "文件必须为 .gguf 扩展名。",
        }
    }

    fn download_model_prompt(&self) -> &'static str {
        match self {
            Language::English => "Download a GGUF model (for example from https://huggingface.co/collections/ggml-org/gguf) and retry.",
            Language::Chinese => "请先下载 GGUF 模型（例如来自 https://huggingface.co/collections/ggml-org/gguf），然后重试。",
        }
    }

    fn downloading_model(&self) -> &'static str {
        match self {
            Language::English => "Downloading model '{}' from Hugging Face...",
            Language::Chinese => "正在从 Hugging Face 下载模型'{}'...",
        }
    }

    fn download_completed(&self) -> &'static str {
        match self {
            Language::English => "Model downloaded to: {}",
            Language::Chinese => "模型已下载至：{}",
        }
    }

    fn auto_downloading_default(&self) -> &'static str {
        match self {
            Language::English => "No local models found. Downloading default model '{}'...",
            Language::Chinese => "未找到本地模型，正在下载默认模型'{}'...",
        }
    }

    fn model_pull_hint(&self) -> &'static str {
        match self {
            Language::English => {
                "Tip: run 'git ca model pull <repo>' to download from Hugging Face."
            }
            Language::Chinese => {
                "提示：运行 'git ca model pull <仓库>' 可从 Hugging Face 下载模型。"
            }
        }
    }

    fn model_pull_usage(&self) -> &'static str {
        match self {
            Language::English => "Usage: git ca model pull <repo>",
            Language::Chinese => "用法：git ca model pull <仓库>",
        }
    }

    fn not_in_git_repository(&self) -> &'static str {
        match self {
            Language::English => "Not in a git repository",
            Language::Chinese => "不在 git 仓库中",
        }
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

fn build_commit_prompt(diff: &str, language: &Language, attempt: usize) -> String {
    match language {
        Language::English => {
            let mut prompt = format!(
                r#"SYSTEM: You are a commit message generator. You must output ONLY a commit message, nothing else.

TASK: Analyze the git diff below and produce exactly ONE commit message in Git Flow format.

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
4. NO explanations, NO markdown fences, NO extra text
5. Output ONLY the commit message, nothing else

HERE IS THE DIFF:
{diff}

YOUR OUTPUT (commit message only):"#
            );

            if attempt > 0 {
                prompt.push_str(
                    "\n\nCRITICAL: Previous output was invalid. You MUST output ONLY a commit message starting with '<type>(<scope>): <subject>'. NO other text, explanations, or formatting.",
                );
            }

            prompt
        }
        Language::Chinese => {
            let mut prompt = format!(
                r#"系统：你是一个提交信息生成器。必须只输出提交信息，不输出其他内容。

任务：分析下面的 git diff，生成一个符合 Git Flow 格式的提交信息。

格式：<类型>(<范围>): <主题>

示例：
- feat(api): 添加用户认证接口
- fix(cli): 解决模型加载超时问题
- docs: 更新安装说明
- refactor(llama): 简化令牌采样逻辑
- chore(deps): 更新依赖包
- test: 添加 diff 解析单元测试

规则：
1. <类型> 必须是：feat、fix、docs、style、refactor、test、chore 之一
2. <范围> 可选，使用 kebab-case（如 cli、api、docs）
3. <主题> 使用祈使语气，简练（≤72 字符）
4. 不输出解释、不使用 markdown、不添加额外文字
5. 只输出提交信息本身，无其他内容

以下是需要分析的 diff：

{diff}

你的输出（仅提交信息）："#
            );

            if attempt > 0 {
                prompt.push_str("\n\n重要：上次输出无效。必须仅输出以 '<类型>(<范围>): <主题>' 开头的提交信息，不要其他文字、解释或格式。");
            }

            prompt
        }
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

    let diff_variants = build_diff_variants(diff, language, context_size);

    for attempt in 0..MAX_ATTEMPTS {
        let fragment = diff_variants
            .get(attempt)
            .or_else(|| diff_variants.last())
            .unwrap();
        let prompt = build_commit_prompt(fragment, language, attempt);
        let response = match session.infer(&prompt, 256) {
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
            if is_valid_commit_message(&processed, language) {
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

        match lower.as_bytes().get(commit_type.len()) {
            Some(b'(') | Some(b':') => true,
            _ => false,
        }
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
        "请仅返回",
        "请只返回",
        "记住：",
        "记住：",
        "请勿包含",
        "回复中只能",
        "请只提供",
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
                    if ext == "rs" {
                        summary.has_code = true;
                    }

                    if path == "src/main.rs" {
                        summary.has_main = true;
                    }
                    if path == "src/llama.rs" {
                        summary.has_llama = true;
                    }
                    if path == "Cargo.toml" {
                        summary.has_cargo_toml = true;
                        summary.docs_only = false;
                    }
                    if path == "Cargo.lock" {
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
    snippet: Vec<String>,
    omitted: bool,
}

fn build_diff_summary(diff: &str, language: &Language, context_size: i32) -> String {
    const SNIPPET_LINE_LIMIT: usize = 120;
    const PER_FILE_SNIPPET_LIMIT: usize = 1200;

    let max_chars = (context_size as usize)
        .saturating_mul(3)
        .saturating_sub(512)
        .max(2048);
    let diff_truncated = diff.len() > max_chars;

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

        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }

        if line.starts_with("@@") {
            if !section.omitted && section.snippet.len() < SNIPPET_LINE_LIMIT {
                section.snippet.push(line.to_string());
            }
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

        let snippet_chars: usize = section.snippet.iter().map(|l| l.len()).sum();
        if section.snippet.len() >= SNIPPET_LINE_LIMIT || snippet_chars >= PER_FILE_SNIPPET_LIMIT {
            section.omitted = true;
            section.snippet.clear();
            continue;
        }

        section.snippet.push(line.to_string());
    }

    if let Some(section) = current.take() {
        sections.push(section);
    }

    if sections.is_empty() {
        return diff
            .chars()
            .take(diff.len().min(max_chars))
            .collect::<String>();
    }

    let mut output = String::new();
    output.push_str(language.changed_files_heading());
    output.push('\n');

    for section in &sections {
        let note = if section.omitted {
            format!(" {}", language.file_omitted_notice())
        } else {
            String::new()
        };
        output.push_str(&format!(
            "- {} (+{} / -{}){}\n",
            section.path, section.additions, section.deletions, note
        ));
    }

    output.push('\n');

    let mut remaining_chars = max_chars.saturating_sub(output.len());

    for section in sections {
        if section.omitted {
            continue;
        }
        if remaining_chars <= 0 {
            output.push_str(language.truncated_diff_notice());
            output.push('\n');
            break;
        }

        output.push_str(language.file_snippet_heading());
        output.push(' ');
        output.push_str(&section.path);
        output.push('\n');

        for line in section.snippet {
            if line.len() + 1 > remaining_chars {
                output.push_str(language.truncated_body_notice());
                output.push('\n');
                remaining_chars = 0;
                break;
            }
            output.push_str(&line);
            output.push('\n');
            remaining_chars = remaining_chars.saturating_sub(line.len() + 1);
        }

        output.push('\n');
    }

    if diff_truncated && !output.contains(language.truncated_diff_notice()) {
        output.push_str(language.truncated_diff_notice());
        output.push('\n');
    }

    output
}

fn is_generated_or_large_file(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("pnpm-lock")
        || lower.contains("package-lock")
        || lower.contains("yarn.lock")
        || lower.contains("cargo.lock")
        || lower.ends_with(".min.js")
        || lower.ends_with(".min.css")
}

fn build_diff_raw_tail(diff: &str, language: &Language, context_size: i32) -> String {
    let max_chars = (context_size as usize)
        .saturating_mul(3)
        .saturating_sub(512)
        .max(2048);

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

fn build_diff_variants(diff: &str, language: &Language, context_size: i32) -> Vec<String> {
    let summary = build_diff_summary(diff, language, context_size);
    let raw = build_diff_raw_tail(diff, language, context_size);
    if summary.trim() == raw.trim() {
        vec![summary]
    } else {
        vec![summary, raw]
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

fn build_subject(language: &Language, template: SubjectTemplate, scope: &str) -> String {
    match (language, template) {
        (Language::English, SubjectTemplate::StabilizeCommitGeneration) => {
            "stabilize commit message generation".to_string()
        }
        (Language::Chinese, SubjectTemplate::StabilizeCommitGeneration) => {
            "稳定提交信息生成流程".to_string()
        }
        (Language::English, SubjectTemplate::UpdateDeps) => "update dependencies".to_string(),
        (Language::Chinese, SubjectTemplate::UpdateDeps) => "更新依赖".to_string(),
        (Language::English, SubjectTemplate::SyncDocsAndCode) => {
            "align docs and code changes".to_string()
        }
        (Language::Chinese, SubjectTemplate::SyncDocsAndCode) => "同步文档与代码更新".to_string(),
        (Language::English, SubjectTemplate::UpdateDocs) => {
            format!("update {} documentation", scope)
        }
        (Language::Chinese, SubjectTemplate::UpdateDocs) => {
            format!("更新{}文档", scope)
        }
        (Language::English, SubjectTemplate::IntroduceScope) => format!("add {}", scope),
        (Language::Chinese, SubjectTemplate::IntroduceScope) => format!("新增{}", scope),
        (Language::English, SubjectTemplate::RefineScope) => format!("refine {}", scope),
        (Language::Chinese, SubjectTemplate::RefineScope) => format!("优化{}", scope),
        (Language::English, SubjectTemplate::UpdateScope) => format!("update {}", scope),
        (Language::Chinese, SubjectTemplate::UpdateScope) => format!("更新{}", scope),
    }
}

fn build_scope_readable(scopes: &[String], language: &Language) -> String {
    if scopes.is_empty() {
        return match language {
            Language::English => "project".to_string(),
            Language::Chinese => "项目".to_string(),
        };
    }

    let words: Vec<String> = scopes.iter().map(|slug| humanize_slug(slug)).collect();
    match (language, words.len()) {
        (Language::English, 1) => words[0].clone(),
        (Language::English, 2) => format!("{} and {}", words[0], words[1]),
        (Language::English, _) => format!("{} and more", words[0]),
        (Language::Chinese, 1) => words[0].clone(),
        (Language::Chinese, 2) => format!("{}和{}", words[0], words[1]),
        (Language::Chinese, _) => format!("{}等", words[0]),
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

fn generate_fallback_commit_message(diff: &str, language: &Language) -> Option<String> {
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
    let scope_readable = build_scope_readable(&scopes, language);
    let subject = build_subject(language, template, &scope_readable);

    Some(if scope_slug.is_empty() {
        format!("{commit_type}: {subject}")
    } else {
        format!("{commit_type}({scope_slug}): {subject}")
    })
}
fn is_valid_commit_message(message: &str, language: &Language) -> bool {
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

    if let Language::English = language {
        if !subject_line.is_ascii() {
            return false;
        }
    }

    true
}

fn parse_commit_subject(line: &str) -> Option<(&'static str, Option<&str>, &str)> {
    for commit_type in COMMIT_TYPES {
        if line.starts_with(commit_type) {
            let rest = &line[commit_type.len()..];
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
            } else if rest.starts_with(':') {
                let subject = rest[1..].trim();
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

fn select_language(git_config: &mut GitConfig) -> Result<Language> {
    let current_lang = get_language(git_config);
    println!("{}", current_lang.available_languages());
    println!("1. English");
    println!("2. 简体中文");

    let choice = loop {
        let input = get_user_input(&current_lang.select_language_prompt())?;
        match input.parse::<usize>() {
            Ok(1) => break Language::English,
            Ok(2) => break Language::Chinese,
            _ => println!("{}", current_lang.invalid_selection()),
        }
    };

    git_config.set(CONFIG_LANGUAGE_KEY, choice.to_string())?;
    println!(
        "{}",
        choice
            .language_set_to()
            .replace("{}", &choice.display_name())
    );
    Ok(choice)
}

fn get_language(git_config: &GitConfig) -> Language {
    git_config
        .get(CONFIG_LANGUAGE_KEY)
        .ok()
        .and_then(|lang| Language::from_str(&lang))
        .unwrap_or(Language::English)
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

fn download_model_from_hub(repo_id: &str, language: &Language) -> Result<PathBuf> {
    let api = Api::new()
        .map_err(|e| AppError::Custom(format!("Failed to initialize Hugging Face client: {e}")))?;
    let repo = api.model(repo_id.to_string());
    let info = repo.info().map_err(|e| {
        AppError::Custom(format!(
            "Failed to fetch repository '{repo_id}' metadata: {e}"
        ))
    })?;

    let mut fallback: Option<&str> = None;
    let mut preferred: Option<&str> = None;

    for sibling in &info.siblings {
        let name = sibling.rfilename.as_str();
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".gguf") {
            continue;
        }

        if fallback.is_none() {
            fallback = Some(name);
        }

        if lower.contains("q4") {
            preferred = Some(name);
            break;
        }
    }

    let filename = preferred.or(fallback).ok_or_else(|| {
        AppError::Custom(format!("No GGUF files found in repository '{repo_id}'"))
    })?;

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

fn ensure_default_model(language: &Language) -> Result<Option<PathBuf>> {
    if find_local_models().is_empty() {
        println!(
            "{}",
            language
                .auto_downloading_default()
                .replace("{}", DEFAULT_MODEL_REPO)
        );
        let downloaded = download_model_from_hub(DEFAULT_MODEL_REPO, language)?;
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

fn get_model_path(language: &Language) -> Result<PathBuf> {
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
        return select_model_path(language);
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

    select_model_path(language)
}

fn select_model_path(language: &Language) -> Result<PathBuf> {
    println!("{}", language.fetching_models());

    let models = find_local_models();
    if models.is_empty() {
        println!("{}", language.no_models_found());
        println!("{}", language.model_pull_hint());
    } else {
        println!("{}", language.available_models());
        for (i, model) in models.iter().enumerate() {
            println!("{}. {}", i + 1, model.display());
        }
    }

    if !io::stdin().is_terminal() {
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
        return Err(AppError::Custom(language.no_models_found().to_string()));
    }

    println!("{}", language.enter_model_path_hint());

    loop {
        let input = match get_user_input(&language.select_model_prompt()) {
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

        if trimmed.is_empty() {
            println!("{}", language.invalid_selection());
            continue;
        }

        if let Ok(index) = trimmed.parse::<usize>() {
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
            } else {
                println!("{}", language.invalid_selection());
                continue;
            }
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
    fn validates_git_flow_subject() {
        assert!(is_valid_commit_message(
            "feat(cli): improve prompts",
            &Language::English
        ));
        assert!(is_valid_commit_message(
            "docs: 更新贡献指南",
            &Language::Chinese
        ));
    }

    #[test]
    fn rejects_invalid_commit_messages() {
        assert!(!is_valid_commit_message(
            "Implement new feature",
            &Language::English
        ));
        assert!(!is_valid_commit_message(
            "feat(): missing subject",
            &Language::English
        ));
        assert!(!is_valid_commit_message(
            "feat(cli) missing colon",
            &Language::English
        ));
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
        let message = generate_fallback_commit_message(diff, &Language::English).expect("fallback");
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
        let message =
            generate_fallback_commit_message(diff, &Language::English).expect("fallback docs");
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
        let message =
            generate_fallback_commit_message(diff, &Language::English).expect("fallback runtime");
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
        let message =
            generate_fallback_commit_message(diff, &Language::English).expect("fallback deps");
        assert_eq!(message, "chore(deps): update dependencies");
    }

    #[test]
    fn truncates_diff_for_prompt() {
        let language = Language::English;
        let long_diff = format!("diff --git a/file b/file\n{}", "a".repeat(5000));
        let prepared = build_diff_summary(&long_diff, &language, 512);
        assert!(prepared.contains(language.truncated_diff_notice()));
        assert!(prepared.len() < long_diff.len());
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("git-ca version {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut git_config = GitConfig::new()?;
    let language = get_language(&git_config);

    if args.len() > 1 {
        match args[1].as_str() {
            "doctor" => {
                run_doctor(&language)?;
                return Ok(());
            }
            "model" => {
                if args.len() > 2 && args[2] == "pull" {
                    if args.len() < 4 {
                        println!("{}", language.model_pull_usage());
                        return Ok(());
                    }
                    let repo_id = &args[3];
                    let downloaded = download_model_from_hub(repo_id, &language)?;
                    persist_model_path(&downloaded);
                    println!(
                        "{}",
                        language
                            .model_set_as_default()
                            .replace("{}", &downloaded.to_string_lossy())
                    );
                    return Ok(());
                } else {
                    select_model_path(&language)?;
                    return Ok(());
                }
            }
            "language" => {
                select_language(&mut git_config)?;
                return Ok(());
            }
            _ => {}
        }
    }

    let model_path = get_model_path(&language)?;

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

    let context_size = DEFAULT_CONTEXT_SIZE;
    let mut commit_msg = match analyze_diff(&diff, &model_path, &language, context_size)? {
        Some(msg) => msg,
        None => {
            if let Some(fallback) = generate_fallback_commit_message(&diff, &language) {
                println!("{}", language.fallback_commit_generated());
                println!("{fallback}");
                fallback
            } else {
                println!("{}", language.model_failed_generate());
                get_user_input(&language.enter_commit_message())?
            }
        }
    };

    if io::stdin().is_terminal() {
        loop {
            let choice = get_user_input(&language.use_edit_cancel_prompt())?;

            match choice.to_lowercase().as_str() {
                "u" => break,
                "e" => {
                    commit_msg = get_user_input(&language.enter_commit_message())?;
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

    let name = git_config.get_or_prompt("user.name", &language.enter_name_prompt())?;
    let email = git_config.get_or_prompt("user.email", &language.enter_email_prompt())?;

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

fn run_doctor(language: &Language) -> Result<()> {
    println!("Running llama.cpp smoke test…");

    let context_size = DEFAULT_CONTEXT_SIZE;
    let model_path = get_model_path(language)?;

    println!("Using model: {}", model_path.to_string_lossy());
    println!("Context length: {}", context_size);

    let mut session = LlamaSession::new(&model_path, context_size).map_err(AppError::from)?;

    let prompt = match language {
        Language::English => {
            "You are a helpful assistant. Reply with a short greeting that confirms the model is working, e.g. \"Model ok\".".to_string()
        }
        Language::Chinese => {
            "你是一个乐于助人的助手。请用简短的话确认模型正常工作，例如“模型正常”。".to_string()
        }
    };

    println!("\nPrompt:\n{}\n", prompt);

    let response = session.infer(&prompt, 64).map_err(AppError::from)?;
    println!("Model response:\n{}\n", response.trim());

    Ok(())
}
