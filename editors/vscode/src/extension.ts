import { workspace, ExtensionContext, window } from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: ExtensionContext): void {
  const config = workspace.getConfiguration("starlint");

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
      fileEvents: workspace.createFileSystemWatcher("**/starlint.toml"),
    },
  };

  client = new LanguageClient(
    "starlint",
    "starlint Language Server",
    serverOptions,
    clientOptions,
  );

  context.subscriptions.push(client);

  client.start().catch((err: Error) => {
    window.showErrorMessage(
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
