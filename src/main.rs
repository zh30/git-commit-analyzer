use git2::{Config, IndexAddOption, Repository, Signature};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

const OLLAMA_API_BASE: &str = "http://localhost:11434/api";

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

fn analyze_diff(diff: &str, model: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let prompt = format!(
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

Here's the diff to analyze:

{}

Your task:
1. Analyze the given git diff.
2. **Generate only one** commit message strictly following the Git Flow format described above.
3. Ensure your response contains **ONLY** the formatted commit message, without any additional explanations or markdown.
4. The commit message MUST start with <type> and follow the exact structure shown.

Example of a valid response:
feat(user-auth): implement password reset functionality

Add a new endpoint for password reset requests.
Implement email sending for reset links.

Remember: Your response should only include the commit message, nothing else.",
        diff
    );

    println!("Generating commit message...");
    
    // Make request with streaming enabled
    let response = client
        .post(format!("{}/generate", OLLAMA_API_BASE))
        .json(&json!({
            "model": model,
            "prompt": prompt,
            "stream": true
        }))
        .send()?;

    if !response.status().is_success() {
        return Err(format!(
            "Error: Unable to get response from Ollama. Status code: {}",
            response.status()
        )
        .into());
    }

    // Process the streaming response
    let mut full_response = String::new();
    let reader = BufReader::new(response);
    io::stdout().flush()?;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        // Parse the JSON response
        if let Ok(json) = serde_json::from_str::<Value>(&line) {
            if let Some(response_part) = json["response"].as_str() {
                print!("{}", response_part);
                io::stdout().flush()?;
                full_response.push_str(response_part);
            }
            
            // If done is true, we've received the complete response
            if json["done"].as_bool().unwrap_or(false) {
                break;
            }
        }
    }
    
    println!("\n\nCommit message generated.");
    
    // Post-process Ollama's response
    Ok(process_ollama_response(&full_response))
}

fn process_ollama_response(response: &str) -> String {
    // First, strip thinking section if it exists
    let response_without_thinking = if response.trim_start().starts_with("<think>") {
        if let Some(end_index) = response.find("</think>") {
            response[(end_index + "</think>".len())..].trim_start()
        } else {
            response
        }
    } else {
        response
    };

    // Remove any "Fixes #XXX" or "Closes #XXX" lines
    let lines: Vec<&str> = response_without_thinking
        .lines()
        .filter(|line| !line.starts_with("Fixes #") && !line.starts_with("Closes #"))
        .collect();

    // Ensure only content in Git Flow format is returned
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

fn get_ollama_models() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
    let response = client.get(format!("{}/tags", OLLAMA_API_BASE)).send()?;

    if response.status().is_success() {
        let data: serde_json::Value = response.json()?;
        let mut models = Vec::new();

        if let Some(models_array) = data["models"].as_array() {
            for model in models_array {
                if let Some(name) = model["name"].as_str() {
                    models.push(name.to_string());
                }
            }
        }

        Ok(models)
    } else {
        Err(format!(
            "Error: Unable to get models from Ollama. Status code: {}",
            response.status()
        )
        .into())
    }
}

fn select_default_model(config: &mut Config) -> Result<String, Box<dyn std::error::Error>> {
    println!("Fetching available Ollama models...");
    
    let models = get_ollama_models()?;
    if models.is_empty() {
        return Err("No models found in Ollama. Please ensure Ollama is running and has models installed.".into());
    }

    println!("\nAvailable models:");
    for (i, model) in models.iter().enumerate() {
        println!("{}. {}", i + 1, model);
    }

    let choice = loop {
        let input = get_user_input("\nSelect a model by number: ")?;
        match input.parse::<usize>() {
            Ok(num) if num > 0 && num <= models.len() => break num - 1,
            _ => println!("Invalid selection. Please try again."),
        }
    };

    let selected_model = models[choice].clone();
    set_user_config(config, "commit-analyzer.model", &selected_model)?;
    
    println!("Model '{}' set as default.", selected_model);
    Ok(selected_model)
}

fn is_ollama_running() -> Result<bool, Box<dyn std::error::Error>> {
    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
    match client.get(format!("{}/tags", OLLAMA_API_BASE)).send() {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut config = get_git_config()?;
    
    // Check if the user wants to change the model
    if args.len() > 1 && args[1] == "model" {
        select_default_model(&mut config)?;
        return Ok(());
    }

    // Ensure Ollama is running
    if !is_ollama_running()? {
        return Err("Ollama is not running. Please start Ollama and try again.".into());
    }
    
    // Get or select default model
    let model = match get_user_config(&config, "commit-analyzer.model") {
        Ok(model) => model,
        Err(_) => {
            println!("No default model set. Please select a model.");
            select_default_model(&mut config)?
        }
    };

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
    
    let mut commit_msg = analyze_diff(&diff, &model)?;

    loop {
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