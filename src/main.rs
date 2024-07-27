use std::process::Command;
use std::io::{self, Write};
use std::env;
use reqwest::blocking::Client;
use serde_json::json;
use git2::{Repository, IndexAddOption, Signature};

const OLLAMA_API_BASE: &str = "http://localhost:11434/api";
const GROQ_API_BASE: &str = "https://api.groq.com/openai/v1/chat/completions";

enum AIProvider {
    Ollama,
    Groq,
}

fn get_diff() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["diff", "--cached"])
        .output()?;
    Ok(String::from_utf8(output.stdout)?)
}

fn analyze_diff(diff: &str, provider: AIProvider) -> Result<String, Box<dyn std::error::Error>> {
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

Please provide only the formatted commit message, without any additional explanations or markdown formatting.", diff);

    match provider {
        AIProvider::Ollama => {
            let response = client.post(format!("{}/generate", OLLAMA_API_BASE))
                .json(&json!({
                    "model": "llama3.1",
                    "prompt": prompt,
                    "stream": false
                }))
                .send()?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json()?;
                Ok(result["response"].as_str().unwrap_or("").trim().to_string())
            } else {
                Err(format!("Error: Unable to get response from Ollama. Status code: {}", response.status()).into())
            }
        },
        AIProvider::Groq => {
            let groq_api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");
            let response = client.post(GROQ_API_BASE)
                .header("Authorization", format!("Bearer {}", groq_api_key))
                .json(&json!({
                    "model": "llama-3.1-8b-instant",
                    "messages": [{"role": "user", "content": prompt}]
                }))
                .send()?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json()?;
                Ok(result["choices"][0]["message"]["content"].as_str().unwrap_or("").trim().to_string())
            } else {
                Err(format!("Error: Unable to get response from Groq. Status code: {}", response.status()).into())
            }
        }
    }
}

fn get_user_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open(".")?;
    let mut index = repo.index()?;
    
    if index.add_all(&["*"], IndexAddOption::DEFAULT, None).is_err() {
        println!("No changes staged for commit.");
        return Ok(());
    }

    let diff = get_diff()?;

    let provider = match get_user_input("Choose AI provider (1: Ollama, 2: Groq): ")?.as_str() {
        "1" => AIProvider::Ollama,
        "2" => AIProvider::Groq,
        _ => {
            println!("Invalid choice. Using Ollama as default.");
            AIProvider::Ollama
        }
    };

    let mut commit_msg = analyze_diff(&diff, provider)?;

    loop {
        println!("\nProposed commit message:");
        println!("{}", commit_msg);

        let choice = get_user_input("\nDo you want to (u)se this message, (e)dit it, or (c)ancel? [u/e/c]: ")?;

        match choice.to_lowercase().as_str() {
            "u" => break,
            "e" => {
                commit_msg = get_user_input("Enter your commit message (use multiple lines if needed, end with an empty line):\n")?;
                break;
            },
            "c" => {
                println!("Commit cancelled.");
                return Ok(());
            },
            _ => println!("Invalid choice. Please try again."),
        }
    }

    let signature = Signature::now("Henry Zhang", "hello@zhanghe.dev")?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let parent_commit = repo.head()?.peel_to_commit()?;

    repo.commit(Some("HEAD"), &signature, &signature, &commit_msg, &tree, &[&parent_commit])?;

    println!("\nChanges committed successfully.");
    println!("Commit message:\n{}", commit_msg);

    Ok(())
}