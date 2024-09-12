use git2::{Config, IndexAddOption, Repository, Signature};
use reqwest::blocking::Client;
use serde_json::json;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

const OLLAMA_API_BASE: &str = "http://localhost:11434/api";
const GROQ_API_BASE: &str = "https://api.groq.com/openai/v1/chat/completions";
const CEREBRAS_API_BASE: &str = "https://api.cerebras.ai/v1/chat/completions";
enum AIProvider {
    Ollama,
    Groq,
    Cerebras,
}

fn find_git_repository(start_path: &PathBuf) -> Option<PathBuf> {
    let mut current_path = start_path.clone();
    loop {
        let git_dir = current_path.join(".git");
        if git_dir.is_dir() {
            return Some(current_path);
        }
        if !current_path.pop() {
            return None;
        }
    }
}

fn get_diff() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(&["diff", "--cached"]).output()?;
    Ok(String::from_utf8(output.stdout)?)
}

fn analyze_diff(diff: &str, provider: AIProvider) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_prompt = format!(
        "Analyze this git diff and provide a commit message following the Git Flow format:

<type>(<scope>): <subject>

<body>

Where:
- <type> is one of: feat, fix, docs, style, refactor, test, chore
- <scope> is optional and represents the module affected
- <subject> is a short description in the imperative mood
- <body> provides detailed description (optional)

Here's the diff to analyze:

{}

",
        diff
    );

    let ollama_prompt = format!(
        "{}

Your task:
1. Analyze the given git diff.
2. Generate a commit message strictly following the Git Flow format described above.
3. Ensure your response contains ONLY the formatted commit message, without any additional explanations or markdown.
4. The commit message MUST start with <type> and follow the exact structure shown.
5. DO NOT include any 'Fixes #XXX' or 'Closes #XXX' statements, or any footer.
6. DO NOT mention or reference any issue numbers.
7. Focus solely on the changes present in the diff.

Example of a valid response:
feat(user-auth): implement password reset functionality

Add a new endpoint for password reset requests.
Implement email sending for reset links.

Remember: Your response should only include the commit message, nothing else.",
        base_prompt
    );

    let groq_prompt = format!(
        "{}

Please provide only the formatted commit message, without any additional explanations or markdown formatting. 
Important: Do not include 'Fixes #XXX' or 'Closes #XXX' unless the issue number is explicitly mentioned in the diff.",
        base_prompt
    );

    match provider {
        AIProvider::Ollama => {
            let response = client
                .post(format!("{}/generate", OLLAMA_API_BASE))
                .json(&json!({
                    "model": "llama3.1",
                    "prompt": ollama_prompt,
                    "stream": false
                }))
                .send()?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json()?;
                let raw_message = result["response"].as_str().unwrap_or("").trim().to_string();

                // 对 Ollama 的响应进行后处理
                Ok(process_ollama_response(&raw_message))
            } else {
                Err(format!(
                    "Error: Unable to get response from Ollama. Status code: {}",
                    response.status()
                )
                .into())
            }
        }
        AIProvider::Groq => {
            let groq_api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");
            let response = client
                .post(GROQ_API_BASE)
                .header("Authorization", format!("Bearer {}", groq_api_key))
                .json(&json!({
                    "model": "llama-3.1-8b-instant",
                    "messages": [{"role": "user", "content": groq_prompt}]
                }))
                .send()?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json()?;
                Ok(result["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .to_string())
            } else {
                Err(format!(
                    "Error: Unable to get response from Groq. Status code: {}",
                    response.status()
                )
                .into())
            }
        }
        AIProvider::Cerebras => {
            let cerebras_api_key = env::var("CEREBRAS_API_KEY").expect("CEREBRAS_API_KEY not set");
            let response = client
                .post(CEREBRAS_API_BASE)
                .header("Authorization", format!("Bearer {}", cerebras_api_key))
                .json(&json!({
                    "model": "llama3.1-70b",
                    "messages": [{"role": "user", "content": groq_prompt}]
                }))
                .send()?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json()?;
                Ok(result["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .to_string())
            } else {
                Err(format!(
                    "Error: Unable to get response from Cerebras. Status code: {}",
                    response.status()
                )
                .into())
            }
        }
    }
}

fn process_ollama_response(response: &str) -> String {
    // 移除任何 "Fixes #XXX" 或 "Closes #XXX" 行
    let lines: Vec<&str> = response
        .lines()
        .filter(|line| !line.starts_with("Fixes #") && !line.starts_with("Closes #"))
        .collect();

    // 确保只返回符合 Git Flow 格式的内容
    let mut processed_lines = Vec::new();
    let mut started = false;

    for line in lines {
        if !started
            && (line.starts_with("feat")
                || line.starts_with("fix")
                || line.starts_with("docs")
                || line.starts_with("style")
                || line.starts_with("refactor")
                || line.starts_with("test")
                || line.starts_with("chore"))
        {
            started = true;
        }
        if started {
            processed_lines.push(line);
        }
    }

    processed_lines.join("\n")
}

