import * as vscode from 'vscode';
import * as child_process from 'child_process';
import * as path from 'path';

export function activate(context: vscode.ExtensionContext) {
    console.log('Git Commit Analyzer extension is now active!');

    // Register the generate message command
    const generateMessageCommand = vscode.commands.registerCommand('gitCommitAnalyzer.generateMessage', async () => {
        await generateCommitMessage();
    });

    context.subscriptions.push(generateMessageCommand);

    // Update button state based on staged changes
    updateButtonState();

    // Listen for changes in the repository
    const gitExtension = vscode.extensions.getExtension('vscode.git')?.exports;
    if (gitExtension) {
        const git = gitExtension.getAPI(1);
        git.onDidChangeState(() => {
            updateButtonState();
        });
        
        git.onDidOpenRepository(() => {
            updateButtonState();
        });
    }
}

async function generateCommitMessage() {
    try {
        // Check if we have a workspace folder
        if (!vscode.workspace.workspaceFolders) {
            vscode.window.showErrorMessage('No workspace folder is open');
            return;
        }

        const workspaceFolder = vscode.workspace.workspaceFolders[0];
        
        // Check if git-ca binary is available
        const binaryPath = findGitCaBinary();
        if (!binaryPath) {
            const selection = await vscode.window.showErrorMessage(
                'git-ca binary not found. Please ensure it is built and available in PATH.',
                'Open Documentation'
            );
            if (selection === 'Open Documentation') {
                vscode.env.openExternal(vscode.Uri.parse('https://github.com/your-repo/git-commit-analyzer#installation'));
            }
            return;
        }

        // Check if we have any staged changes
        const hasStagedChanges = await checkStagedChanges(workspaceFolder.uri.fsPath);
        if (!hasStagedChanges) {
            vscode.window.showWarningMessage('No staged changes found. Please stage some changes first.');
            return;
        }

        // Show progress
        vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: 'Generating commit message...',
            cancellable: false
        }, async (progress) => {
            try {
                progress.report({ increment: 10 });
                const message = await runGitCommitAnalyzer(workspaceFolder.uri.fsPath);
                
                if (message) {
                    // Insert the message into the commit input box
                    const gitExtension = vscode.extensions.getExtension('vscode.git')?.exports;
                    if (gitExtension) {
                        const git = gitExtension.getAPI(1);
                        if (git.repositories.length > 0) {
                            const repository = git.repositories[0];
                            repository.inputBox.value = message;
                            vscode.window.showInformationMessage('Commit message generated!', 'Preview').then(selection => {
                                if (selection === 'Preview') {
                                    vscode.window.showInformationMessage(message, { modal: true });
                                }
                            });
                        }
                    }
                }
            } catch (error) {
                const errorMessage = error instanceof Error ? error.message : String(error);
                vscode.window.showErrorMessage(`Failed to generate commit message: ${errorMessage}`, 'View Details').then(selection => {
                    if (selection === 'View Details') {
                        vscode.window.showErrorMessage(errorMessage, { modal: true });
                    }
                });
            }
        });

    } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        vscode.window.showErrorMessage(`Error: ${errorMessage}`);
    }
}

async function checkStagedChanges(workspacePath: string): Promise<boolean> {
    return new Promise((resolve, reject) => {
        child_process.exec('git diff --cached --name-only', { cwd: workspacePath }, (error, stdout) => {
            if (error) {
                reject(error);
                return;
            }
            resolve(stdout.trim().length > 0);
        });
    });
}

async function runGitCommitAnalyzer(workspacePath: string): Promise<string> {
    return new Promise((resolve, reject) => {
        const binaryPath = findGitCaBinary();
        if (!binaryPath) {
            reject(new Error('git-ca binary not found'));
            return;
        }

        // Execute the binary with --quiet flag to skip interactive prompt
        const child = child_process.spawn(binaryPath, ['--quiet'], {
            cwd: workspacePath,
            stdio: ['pipe', 'pipe', 'pipe']
        });

        let output = '';
        let errorOutput = '';

        child.stdout.on('data', (data) => {
            output += data.toString();
        });

        child.stderr.on('data', (data) => {
            errorOutput += data.toString();
        });

        child.on('close', (code) => {
            if (code === 0) {
                const message = output.trim();
                if (message) {
                    resolve(message);
                } else {
                    reject(new Error('No commit message generated'));
                }
            } else {
                reject(new Error(errorOutput || `git-ca exited with code ${code}`));
            }
        });

        child.on('error', (error) => {
            reject(new Error(`Failed to start git-ca: ${error.message}`));
        });
    });
}


function findGitCaBinary(): string | null {
    if (!vscode.workspace.workspaceFolders) {
        return null;
    }

    const workspaceRoot = vscode.workspace.workspaceFolders[0].uri.fsPath;
    
    // Try common locations
    const possiblePaths = [
        // Look in workspace parent directory (for development)
        path.join(workspaceRoot, '..', 'target', 'release', 'git-ca'),
        path.join(workspaceRoot, '..', 'target', 'debug', 'git-ca'),
        // Look in system PATH
        'git-ca',
        // Look in common bin directories
        path.join(process.env.HOME || '', '.cargo', 'bin', 'git-ca'),
        path.join('/usr', 'local', 'bin', 'git-ca'),
        path.join('/usr', 'bin', 'git-ca'),
        // Look in git-plugins directory
        path.join(process.env.HOME || '', '.git-plugins', 'git-ca')
    ];

    for (const binaryPath of possiblePaths) {
        try {
            // Check if the file exists and is executable
            if (binaryPath === 'git-ca') {
                // Check PATH
                child_process.execSync('which git-ca', { stdio: 'ignore' });
                return 'git-ca';
            } else {
                // Check specific file
                const stats = require('fs').statSync(binaryPath);
                if (stats.isFile()) {
                    child_process.execSync(`test -x "${binaryPath}"`, { stdio: 'ignore' });
                    return binaryPath;
                }
            }
        } catch {
            // Continue searching
        }
    }

    return null;
}

function updateButtonState() {
    // This function would update the button state based on staged changes
    // The actual state management is handled by VS Code's "when" clauses in package.json
}

export function deactivate() {
    console.log('Git Commit Analyzer extension is now deactivated');
}