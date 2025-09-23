use git2::{Config, IndexAddOption, Repository, Signature};
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod mlx;

#[cfg(feature = "mlx")]
mod mlx_bridge;

const CONFIG_LANGUAGE_KEY: &str = "commit-analyzer.language";
const COMMIT_TYPES: &[&str] = &["feat", "fix", "docs", "style", "refactor", "test", "chore"];

const DEFAULT_MLX_MODEL: &str = "gemma-3-270m-it-6bit";

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

    #[allow(dead_code)]
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

    fn mlx_dependency_warning(&self) -> &'static str {
        match self {
            Language::English => "Warning: MLX dependency check failed: {}",
            Language::Chinese => "警告：MLX 依赖检查失败：{}",
        }
    }

    #[allow(dead_code)]
    fn ensure_mlx_installed(&self) -> &'static str {
        match self {
            Language::English => {
                "Please ensure Python3 and MLX-LM are installed: pip install mlx-lm"
            }
            Language::Chinese => "请确保已安装 Python3 和 MLX-LM：pip install mlx-lm",
        }
    }

    fn installing_mlx(&self) -> &'static str {
        match self {
            Language::English => "Attempting to install MLX-LM with pip...",
            Language::Chinese => "正在通过 pip 自动安装 MLX-LM...",
        }
    }

    fn mlx_install_success(&self) -> &'static str {
        match self {
            Language::English => "MLX-LM installed successfully.",
            Language::Chinese => "MLX-LM 已成功安装。",
        }
    }

    fn mlx_install_failed(&self) -> &'static str {
        match self {
            Language::English => "Automatic MLX-LM installation failed: {}",
            Language::Chinese => "自动安装 MLX-LM 失败：{}",
        }
    }

    fn python_missing_warning(&self) -> &'static str {
        match self {
            Language::English => {
                "Warning: Python3 not found. Automatic MLX setup requires Python 3."
            }
            Language::Chinese => "警告：未找到 Python3。自动配置 MLX 需要安装 Python 3。",
        }
    }

    fn installing_pip(&self) -> &'static str {
        match self {
            Language::English => "Bootstrapping pip for the current Python environment...",
            Language::Chinese => "正在为当前 Python 环境初始化 pip...",
        }
    }

    #[allow(dead_code)]
    fn pip_missing_warning(&self) -> &'static str {
        match self {
            Language::English => "pip is not available yet: {}",
            Language::Chinese => "pip 尚不可用：{}",
        }
    }

    fn pip_install_failed(&self) -> &'static str {
        match self {
            Language::English => "Unable to prepare pip: {}",
            Language::Chinese => "无法准备 pip：{}",
        }
    }

    #[allow(dead_code)]
    fn no_default_model(&self) -> &'static str {
        match self {
            Language::English => "No default model set. Please select a model.",
            Language::Chinese => "未设置默认模型，请选择一个模型。",
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

    fn mlx_not_configured(&self) -> &'static str {
        match self {
            Language::English => "MLX environment not properly configured. Please ensure Python3 and MLX-LM are installed, then try again.",
            Language::Chinese => "MLX 环境配置不正确。请确保已安装 Python3 和 MLX-LM，然后重试。",
        }
    }

    #[allow(dead_code)]
    fn no_models_found(&self) -> &'static str {
        match self {
            Language::English => {
                "No MLX models available. Please ensure MLX-LM is properly installed."
            }
            Language::Chinese => "未找到可用的 MLX 模型。请确保 MLX-LM 已正确安装。",
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
    Custom(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Git(e) => write!(f, "Git error: {e}"),
            AppError::Io(e) => write!(f, "IO error: {e}"),
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

#[allow(dead_code)]
fn build_commit_prompt(diff: &str, language: &Language) -> String {
    match language {
        Language::English => format!(
            "Analyze this git diff and provide a **single** commit message following the Git Flow format:

<type>(<scope>): <subject>

<body>

Where:
- <type> is one of: feat, fix, docs, style, refactor, test, chore
- <scope> is optional and represents the module affected
- <subject> is a short description in the imperative mood
- <body> provides detailed description (optional)

Important guidelines:
1. Choose only ONE type that best represents the primary purpose of the changes.
2. Summarize ALL changes into a single, concise subject line.
3. Do not include a body or footer in the commit message.
4. Do not mention or reference any issue numbers.
5. Focus solely on the most significant change if there are multiple unrelated changes.
6. **Ensure that only one commit message is generated.**
7. **The commit message content must be written in English language.**
8. **Do not use any other languages except English for the content.**

Here's the diff to analyze:

{diff}

Your task:
1. Analyze the given git diff.
2. **Generate only one** commit message strictly following the Git Flow format described above.
3. Ensure your response contains **ONLY** the formatted commit message, without any additional explanations or markdown.
4. **The commit message content (subject and body) must be written in English.**

Valid response example:
feat(user-auth): implement password reset functionality

Add a new endpoint for password reset requests.
Implement email sending for reset links.

Remember: Your response should only include the English commit message, nothing else."
        ),
        Language::Chinese => format!(
            "分析这个 git diff 并提供一个遵循 Git Flow 格式的提交信息：

<类型>(<范围>): <主题>

<正文>

其中：
- <类型> 是以下之一：feat, fix, docs, style, refactor, test, chore
- <范围> 是可选的，表示受影响的模块
- <主题> 是命令式语气的简短描述
- <正文> 提供详细描述（可选）

重要指导原则：
1. 只选择一个最能代表变更主要目的的类型。
2. 将所有变更总结为一个简洁的主题行。
3. 不要在提交信息中包含正文或脚注。
4. 不要提及或引用任何问题编号。
5. 如果有多个不相关的变更，只关注最重要的变更。
6. **确保只生成一个提交信息。**
7. **提交信息的内容必须使用简体中文，包括主题和正文。**
8. **不允许使用英文，除了 Git Flow 格式的类型关键字（feat、fix、docs 等）。**

以下是要分析的 diff：

{diff}

你的任务：
1. 分析给定的 git diff。
2. **生成一个**严格遵循上述 Git Flow 格式的提交信息。
3. 确保你的回复**只**包含格式化的提交信息，不要有任何额外的解释或 markdown。
4. 提交信息**必须**以 <类型> 开头并遵循所示的确切结构。
5. **提交信息的内容（主题和正文）必须使用简体中文。**

有效回复的示例：
feat(用户认证): 实现密码重置功能

添加密码重置请求的新端点。
实现重置链接的邮件发送。

记住：你的回复应该只包含中文的提交信息，不要有其他内容。"
        )
    }
}

fn analyze_diff(diff: &str, model: &str, language: &Language, python: &Path) -> Result<String> {
    println!("{}", language.generating_commit_message());
    eprintln!("\x1b[90m{}\x1b[0m", language.this_may_take_moment());

    // Build the Python command
    let mut command = Command::new(python);
    command
        .arg("generate_commit.py")
        .arg("--diff")
        .arg(diff)
        .arg("--model")
        .arg(model)
        .arg("--language")
        .arg(language.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| AppError::Custom(format!("Failed to start Python script: {}", e)))?;

    // Capture output in real-time
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Custom("Failed to capture stdout".to_string()))?;

    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::Custom("Failed to capture stderr".to_string()))?;

    use std::sync::{Arc, Mutex};
    use std::thread;

    let output_arc = Arc::new(Mutex::new(Vec::new()));
    let error_output_arc = Arc::new(Mutex::new(Vec::new()));

    // Handle stdout in a separate thread
    let stdout_handle = {
        let output_clone = output_arc.clone();
        thread::spawn(move || {
            use io::Read;
            let mut buffer = [0; 1024];
            loop {
                match stdout.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = &buffer[..n];
                        // Print to console for user feedback
                        io::stdout().write_all(chunk).unwrap();
                        io::stdout().flush().unwrap();
                        if let Ok(mut output) = output_clone.lock() {
                            output.extend_from_slice(chunk);
                        }
                    }
                    Err(_) => break,
                }
            }
        })
    };

    // Handle stderr in a separate thread
    let stderr_handle = {
        let error_output_clone = error_output_arc.clone();
        thread::spawn(move || {
            use io::Read;
            let mut buffer = [0; 1024];
            loop {
                match stderr.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(mut error_output) = error_output_clone.lock() {
                            error_output.extend_from_slice(&buffer[..n]);
                        }
                        // Print stderr to stderr
                        io::stderr().write_all(&buffer[..n]).unwrap();
                    }
                    Err(_) => break,
                }
            }
        })
    };

    // Wait for both threads to complete
    stdout_handle
        .join()
        .map_err(|_| AppError::Custom("Error waiting for stdout thread".to_string()))?;
    stderr_handle
        .join()
        .map_err(|_| AppError::Custom("Error waiting for stderr thread".to_string()))?;

    // Wait for the process to complete
    let status = child
        .wait()
        .map_err(|e| AppError::Custom(format!("Failed to wait for Python process: {}", e)))?;

    if !status.success() {
        let error_msg = if let Ok(error_output) = error_output_arc.lock() {
            if !error_output.is_empty() {
                String::from_utf8_lossy(&error_output).to_string()
            } else {
                format!("Python script exited with status: {}", status)
            }
        } else {
            format!("Python script exited with status: {}", status)
        };
        return Err(AppError::Custom(format!(
            "MLX generation failed: {}",
            error_msg
        )));
    }

    let response = if let Ok(output) = output_arc.lock() {
        String::from_utf8_lossy(&output).to_string()
    } else {
        String::new()
    };
    println!("{}", language.commit_message_generated());
    Ok(process_mlx_response(&response, diff, language))
}