fn get_user_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn get_git_config() -> Result<Config, git2::Error> {
    let config = Config::open_default()?;
    Ok(config)
}

fn get_user_config(config: &Config, key: &str) -> Result<String, git2::Error> {
    config.get_string(key)
}

fn set_user_config(config: &mut Config, key: &str, value: &str) -> Result<(), git2::Error> {
    config.set_str(key, value)
}

fn get_or_set_user_info(
    config: &mut Config,
    key: &str,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    match get_user_config(config, key) {
        Ok(value) => Ok(value),
        Err(_) => {
            let value = get_user_input(prompt)?;
            set_user_config(config, key, &value)?;
            Ok(value)
        }
    }
}

fn is_ollama_running() -> Result<bool, Box<dyn std::error::Error>> {
    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;

    let response = client.get(format!("{}/tags", OLLAMA_API_BASE)).send()?;

    if response.status().is_success() {
        let data: serde_json::Value = response.json()?;

        if let Some(models) = data["models"].as_array() {
            for model in models {
                if let Some(name) = model["name"].as_str() {
                    // 检查模型名称是否包含 "llama" 和 "3.1"
                    if name.to_lowercase().contains("llama") && name.contains("3.1") {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let repo_path =
        find_git_repository(&current_dir).ok_or_else(|| "Not in a git repository".to_string())?;

    let repo = Repository::open(repo_path)?;
    let mut index = repo.index()?;

    std::env::set_current_dir(&repo.path().parent().unwrap())?;

    if index
        .add_all(&["*"], IndexAddOption::DEFAULT, None)
        .is_err()
    {
        println!("No changes staged for commit.");
        return Ok(());
    }

    let diff = get_diff()?;

    let provider = if env::var("GROQ_API_KEY").is_ok() || env::var("CEREBRAS_API_KEY").is_ok() {
        match get_user_input("Choose AI provider (1: Ollama, 2: Groq, 3: Cerebras): ")?.as_str() {
            "1" => check_ollama_and_fallback(),
            "2" if env::var("GROQ_API_KEY").is_ok() => AIProvider::Groq,
            "3" if env::var("CEREBRAS_API_KEY").is_ok() => AIProvider::Cerebras,
            _ => {
                println!("Invalid choice or API key not set. Attempting to use Ollama...");
                check_ollama_and_fallback()
            }
        }
    } else {
        println!("No API keys set. Attempting to use Ollama...");
        check_ollama_and_fallback()
    };

    fn check_ollama_and_fallback() -> AIProvider {
        match is_ollama_running() {
            Ok(true) => AIProvider::Ollama,
            Ok(false) => {
                println!("Warning: Ollama is running, but llama 3.1 model is not available.");
                fallback_to_available_provider()
            }
            Err(e) => {
                println!("Error checking Ollama status: {}.", e);
                fallback_to_available_provider()
            }
        }
    }

    fn fallback_to_available_provider() -> AIProvider {
        if env::var("GROQ_API_KEY").is_ok() {
            println!("Falling back to Groq...");
            AIProvider::Groq
        } else if env::var("CEREBRAS_API_KEY").is_ok() {
            println!("Falling back to Cerebras...");
            AIProvider::Cerebras
        } else {
            println!("No available AI provider. Please ensure Ollama is running with llama 3.1 model, or set GROQ_API_KEY or CEREBRAS_API_KEY.");
            panic!("No available AI provider");
        }
    }

    let mut commit_msg = analyze_diff(&diff, provider)?;

    loop {
        println!("\nProposed commit message:");
        println!("{}", commit_msg);

        let choice = get_user_input(
            "\nDo you want to (u)se this message, (e)dit it, or (c)ancel? [u/e/c]: ",
        )?;

        match choice.to_lowercase().as_str() {
            "u" => break,
            "e" => {
                commit_msg = get_user_input("Enter your commit message (use multiple lines if needed, end with an empty line):\n")?;
                break;
            }
            "c" => {
                println!("Commit cancelled.");
                return Ok(());
            }
            _ => println!("Invalid choice. Please try again."),
        }
    }

    let mut config = get_git_config()?;
    let name = get_or_set_user_info(&mut config, "user.name", "Enter your name: ")?;
    let email = get_or_set_user_info(&mut config, "user.email", "Enter your email: ")?;

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

    println!("\nChanges committed successfully.");
    println!("Commit message:\n{}", commit_msg);

    Ok(())
}
