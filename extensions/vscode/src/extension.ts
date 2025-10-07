import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

// Define the RequestState and ResponseState interfaces to match your Rust structs
interface RequestState {
    type: 'RequestModification' | 'NotifyModified';
    file_path: string;
    // Add other fields as needed, e.g., 'content' for modification requests
}

interface ResponseState {
    success: boolean;
    message?: string;
    // Add other fields as needed
}

export function activate(context: vscode.ExtensionContext) {
    console.log('Vespe Editor Communicator extension is active!');

    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!workspaceRoot) {
        vscode.window.showErrorMessage('No workspace folder open.');
        return;
    }

    // Define the paths for the communication files
    // IMPORTANT: Adjust these paths to where your Rust program will create them
    const requestFilePath = path.join(workspaceRoot, 'request.json'); 
    const responseFilePath = path.join(workspaceRoot, 'response.json'); 

    // --- File Watcher for request.json ---
    const watcher = vscode.workspace.createFileSystemWatcher(requestFilePath);

    watcher.onDidChange(async (uri) => {
        console.log(`request.json changed: ${uri.fsPath}`);
        try {
            const content = await fs.promises.readFile(uri.fsPath, 'utf8');
            const request: RequestState = JSON.parse(content);

            console.log('Received request:', request);

            let response: ResponseState = { success: false };

            switch (request.type) {
                case 'RequestModification':
                    response = await handleRequestModification(request.file_path);
                    break;
                case 'NotifyModified':
                    response = await handleNotifyModified(request.file_path);
                    break;
                default:
                    response.message = `Unknown request type: ${request.type}`;
                    break;
            }

            // Write response back to response.json
            await fs.promises.writeFile(responseFilePath, JSON.stringify(response, null, 2), 'utf8');
            console.log('Sent response:', response);

        } catch (error: any) {
            console.error('Error processing request.json:', error);
            await fs.promises.writeFile(responseFilePath, JSON.stringify({ success: false, message: error.message }, null, 2), 'utf8');
        }
    });

    context.subscriptions.push(watcher);

    // --- Handlers for different request types ---

    async function handleRequestModification(filePath: string): Promise<ResponseState> {
        try {
            const document = await vscode.workspace.openTextDocument(filePath);
            // If the document has unsaved changes, save it
            if (document.isDirty) {
                await document.save();
                console.log(`Saved dirty file: ${filePath}`);
            }
            // Here you might implement a "lock" mechanism, e.g., by making the document read-only
            // For now, we just confirm it's ready for modification by the external program.
            return { success: true, message: `File ${filePath} is ready for modification.` };
        } catch (error: any) {
            return { success: false, message: `Failed to prepare file for modification: ${error.message}` };
        }
    }

    async function handleNotifyModified(filePath: string): Promise<ResponseState> {
        try {
            const document = vscode.workspace.textDocuments.find(doc => doc.uri.fsPath === filePath);
            if (document) {
                // Reload the file to reflect external changes
                // The 'true' argument forces a reload from disk
                    // VSCode automatically reloads files when external changes are detected.
                // If the document is dirty, a prompt might appear to reload. Forcing a reload
                // without user interaction for a dirty file is complex and often not desired.
                // For now, we assume external changes are handled by VSCode's built-in mechanisms.
                // If a more explicit reload is needed for dirty files, consider `vscode.commands.executeCommand('workbench.action.files.revert')`
                // or closing and reopening the document.
                // await document.getText(); // This would just get the current text, not force a reload from disk if dirty. 
                console.log(`Reloaded file: ${filePath}`);
            }
            // Here you might "unlock" the file if a locking mechanism was in place
            return { success: true, message: `File ${filePath} reloaded.` };
        } catch (error: any) {
            return { success: false, message: `Failed to reload file: ${error.message}` };
        }
    }
}

export function deactivate() {
    console.log('Vespe Editor Communicator extension is deactivated.');
}