fn process_mlx_response(response: &str, diff: &str, language: &Language) -> String {
    let response_without_thinking = if response.trim_start().starts_with("<think>") {
        response
            .find("</think>")
            .map(|end_index| response[(end_index + "</think>".len())..].trim_start())
            .unwrap_or(response)
    } else {
        response
    };

    let mut best_line: Option<String> = None;

    for raw_line in response_without_thinking.lines() {
        let line = raw_line.trim();
        if line.is_empty()
            || line.starts_with("```")
            || line.starts_with("git diff")
            || line.starts_with("diff --")
            || line.starts_with("@@")
            || line.starts_with("+")
            || line.starts_with("-")
            || line.starts_with("#")
            || line.starts_with("<type>")
            || line.starts_with("<类型>")
        {
            continue;
        }

        if is_valid_commit_line(line) {
            best_line = Some(line.to_string());
            break;
        }
    }

    if let Some(line) = best_line {
        return line;
    }

    fallback_commit_message(diff, language)
}

fn is_valid_commit_line(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }

    let lower = line.to_lowercase();
    if !COMMIT_TYPES.iter().any(|kind| lower.starts_with(kind)) {
        return false;
    }

    lower.contains(':') && !lower.starts_with("```")
}

fn fallback_commit_message(diff: &str, language: &Language) -> String {
    let path = extract_primary_path(diff).unwrap_or_else(|| "project".to_string());
    let commit_type = guess_commit_type(&path);
    let scope = guess_scope(&path);
    let subject = guess_subject(&path, &commit_type, language);

    format_commit_line(&commit_type, scope.as_deref(), &subject)
}

