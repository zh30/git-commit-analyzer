use git2::{Config, IndexAddOption, Repository, Signature};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, HOST};
use serde_json::{json, Value};
use std::env;
use std::fmt;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const OLLAMA_API_BASE: &str = "http://localhost:11434/api";
const CONFIG_MODEL_KEY: &str = "commit-analyzer.model";
const CONFIG_LANGUAGE_KEY: &str = "commit-analyzer.language";
const COMMIT_TYPES: &[&str] = &["feat", "fix", "docs", "style", "refactor", "test", "chore"];

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
            Language::English => "Fetching available Ollama models...",
            Language::Chinese => "正在获取可用的 Ollama 模型...",
        }
    }

    fn available_models(&self) -> &'static str {
        match self {
            Language::English => "\nAvailable models:",
            Language::Chinese => "\n可用模型：",
        }
    }

    fn select_model_prompt(&self) -> &'static str {
        match self {
            Language::English => "\nSelect a model by number: ",
            Language::Chinese => "\n请输入模型编号：",
        }
    }

    fn model_set_as_default(&self) -> &'static str {
        match self {
            Language::English => "Model '{}' set as default.",
            Language::Chinese => "已将模型'{}'设置为默认模型。",
        }
    }

    fn ollama_connection_warning(&self) -> &'static str {
        match self {
            Language::English => "Warning: Failed to connect to Ollama: {}",
            Language::Chinese => "警告：连接 Ollama 失败：{}",
        }
    }

    fn ensure_ollama_running(&self) -> &'static str {
        match self {
            Language::English => "Please ensure Ollama is running on localhost:11434",
            Language::Chinese => "请确保 Ollama 正在 localhost:11434 上运行",
        }
    }

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
            Language::English => "\nDo you want to (u)se this message, (e)dit it, or (c)ancel? [u/e/c]: ",
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

    fn ollama_not_accessible(&self) -> &'static str {
        match self {
            Language::English => "Ollama is not running or not accessible. Please start Ollama and ensure it's running on localhost:11434, then try again.",
            Language::Chinese => "Ollama 未运行或不可访问。请启动 Ollama 并确保它在 localhost:11434 上运行，然后重试。",
        }
    }

    fn no_models_found(&self) -> &'static str {
        match self {
            Language::English => "No models found in Ollama. Please ensure Ollama is running and has models installed.",
            Language::Chinese => "在 Ollama 中未找到模型。请确保 Ollama 正在运行并已安装模型。",
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
    Http(reqwest::Error),
    Json(serde_json::Error),
    Custom(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Git(e) => write!(f, "Git error: {e}"),
            AppError::Io(e) => write!(f, "IO error: {e}"),
            AppError::Http(e) => write!(f, "HTTP error: {e}"),
            AppError::Json(e) => write!(f, "JSON error: {e}"),
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

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Http(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Json(err)
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

fn analyze_diff(diff: &str, model: &str, language: &Language) -> Result<String> {
    let client = create_generation_client()?;
    let prompt = build_commit_prompt(diff, language);

    println!("{}", language.generating_commit_message());
    println!("{}", language.this_may_take_moment());
    
    let response = client
        .post(format!("{OLLAMA_API_BASE}/generate"))
        .json(&json!({
            "model": model,
            "prompt": prompt,
            "stream": true
        }))
        .send()
        .map_err(|e| {
            if e.is_timeout() {
                AppError::Custom(format!(
                    "Request timed out after 2 minutes. This might happen with large models or slow systems.\n\
                    Try using a smaller/faster model with 'git ca model' or ensure your system has sufficient resources."
                ))
            } else if e.is_connect() {
                AppError::Custom(format!(
                    "Failed to connect to Ollama at {}. Please ensure Ollama is running and accessible.",
                    OLLAMA_API_BASE
                ))
            } else {
                AppError::Custom(format!("Network error: {}", e))
            }
        })?;

    if !response.status().is_success() {
        return Err(AppError::Custom(format!(
            "Unable to get response from Ollama. Status code: {}. Please ensure Ollama is running and accessible.",
            response.status()
        )));
    }

    let mut full_response = String::new();
    let reader = BufReader::new(response);
    io::stdout().flush()?;

    println!("{}", language.processing_response());

    for line in reader.lines() {
        let line = line.map_err(|e| AppError::Custom(format!("Failed to read response: {}", e)))?;
        if line.is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<Value>(&line) {
            if let Some(response_part) = json["response"].as_str() {
                print!("{response_part}");
                io::stdout().flush()?;
                full_response.push_str(response_part);
            }
            
            if json["done"].as_bool().unwrap_or(false) {
                break;
            }
        }
    }
    
    println!("{}", language.commit_message_generated());
    Ok(process_ollama_response(&full_response))
}

fn process_ollama_response(response: &str) -> String {
    let response_without_thinking = if response.trim_start().starts_with("<think>") {
        response.find("</think>")
            .map(|end_index| response[(end_index + "</think>".len())..].trim_start())
            .unwrap_or(response)
    } else {
        response
    };

    let lines: Vec<&str> = response_without_thinking
        .lines()
        .filter(|line| !line.starts_with("Fixes #") && !line.starts_with("Closes #"))
        .collect();

    let mut processed_lines = Vec::new();
    let mut started = false;

    for line in lines {
        if !started && COMMIT_TYPES.iter().any(|&t| line.starts_with(t)) {
            started = true;
        }
        if started {
            processed_lines.push(line);
        }
    }

    processed_lines.join("\n")
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

fn create_http_client() -> Result<Client> {
    Ok(Client::builder().timeout(Duration::from_secs(5)).build()?)
}

fn create_generation_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(HOST, HeaderValue::from_static("localhost:11434"));
    
    Ok(Client::builder()
        .timeout(Duration::from_secs(120))  // 2 minutes for AI generation
        .default_headers(headers)
        .build()?)
}

fn get_ollama_models() -> Result<Vec<String>> {
    let client = create_http_client()?;
    let response = client.get(format!("{OLLAMA_API_BASE}/tags")).send()?;

    if !response.status().is_success() {
        return Err(AppError::Custom(format!(
            "Unable to get models from Ollama. Status code: {}",
            response.status()
        )));
    }

    let data: Value = response.json()?;
    let models = data["models"]
        .as_array()
        .ok_or("Invalid response format")?
        .iter()
        .filter_map(|model| model["name"].as_str())
        .map(String::from)
        .collect();

    Ok(models)
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
    println!("{}", choice.language_set_to().replace("{}", &choice.display_name()));
    Ok(choice)
}

fn get_language(git_config: &GitConfig) -> Language {
    git_config
        .get(CONFIG_LANGUAGE_KEY)
        .ok()
        .and_then(|lang| Language::from_str(&lang))
        .unwrap_or(Language::English)
}

fn select_default_model(git_config: &mut GitConfig, language: &Language) -> Result<String> {
    println!("{}", language.fetching_models());
    
    let models = get_ollama_models()?;
    if models.is_empty() {
        return Err(language.no_models_found().into());
    }

    println!("{}", language.available_models());
    for (i, model) in models.iter().enumerate() {
        println!("{}. {}", i + 1, model);
    }

    let choice = loop {
        let input = get_user_input(&language.select_model_prompt())?;
        match input.parse::<usize>() {
            Ok(num) if num > 0 && num <= models.len() => break num - 1,
            _ => println!("{}", language.invalid_selection()),
        }
    };

    let selected_model = models[choice].clone();
    git_config.set(CONFIG_MODEL_KEY, &selected_model)?;
    
    println!("{}", language.model_set_as_default().replace("{}", &selected_model));
    Ok(selected_model)
}

fn is_ollama_running() -> Result<bool> {
    let client = create_http_client()?;
    match client.get(format!("{OLLAMA_API_BASE}/tags")).send() {
        Ok(response) => Ok(response.status().is_success()),
        Err(e) => {
            let language = Language::English;
            eprintln!("{}", language.ollama_connection_warning().replace("{}", &e.to_string()));
            eprintln!("{}", language.ensure_ollama_running());
            Ok(false)
        }
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
    
    if args.len() > 1 && args[1] == "model" {
        select_default_model(&mut git_config, &language)?;
        return Ok(());
    }

    if args.len() > 1 && args[1] == "language" {
        select_language(&mut git_config)?;
        return Ok(());
    }

    if !is_ollama_running()? {
        return Err(language.ollama_not_accessible().into());
    }
    
    let model = match git_config.get(CONFIG_MODEL_KEY) {
        Ok(model) => model,
        Err(_) => {
            println!("{}", language.no_default_model());
            select_default_model(&mut git_config, &language)?
        }
    };

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
    let mut commit_msg = analyze_diff(&diff, &model, &language)?;

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

    let name = git_config.get_or_prompt("user.name", &language.enter_name_prompt())?;
    let email = git_config.get_or_prompt("user.email", &language.enter_email_prompt())?;

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
    println!("{}", language.commit_message_label().replace("{}", &commit_msg));

    Ok(())
}