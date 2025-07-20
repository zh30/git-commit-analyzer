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
                const message = await runGitCommitAnalyzer(workspaceFolder.uri.fsPath);
                
                if (message) {
                    // Insert the message into the commit input box
                    const gitExtension = vscode.extensions.getExtension('vscode.git')?.exports;
                    if (gitExtension) {
                        const git = gitExtension.getAPI(1);
                        if (git.repositories.length > 0) {
                            const repository = git.repositories[0];
                            repository.inputBox.value = message;
                            vscode.window.showInformationMessage('Commit message generated!');
                        }
                    }
                }
            } catch (error) {
                vscode.window.showErrorMessage(`Failed to generate commit message: ${error}`);
            }
        });

    } catch (error) {
        vscode.window.showErrorMessage(`Error: ${error}`);
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
        // Find the git-ca binary
        const binaryPath = findGitCaBinary();
        if (!binaryPath) {
            reject('git-ca binary not found. Please ensure it is built and available.');
            return;
        }

        // Execute the binary and capture output
        const child = child_process.spawn(binaryPath, [], {
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
                // Extract the commit message from the output
                const message = extractCommitMessage(output);
                resolve(message);
            } else {
                reject(errorOutput || `Process exited with code ${code}`);
            }
        });

        child.on('error', (error) => {
            reject(`Failed to start process: ${error.message}`);
        });

        // Send 'u' to use the generated message
        child.stdin.write('u\n');
        child.stdin.end();
    });
}

function extractCommitMessage(output: string): string {
    // Look for the commit message in the output
    const lines = output.split('\n');
    let messageStart = false;
    const messageLines: string[] = [];

    for (const line of lines) {
        if (line.includes('Changes committed successfully.')) {
            break;
        }
        
        if (messageStart) {
            if (line.trim()) {
                messageLines.push(line);
            }
        }
        
        if (line.includes('Commit message:')) {
            messageStart = true;
        }
    }

    return messageLines.join('\n').trim();
}

function findGitCaBinary(): string | null {
    // Try common locations
    const possiblePaths = [
        // Look in workspace parent directory
        path.join(vscode.workspace.workspaceFolders![0].uri.fsPath, '..', 'target', 'release', 'git-ca'),
        // Look in system PATH
        'git-ca',
        // Look in common bin directories
        path.join(process.env.HOME || '', '.cargo', 'bin', 'git-ca'),
        path.join('/usr', 'local', 'bin', 'git-ca'),
        path.join('/usr', 'bin', 'git-ca')
    ];

    for (const binaryPath of possiblePaths) {
        try {
            child_process.execSync(`which ${binaryPath}`);
            return binaryPath;
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