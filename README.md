# Git Commit Analyzer

Git Commit Analyzer is a powerful Git plugin that leverages AI to automatically generate meaningful commit messages based on your staged changes. It supports both local (Ollama) and remote (Groq) AI models to analyze git diffs and propose commit messages following the Git Flow format.

## Features

- Automatic generation of Git Flow compliant commit messages
- Support for multiple AI providers (Ollama and Groq)
- Interactive mode allowing users to choose AI provider, use, edit, or cancel the proposed commit message
- Cross-platform compatibility (Linux, macOS, Windows)
- Customizable with your personal Git signature

## Prerequisites

- Git (version 2.0 or later)
- Rust (version 1.54 or later)
- Cargo (usually comes with Rust)
- Ollama (with llama3.1 model installed) for local AI processing
- Groq API key (optional, for using Groq's remote AI model)

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

7. If you plan to use Groq, set up the API key as an environment variable:
   ```
   echo 'export GROQ_API_KEY=your_groq_api_key_here' >> ~/.bashrc  # or ~/.bash_profile, or ~/.zshrc
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

6. If you plan to use Groq, set up the API key as an environment variable:
   - In the same 'Environment Variables' dialog, under 'User variables', click 'New'
   - Set Variable name as `GROQ_API_KEY` and Variable value as your Groq API key
   - Click 'OK' to close all dialogs

7. Restart any open command prompts for the changes to take effect.

## Usage

Once installed, you can use the Git Commit Analyzer in any Git repository by running:

```
git commit-analyzer
```

This will:
1. Prompt you to choose an AI provider (Ollama or Groq)
2. Analyze your staged changes using the selected AI provider
3. Generate a commit message proposal
4. Allow you to use the proposed message, edit it, or cancel the commit

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- The Rust community for providing excellent libraries and tools
- Ollama for providing the llama3.1 model
- Groq for their API service