fn extract_primary_path(diff: &str) -> Option<String> {
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            let mut parts = rest.split_whitespace();
            let _ = parts.next()?; // a/path
            let b = parts.next()?; // b/path
            let path = b.trim_start_matches('b').trim_start_matches('/');
            if !path.is_empty() {
                return Some(path.to_string());
            }
        }
    }
    None
}

fn guess_commit_type(path: &str) -> String {
    let lower = path.to_lowercase();
    let commit_type = if lower.ends_with(".md")
        || lower.ends_with(".rst")
        || lower.contains("docs/")
        || lower.contains("/docs")
    {
        "docs"
    } else if lower.contains("test") || lower.contains("spec") {
        "test"
    } else if lower.ends_with(".rs") || lower.ends_with(".py") || lower.ends_with(".ts") {
        "feat"
    } else {
        "chore"
    };

    commit_type.to_string()
}

fn guess_scope(path: &str) -> Option<String> {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        let scope = parent
            .components()
            .next()
            .and_then(|comp| comp.as_os_str().to_str())?
            .replace(['.', ' '], "-");

        if scope.is_empty() || scope == "a" || scope == "b" {
            None
        } else {
            Some(scope)
        }
    } else {
        None
    }
}

fn guess_subject(path: &str, commit_type: &str, language: &Language) -> String {
    let file_name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name);
    let cleaned = stem.replace('-', " ").replace('_', " ").trim().to_string();

    match language {
        Language::Chinese => match commit_type {
            "docs" => format!("更新 {} 文档", cleaned),
            "test" => format!("更新 {} 测试", cleaned),
            "feat" => format!("更新 {} 逻辑", cleaned),
            _ => format!("更新 {}", cleaned),
        },
        Language::English => match commit_type {
            "docs" => format!("update {} docs", cleaned),
            "test" => format!("update {} tests", cleaned),
            "feat" => format!("update {} logic", cleaned),
            _ => format!("update {}", cleaned),
        },
    }
}

