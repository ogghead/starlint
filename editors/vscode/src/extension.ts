import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

/** Shape of the SnippetWorkspaceEdit sent by the server. */
interface SnippetTextEdit {
  range: { start: { line: number; character: number }; end: { line: number; character: number } };
  newText: string;
  insertTextFormat: number;
}

interface SnippetWorkspaceEdit {
  changes: Record<string, SnippetTextEdit[]>;
}

export function activate(context: vscode.ExtensionContext): void {
  const config = vscode.workspace.getConfiguration("starlint");

  if (!config.get<boolean>("enable", true)) {
    return;
  }

  const binaryPath = config.get<string>("path", "") || "starlint";

  const serverOptions: ServerOptions = {
    command: binaryPath,
    args: ["lsp"],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", language: "javascript" },
      { scheme: "file", language: "javascriptreact" },
      { scheme: "file", language: "typescript" },
      { scheme: "file", language: "typescriptreact" },
    ],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/starlint.toml"),
    },
    initializationOptions: {
      experimental: {
        snippetTextEdit: true,
      },
    },
  };

  // Register the command handler for snippet workspace edits.
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "starlint.applySnippetWorkspaceEdit",
      async (edit: SnippetWorkspaceEdit) => {
        if (!edit?.changes) {
          return;
        }

        const wsEdit = new vscode.WorkspaceEdit();
        for (const [uriStr, edits] of Object.entries(edit.changes)) {
          const uri = vscode.Uri.parse(uriStr);
          const snippetEdits = edits.map(
            (e) =>
              new vscode.SnippetTextEdit(
                new vscode.Range(
                  new vscode.Position(e.range.start.line, e.range.start.character),
                  new vscode.Position(e.range.end.line, e.range.end.character),
                ),
                new vscode.SnippetString(e.newText),
              ),
          );
          wsEdit.set(uri, snippetEdits);
        }

        await vscode.workspace.applyEdit(wsEdit);
      },
    ),
  );

  client = new LanguageClient(
    "starlint",
    "starlint Language Server",
    serverOptions,
    clientOptions,
  );

  context.subscriptions.push(client);

  client.start().catch((err: Error) => {
    vscode.window.showErrorMessage(
      `Failed to start starlint LSP server: ${err.message}. ` +
        `Make sure the 'starlint' binary is installed and in your PATH, ` +
        `or set the 'starlint.path' setting.`,
    );
  });
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
