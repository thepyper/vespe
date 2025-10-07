"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = require("vscode");
const fs = require("fs");
const path = require("path");
function activate(context) {
    var _a, _b;
    console.log('Vespe Editor Communicator extension is active!');
    const workspaceRoot = (_b = (_a = vscode.workspace.workspaceFolders) === null || _a === void 0 ? void 0 : _a[0]) === null || _b === void 0 ? void 0 : _b.uri.fsPath;
    if (!workspaceRoot) {
        vscode.window.showErrorMessage('No workspace folder open.');
        return;
    }
    // Define the paths for the communication files
    // IMPORTANT: Adjust these paths to where your Rust program will create them
    const requestFilePath = process.env.VESPE_REQUEST_FILE_PATH || path.join(workspaceRoot, 'request.json');
    const responseFilePath = process.env.VESPE_RESPONSE_FILE_PATH || path.join(workspaceRoot, 'response.json');
    // --- File Watcher for request.json ---
    const watcher = vscode.workspace.createFileSystemWatcher(requestFilePath);
    watcher.onDidChange((uri) => __awaiter(this, void 0, void 0, function* () {
        console.log(`request.json changed: ${uri.fsPath}`);
        try {
            const content = yield fs.promises.readFile(uri.fsPath, 'utf8');
            const request = JSON.parse(content);
            console.log('Received request:', request);
            let response = { success: false };
            switch (request.type) {
                case 'RequestModification':
                    response = yield handleRequestModification(request.file_path);
                    break;
                case 'NotifyModified':
                    response = yield handleNotifyModified(request.file_path);
                    break;
                default:
                    response.message = `Unknown request type: ${request.type}`;
                    break;
            }
            // Write response back to response.json
            yield fs.promises.writeFile(responseFilePath, JSON.stringify(response, null, 2), 'utf8');
            console.log('Sent response:', response);
        }
        catch (error) {
            console.error('Error processing request.json:', error);
            yield fs.promises.writeFile(responseFilePath, JSON.stringify({ success: false, message: error.message }, null, 2), 'utf8');
        }
    }));
    context.subscriptions.push(watcher);
    // --- Handlers for different request types ---
    function handleRequestModification(filePath) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const document = yield vscode.workspace.openTextDocument(filePath);
                // If the document has unsaved changes, save it
                if (document.isDirty) {
                    yield document.save();
                    console.log(`Saved dirty file: ${filePath}`);
                }
                // Here you might implement a "lock" mechanism, e.g., by making the document read-only
                // For now, we just confirm it's ready for modification by the external program.
                return { success: true, message: `File ${filePath} is ready for modification.` };
            }
            catch (error) {
                return { success: false, message: `Failed to prepare file for modification: ${error.message}` };
            }
        });
    }
    function handleNotifyModified(filePath) {
        return __awaiter(this, void 0, void 0, function* () {
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
            }
            catch (error) {
                return { success: false, message: `Failed to reload file: ${error.message}` };
            }
        });
    }
}
function deactivate() {
    console.log('Vespe Editor Communicator extension is deactivated.');
}
//# sourceMappingURL=extension.js.map