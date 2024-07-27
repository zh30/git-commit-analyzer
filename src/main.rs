use std::process::Command;
use std::io::{self, Write};
use reqwest::blocking::Client;
use serde_json::json;
use git2::{Repository, IndexAddOption, Signature};

const OLLAMA_API_BASE: &str = "http://localhost:11434/api";

fn get_diff() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["diff", "--cached"])
        .output()?;
    Ok(String::from_utf8(output.stdout)?)
}

fn analyze_diff(diff: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let prompt = format!(
        "Analyze this git diff and provide a commit message following the Git Flow format:

<type>(<scope>): <subject>

<body>

<footer>

Where:
- <type> is one of: feat, fix, docs, style, refactor, test, chore
- <scope> is optional and represents the module affected
- <subject> is a short description in the imperative mood
- <body> provides detailed description (optional)
- <footer> mentions any breaking changes or closed issues (optional)

Here's the diff to analyze:

{}

Please provide only the formatted commit message, without any additional explanations.", diff);
    
    let response = client.post(format!("{}/generate", OLLAMA_API_BASE))
        .json(&json!({
            "model": "codegemma:2b",
            "prompt": prompt,
            "stream": false
        }))
        .send()?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json()?;
        Ok(result["response"].as_str().unwrap_or("").to_string())
    } else {
        Err(format!("Error: Unable to get response from Ollama. Status code: {}", response.status()).into())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open(".")?;
    let mut index = repo.index()?;
    
    if index.add_all(&["*"], IndexAddOption::DEFAULT, None).is_err() {
        println!("No changes staged for commit.");
        return Ok(());
    }

    let diff = get_diff()?;
    let summary = analyze_diff(&diff)?;

    println!("\nProposed commit message:");
    println!("{}", summary);

    print!("\nDo you want to use this message? (Y/n): ");
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    let response = response.trim().to_lowercase();

    let commit_msg = if response.is_empty() || response == "y" {
        summary
    } else {
        print!("Enter your commit message: ");
        io::stdout().flush()?;
        let mut custom_msg = String::new();
        io::stdin().read_line(&mut custom_msg)?;
        custom_msg.trim().to_string()
    };

    let signature = Signature::now("Henry Zhang", "hello@zhanghe.dev")?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let parent_commit = repo.head()?.peel_to_commit()?;

    repo.commit(Some("HEAD"), &signature, &signature, &commit_msg, &tree, &[&parent_commit])?;

    println!("\nChanges committed successfully.");

    Ok(())
}