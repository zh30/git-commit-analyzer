# Git Commit Analyzer

Git Commit Analyzer is a powerful Git plugin that leverages AI to automatically generate meaningful commit messages based on your staged changes. It uses the llama3.1 model to analyze git diffs and propose commit messages following the Git Flow format.

## Features

- Git Commit Analyzer
  - Features
  - Prerequisites
  - Installation
    - Linux and macOS
    - Windows - theoretically possible
  - Usage
  - Customization
  - Contributing
  - License
  - Acknowledgments

## Prerequisites

- Git (version 2.0 or later)
- Rust (version 1.54 or later)
- Cargo (usually comes with Rust)
- Ollama (with llama3.1 model installed)

## Installation

### Linux and macOS

1. Clone the repository:
   ```
   git clone https://github.com/zh30/git-commit-analyzer.git
   cd git-commit-analyzer
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Create a directory for Git plugins (if it doesn't exist):
   ```
   mkdir -p ~/.git-plugins
   ```

4. Copy the compiled binary to the plugins directory:
   ```
   cp target/release/git-commit-analyzer ~/.git-plugins/
   ```

5. Add the plugins directory to your PATH. Add the following line to your `~/.bashrc`, `~/.bash_profile`, or `~/.zshrc` (depending on your shell):
   ```
   export PATH="$HOME/.git-plugins:$PATH"
   ```

6. Reload your shell configuration:
   ```
   source ~/.bashrc  # or ~/.bash_profile, or ~/.zshrc
   ```

### Windows - theoretically possible 

1. Clone the repository:
   ```
   git clone https://github.com/zh30/git-commit-analyzer.git
   cd git-commit-analyzer
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Create a directory for Git plugins (if it doesn't exist):
   ```
   mkdir %USERPROFILE%\.git-plugins
   ```

4. Copy the compiled binary to the plugins directory:
   ```
   copy target\release\git-commit-analyzer.exe %USERPROFILE%\.git-plugins\
   ```

5. Add the plugins directory to your PATH:
   - Right-click on 'This PC' or 'My Computer' and select 'Properties'
   - Click on 'Advanced system settings'
   - Click on 'Environment Variables'
   - Under 'System variables', find and select 'Path', then click 'Edit'
   - Click 'New' and add `%USERPROFILE%\.git-plugins`
   - Click 'OK' to close all dialogs

6. Restart any open command prompts for the changes to take effect.

## Usage

Once installed, you can use the Git Commit Analyzer in any Git repository by running:

```
git commit-analyzer
```

This will:
1. Analyze your staged changes
2. Generate a commit message proposal
3. Allow you to use the proposed message, edit it, or cancel the commit

## Customization

To customize the Git signature used in commits, modify the following line in `src/main.rs`:

```rust
let signature = Signature::now("Your Name", "your.email@example.com")?;
```

Replace "Your Name" and "your.email@example.com" with your name and email address.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- The Rust community for providing excellent libraries and tools
- Ollama for providing the llama3.1 model