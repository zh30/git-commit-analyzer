# Git Commit Analyzer (git-ca)

[中文版本 (Chinese Version)](README-cn.md) | [Developer Documentation](README-dev.md)

Git Commit Analyzer (git-ca) is a powerful Git plugin that utilizes AI technology to automatically generate meaningful commit messages based on your staged changes. Whether you're a programming novice or an experienced developer, this tool will make your Git workflow more efficient.

## 🌟 Features

- 🤖 Automatically generates commit messages compliant with Git Flow conventions
- 🔄 Supports multiple AI providers (local Ollama and remote Groq)
- 🖥️ Interactive mode allowing you to choose AI providers, use, edit, or cancel suggested commit messages
- 🌍 Cross-platform compatibility (Linux, macOS, Windows)
- 🎨 Customizable with your personal Git signature

## 📋 Prerequisites

Before you begin, ensure you have the following software installed on your system:

- [Git](https://git-scm.com/downloads) (version 2.0 or higher)
- [Ollama](https://ollama.ai/download) (with llama3.1 model installed) - for local AI processing
- Groq API key (optional, for using Groq's remote AI model)

## 🚀 Installation Guide

We provide a convenient installation script that works on Linux, macOS, and Windows (Git Bash).

### For Linux and macOS Users

Open a terminal and run the following command:

```bash
bash <(curl -s https://scripts.zhanghe.dev/git_ca_install.sh)
```

### For Windows Users - TBD (Currently Unavailable)

1. Download the [install.bat](https://scripts.zhanghe.dev/git_ca_install.bat) file.
2. Double-click to run the `install.bat` file.

The installation script will automatically download the `git-ca` executable and place it in the appropriate location. For Unix-like systems, it will also add it to your PATH automatically.

### Manual Setup of Groq API Key (Optional)

If you plan to use Groq as an AI provider, you need to set up the Groq API key:

- **For Linux and macOS**:
  ```bash
  echo 'export GROQ_API_KEY=your_groq_api_key' >> ~/.bashrc
  source ~/.bashrc
  ```

- **For Windows**:
  Run in Command Prompt:
  ```
  setx GROQ_API_KEY "your_groq_api_key"
  ```

## 🎯 How to Use

After installation, you can use Git Commit Analyzer in any Git repository:

1. Stage your changes in your Git repository (using the `git add` command).
2. Run the following command:

   ```
   git-ca
   ```

3. Follow the prompts to select an AI provider (Ollama or Groq).
4. The program will analyze your staged changes and generate a suggested commit message.
5. You can choose to use the suggested message, edit it, or cancel the commit.

## 🤝 Contributing

We welcome contributions of any kind! If you have good ideas or find bugs, feel free to submit a Pull Request or create an Issue.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgements

- The Rust community for excellent libraries and tools
- Ollama for providing the llama3.1 model
- Groq for their API service