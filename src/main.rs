use std::process::Command;
use std::io::{self, Write};
use std::time::Duration;
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
    let client = Client::builder()
        .timeout(Duration::from_secs(30))  // 设置 30 秒超时
        .build()?;
    
    let prompt = format!("Analyze this git diff and provide a concise summary of the changes:\n\n{}", diff);
    println!("Prompt: {}", prompt);
    let response = client.post(format!("{}/generate", OLLAMA_API_BASE))
        .json(&json!({
            "model": "qwen2:1.5b",
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
    
    match analyze_diff(&diff) {
        Ok(summary) => {
            println!("\nAnalyzed changes:");
            println!("{}", summary);

            print!("\nEnter commit message (press Enter to use the analysis as the commit message): ");
            io::stdout().flush()?;

            let mut commit_msg = String::new();
            io::stdin().read_line(&mut commit_msg)?;
            let commit_msg = commit_msg.trim();

            let commit_msg = if commit_msg.is_empty() { &summary } else { commit_msg };

            let signature = Signature::now("Your Name", "your.email@example.com")?;
            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;
            let parent_commit = repo.head()?.peel_to_commit()?;

            repo.commit(Some("HEAD"), &signature, &signature, commit_msg, &tree, &[&parent_commit])?;

            println!("\nChanges committed successfully.");
        },
        Err(e) => {
            eprintln!("Error analyzing diff: {}. Proceeding with manual commit.", e);
            print!("Enter commit message: ");
            io::stdout().flush()?;

            let mut commit_msg = String::new();
            io::stdin().read_line(&mut commit_msg)?;
            let commit_msg = commit_msg.trim();

            if !commit_msg.is_empty() {
                let signature = Signature::now("Your Name", "your.email@example.com")?;
                let tree_id = index.write_tree()?;
                let tree = repo.find_tree(tree_id)?;
                let parent_commit = repo.head()?.peel_to_commit()?;

                repo.commit(Some("HEAD"), &signature, &signature, commit_msg, &tree, &[&parent_commit])?;

                println!("\nChanges committed successfully.");
            } else {
                println!("No commit message provided. Aborting commit.");
            }
        }
    }

    Ok(())
}