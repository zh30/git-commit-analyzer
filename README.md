# Git Commit Analyzer

[中文版](README_ZH.md)

Git Commit Analyzer is a powerful Git plugin that leverages AI to automatically generate meaningful commit messages based on your staged changes. It uses Ollama to analyze git diffs and propose commit messages following the Git Flow format.

## Features

- Automatic generation of Git Flow compliant commit messages
- Powered by Ollama for local AI processing
- Interactive mode allowing users to use, edit, or cancel the proposed commit message
- Cross-platform compatibility (Linux, macOS, Windows)
- Customizable with your personal Git signature
- Support for model selection and persistence

## Prerequisites

- Git (version 2.0 or later)
- Ollama installed and running (https://ollama.com/download)
- At least one language model installed in Ollama

## Installation

### Homebrew (macOS and Linux)

The easiest way to install Git Commit Analyzer is via Homebrew:

```
brew install git-ca
```

After installation, you can immediately use the `git ca` command.

### Manual Installation (Linux and macOS)

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
   cp target/release/git-ca ~/.git-plugins/
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

## How to Use

After installation, you can use Git Commit Analyzer in any Git repository:

1. Stage your changes in your Git repository (using the `git add` command).
2. Run the following command:

   ```
   git ca
   ```

3. If it's your first time running the command, you'll be prompted to select a model from your installed Ollama models.
4. The program will analyze your staged changes and generate a suggested commit message.
5. You can choose to use the suggested message, edit it, or cancel the commit.

To change the default model at any time, run:

```
git ca model
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- The Rust community for providing excellent libraries and tools
- Ollama for providing local AI model support