fn format_commit_line(commit_type: &str, scope: Option<&str>, subject: &str) -> String {
    let subject = subject.trim();
    if let Some(scope) = scope {
        if !scope.is_empty() {
            return format!("{}({}): {}", commit_type, scope, subject);
        }
    }
    format!("{}: {}", commit_type, subject)
}

fn get_user_input(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
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
        let input = get_user_input(current_lang.select_language_prompt())?;
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
            .replace("{}", choice.display_name())
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

fn detect_system_python(language: &Language) -> Result<PathBuf> {
    let output = Command::new("python3")
        .arg("-c")
        .arg("import sys; print(sys.executable)")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if path.is_empty() {
                eprintln!("{}", language.python_missing_warning());
                Err(language.mlx_not_configured().into())
            } else {
                Ok(PathBuf::from(path))
            }
        }
        _ => {
            eprintln!("{}", language.python_missing_warning());
            Err(language.mlx_not_configured().into())
        }
    }
}

fn default_venv_dir() -> PathBuf {
    if let Ok(path) = env::var("GIT_CA_VENV_PATH") {
        PathBuf::from(path)
    } else if let Ok(home) = env::var("HOME") {
        Path::new(&home).join(".cache/git-ca/venv")
    } else {
        PathBuf::from(".git-ca-venv")
    }
}

