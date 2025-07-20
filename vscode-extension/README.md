# Git Commit Analyzer VS Code Extension

A VS Code extension that integrates with the Git Commit Analyzer (git-ca) to provide AI-powered commit message generation directly in the source control panel.

## Features

- **Seamless Integration**: Adds a generate button to the source control panel
- **Smart Detection**: Only enables the button when there are staged changes
- **AI-Powered**: Uses Ollama AI to generate meaningful commit messages
- **Git Flow Format**: Follows conventional commit message format
- **Real-time**: Generates messages in real-time with streaming responses

## Requirements

1. **Git Commit Analyzer (git-ca)**: The Rust binary must be built and available
2. **Ollama**: Must be running locally with at least one model installed
3. **Git**: Must be initialized in your workspace

## Installation

### 1. Build the Rust Binary

```bash
# From the root of git-commit-analyzer
cargo build --release
```

### 2. Install the Extension

```bash
cd vscode-extension
npm install
npm run compile
```

Then install the extension in VS Code:
1. Open VS Code
2. Go to Extensions view (Ctrl+Shift+X)
3. Click "..." and select "Install from VSIX..."
4. Select the generated `.vsix` file

### 3. Development Mode

For development, you can run the extension in debug mode:
1. Open the vscode-extension folder in VS Code
2. Press F5 to launch Extension Development Host

## Usage

1. **Stage Changes**: Use `git add` or VS Code's source control panel to stage changes
2. **Generate Message**: Click the magic wand icon (🪄) in the source control panel
3. **Review**: The generated commit message will appear in the commit input box
4. **Commit**: Review and modify the message if needed, then commit

## Configuration

The extension will automatically use the same configuration as the git-ca binary, including:
- Default Ollama model (stored in Git config)
- User information from Git config

To change the default model, you can run the binary directly:
```bash
./target/release/git-ca model
```

## Troubleshooting

### Binary Not Found
If the extension can't find the git-ca binary:
1. Ensure you've built it with `cargo build --release`
2. Check the binary is in `target/release/git-ca`
3. Add the binary directory to your PATH

### Ollama Issues
- Ensure Ollama is running: `ollama serve`
- Check available models: `ollama list`
- Install a model: `ollama pull llama2` or `ollama pull mistral`

### Git Issues
- Ensure you're in a Git repository
- Check that you have staged changes
- Verify your Git configuration is set up correctly

## Extension Development

### Building
```bash
cd vscode-extension
npm install
npm run compile
```

### Packaging
```bash
npm install -g vsce
vsce package
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

MIT License - see LICENSE file for details