fn resolve_venv_python(venv_dir: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let candidates = ["Scripts/python.exe", "Scripts/python"];
    #[cfg(not(target_os = "windows"))]
    let candidates = ["bin/python3", "bin/python"];

    for rel in candidates.iter() {
        let candidate = venv_dir.join(rel);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn get_mlx_import_error(python: &Path) -> Result<Option<String>> {
    match Command::new(python).arg("-c").arg("import mlx_lm").output() {
        Ok(output) => {
            if output.status.success() {
                Ok(None)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let detail = if !stderr.trim().is_empty() {
                    stderr.trim().to_string()
                } else if !stdout.trim().is_empty() {
                    stdout.trim().to_string()
                } else {
                    format!("Exit status: {}", output.status)
                };
                Ok(Some(detail))
            }
        }
        Err(e) => Err(AppError::Custom(format!(
            "Failed to execute {python:?}: {e}"
        ))),
    }
}

fn create_or_refresh_venv(system_python: &Path, venv_dir: &Path) -> Result<()> {
    if let Some(parent) = venv_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut command = Command::new(system_python);
    command
        .arg("-m")
        .arg("venv")
        .arg("--upgrade-deps")
        .arg(venv_dir);

    match command.status() {
        Ok(status) if status.success() => return Ok(()),
        Ok(_) | Err(_) => {
            let status = Command::new(system_python)
                .arg("-m")
                .arg("venv")
                .arg(venv_dir)
                .status()
                .map_err(|e| {
                    AppError::Custom(format!("Failed to create venv with {system_python:?}: {e}"))
                })?;
            if status.success() {
                Ok(())
            } else {
                Err(AppError::Custom(format!(
                    "Failed to bootstrap virtual environment with {system_python:?}: {status}",
                )))
            }
        }
    }
}

fn prepare_python_for_install(python: &Path, language: &Language) -> Result<()> {
    let pip_status = Command::new(python)
        .args(["-m", "pip", "--version"])
        .status();

    if !pip_status.map(|status| status.success()).unwrap_or(false) {
        println!("{}", language.installing_pip());
        let status = Command::new(python)
            .args(["-m", "ensurepip", "--upgrade"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| {
                AppError::Custom(format!("Failed to bootstrap pip with {python:?}: {e}"))
            })?;

        if !status.success() {
            return Err(AppError::Custom(
                language
                    .pip_install_failed()
                    .replace("{}", &status.to_string()),
            ));
        }
    }

    let status = Command::new(python)
        .args(["-m", "pip", "install", "--upgrade", "pip"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| AppError::Custom(format!("Failed to upgrade pip with {python:?}: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::Custom(
            language
                .pip_install_failed()
                .replace("{}", &status.to_string()),
        ))
    }
}

fn install_mlx(python: &Path, language: &Language) -> Result<()> {
    let status = Command::new(python)
        .args(["-m", "pip", "install", "--upgrade", "mlx-lm"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            AppError::Custom(format!(
                "Failed to run pip install mlx-lm via {python:?}: {e}"
            ))
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::Custom(
            language
                .mlx_install_failed()
                .replace("{}", &status.to_string()),
        ))
    }
}

fn ensure_python_mlx(language: &Language) -> Result<PathBuf> {
    let system_python = detect_system_python(language)?;
    let venv_dir = default_venv_dir();

    if let Some(existing_python) = resolve_venv_python(&venv_dir) {
        if get_mlx_import_error(&existing_python)?.is_none() {
            return Ok(existing_python);
        }
    }

    match get_mlx_import_error(&system_python)? {
        None => return Ok(system_python),
        Some(initial_error) => eprintln!(
            "{}",
            language
                .mlx_dependency_warning()
                .replace("{}", initial_error.as_str())
        ),
    }

    create_or_refresh_venv(&system_python, &venv_dir)?;
    let venv_python = resolve_venv_python(&venv_dir).ok_or_else(|| {
        AppError::Custom(format!(
            "Failed to locate Python interpreter inside {:?}",
            venv_dir
        ))
    })?;

    prepare_python_for_install(&venv_python, language)?;
    println!("{}", language.installing_mlx());
    install_mlx(&venv_python, language)?;

    match get_mlx_import_error(&venv_python)? {
        None => {
            println!("{}", language.mlx_install_success());
            Ok(venv_python)
        }
        Some(final_error) => Err(AppError::Custom(format!(
            "{}\n{}",
            language.mlx_not_configured(),
            final_error
        ))),
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

    if args.len() > 1 && args[1] == "language" {
        select_language(&mut git_config)?;
        return Ok(());
    }

    let python_path = ensure_python_mlx(&language)?;

    let model = DEFAULT_MLX_MODEL.to_string();

    let current_dir = env::current_dir()?;
    let repo_path = find_git_repository(&current_dir)
        .ok_or_else(|| AppError::Custom(language.not_in_git_repository().to_string()))?;

    let repo = Repository::open(repo_path)?;
    let mut index = repo.index()?;

    env::set_current_dir(repo.path().parent().unwrap())?;

    if index.add_all(["*"], IndexAddOption::DEFAULT, None).is_err() {
        println!("{}", language.no_changes_staged());
        return Ok(());
    }

    let diff = get_diff()?;
    let mut commit_msg = analyze_diff(&diff, &model, &language, &python_path)?;

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

    let name = git_config.get_or_prompt("user.name", language.enter_name_prompt())?;
    let email = git_config.get_or_prompt("user.email", language.enter_email_prompt())?;

    let signature = Signature::now(&name, &email)?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let parent_commit = repo.head()?.peel_to_commit()?;

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_msg,
        &tree,
        &[&parent_commit],
    )?;

    println!("{}", language.changes_committed());
    println!(
        "{}",
        language.commit_message_label().replace("{}", &commit_msg)
    );

    Ok(())
}
