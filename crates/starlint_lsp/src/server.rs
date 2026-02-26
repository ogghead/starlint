//! LSP server implementation for starlint.
//!
//! Implements the `tower_lsp::LanguageServer` trait, providing:
//! - Inline diagnostics on open/change/save
//! - Quick-fix code actions for rules that provide fixes
//! - Config hot-reload when `starlint.toml` changes

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability, CodeActionResponse,
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidChangeWatchedFilesRegistrationOptions, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, FileSystemWatcher, GlobPattern,
    InitializeParams, InitializeResult, InitializedParams, Range, Registration, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Url, WatchKind,
};
use tower_lsp::{Client, LanguageServer};

use starlint_config::resolve::{find_config_file, load_config};
use starlint_core::diagnostic::OutputFormat;
use starlint_core::engine::LintSession;
use starlint_core::rules::rules_for_config;

use crate::convert;
use crate::document::{CachedFix, DocumentState};

/// Maximum time allowed for session rebuild (config loading + WASM compilation).
const REBUILD_TIMEOUT: Duration = Duration::from_secs(30);

/// LSP server backend for starlint.
pub struct Backend {
    /// LSP client handle for sending notifications.
    client: Client,
    /// Active lint session (rebuilt on config change).
    session: Arc<RwLock<Option<LintSession>>>,
    /// Open document states keyed by URI.
    documents: Arc<RwLock<HashMap<Url, DocumentState>>>,
    /// Resolved workspace root path.
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
}

impl Backend {
    /// Create a new backend with the given LSP client.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            session: Arc::new(RwLock::new(None)),
            documents: Arc::new(RwLock::new(HashMap::new())),
            workspace_root: Arc::new(RwLock::new(None)),
        }
    }

    /// Build a `LintSession` from the workspace config.
    ///
    /// Config loading, rule construction, and WASM compilation are offloaded
    /// to a blocking thread to avoid stalling the LSP event loop.
    async fn rebuild_session(&self) {
        let search_dir = self
            .workspace_root
            .read()
            .await
            .clone()
            .unwrap_or_else(|| PathBuf::from("."));

        let task = tokio::task::spawn_blocking(move || {
            let config = find_config_file(&search_dir)
                .and_then(|p| match load_config(&p) {
                    Ok(c) => Some(c),
                    Err(err) => {
                        tracing::warn!("failed to parse {}: {err}", p.display());
                        None
                    }
                })
                .unwrap_or_default();

            let configured = rules_for_config(&config.rules, &config.overrides);
            tracing::info!("LSP: {} native rule(s) enabled", configured.rules.len());

            let override_set = starlint_core::overrides::OverrideSet::compile(&config.overrides);
            let mut session = LintSession::new(configured.rules, OutputFormat::Pretty)
                .with_severity_overrides(configured.severity_overrides)
                .with_override_set(override_set)
                .with_disabled_rules(configured.disabled_rules);

            // Load WASM plugins if configured.
            if !config.plugins.is_empty() {
                match build_plugin_host(&config.plugins) {
                    Ok(host) => {
                        tracing::info!("LSP: loaded {} WASM plugin(s)", config.plugins.len());
                        session = session.with_plugin_host(Box::new(host));
                    }
                    Err(err) => {
                        tracing::warn!("LSP: failed to load WASM plugins: {err}");
                    }
                }
            }

            session
        });

        match tokio::time::timeout(REBUILD_TIMEOUT, task).await {
            Ok(Ok(session)) => *self.session.write().await = Some(session),
            Ok(Err(err)) => tracing::error!("LSP: session rebuild panicked: {err}"),
            Err(_elapsed) => {
                tracing::error!("LSP: session rebuild timed out after {REBUILD_TIMEOUT:?}");
            }
        }
    }

    /// Lint a document and publish diagnostics.
    async fn lint_and_publish(&self, uri: &Url) {
        let session_guard = self.session.read().await;
        let Some(session) = session_guard.as_ref() else {
            return;
        };

        let mut docs = self.documents.write().await;
        let Some(doc) = docs.get_mut(uri) else {
            return;
        };

        let file_path = uri_to_path(uri);
        let result = session.lint_single_file(&file_path, &doc.text);

        // Convert diagnostics.
        let mut lsp_diagnostics = Vec::new();
        let mut cached_fixes = Vec::new();

        for diag in &result.diagnostics {
            let lsp_diag = convert::to_lsp_diagnostic(diag, &doc.text);
            lsp_diagnostics.push(lsp_diag.clone());

            // Cache code actions for fixes.
            if let Some(action) = convert::fix_to_code_action(diag, &lsp_diag, uri, &doc.text) {
                cached_fixes.push(CachedFix {
                    diagnostic_range: lsp_diag.range,
                    action,
                });
            }
        }

        doc.cached_fixes = cached_fixes;

        // Drop locks before async call.
        let version = doc.version;
        drop(docs);
        drop(session_guard);

        self.client
            .publish_diagnostics(uri.clone(), lsp_diagnostics, Some(version))
            .await;
    }
}

/// Build a WASM plugin host from config plugin declarations.
fn build_plugin_host(
    plugins: &[starlint_config::PluginDeclaration],
) -> std::result::Result<starlint_wasm_host::runtime::WasmPluginHost, Box<dyn std::error::Error>> {
    let pairs: Vec<_> = plugins.iter().map(|p| (p.path.as_path(), "")).collect();
    starlint_wasm_host::runtime::WasmPluginHost::with_plugins(&pairs)
}

/// Convert a `Url` to a `PathBuf`, falling back to the URL path on error.
fn uri_to_path(uri: &Url) -> PathBuf {
    uri.to_file_path()
        .unwrap_or_else(|()| PathBuf::from(uri.path()))
}

#[tower_lsp::async_trait]
#[allow(clippy::ignored_unit_patterns)] // tower-lsp async_trait generates unit patterns
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Store workspace root for config discovery.
        if let Some(root_uri) = params.root_uri {
            *self.workspace_root.write().await = Some(uri_to_path(&root_uri));
        } else if let Some(folders) = &params.workspace_folders {
            if let Some(first) = folders.first() {
                *self.workspace_root.write().await = Some(uri_to_path(&first.uri));
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("starlint LSP server initialized");

        // Register file watcher for config hot-reload.
        if let Ok(opts) = serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
            watchers: vec![FileSystemWatcher {
                glob_pattern: GlobPattern::String("**/starlint.toml".to_owned()),
                kind: Some(WatchKind::all()),
            }],
        }) {
            let registration = Registration {
                id: "starlint-config-watch".to_owned(),
                method: "workspace/didChangeWatchedFiles".to_owned(),
                register_options: Some(opts),
            };
            if let Err(err) = self.client.register_capability(vec![registration]).await {
                tracing::warn!("failed to register config file watcher: {err}");
            }
        }

        self.rebuild_session().await;

        // Re-lint all documents that were opened during initialization.
        // VS Code sends did_open for already-open files before the session is
        // ready, so those initial lint_and_publish calls silently return empty.
        let uris: Vec<Url> = self.documents.read().await.keys().cloned().collect();
        for uri in &uris {
            self.lint_and_publish(uri).await;
        }
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("starlint LSP server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        let text = params.text_document.text;

        self.documents
            .write()
            .await
            .insert(uri.clone(), DocumentState::new(version, text));

        self.lint_and_publish(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // We use TextDocumentSyncKind::FULL, so the first content change has the full text.
        if let Some(change) = params.content_changes.into_iter().next() {
            let mut docs = self.documents.write().await;
            if let Some(doc) = docs.get_mut(&uri) {
                doc.update(version, change.text);
            }
        }

        self.lint_and_publish(&uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.lint_and_publish(&params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.write().await.remove(&uri);

        // Clear diagnostics for closed document.
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    #[allow(clippy::significant_drop_tightening)] // Read lock needed across iteration
    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let docs = self.documents.read().await;
        let Some(doc) = docs.get(&uri) else {
            return Ok(None);
        };

        // Return cached fixes whose diagnostic range intersects the requested range.
        let actions: Vec<_> = doc
            .cached_fixes
            .iter()
            .filter(|f| ranges_intersect(&f.diagnostic_range, &params.range))
            .map(|f| CodeActionOrCommand::CodeAction(f.action.clone()))
            .collect();

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn did_change_watched_files(&self, _params: DidChangeWatchedFilesParams) {
        // Config file changed — rebuild session and re-lint all open documents.
        tracing::info!("LSP: config changed, rebuilding session");
        self.rebuild_session().await;

        let uris: Vec<Url> = self.documents.read().await.keys().cloned().collect();
        for uri in &uris {
            self.lint_and_publish(uri).await;
        }
    }
}

/// Check whether two LSP ranges overlap or touch.
const fn ranges_intersect(a: &Range, b: &Range) -> bool {
    !(a.end.line < b.start.line
        || (a.end.line == b.start.line && a.end.character < b.start.character)
        || b.end.line < a.start.line
        || (b.end.line == a.start.line && b.end.character < a.start.character))
}

#[cfg(test)]
mod tests {
    use super::{ranges_intersect, uri_to_path};
    use std::path::PathBuf;
    use tower_lsp::lsp_types::{Position, Range, Url};

    #[test]
    fn test_ranges_intersect() {
        let a = Range::new(Position::new(0, 0), Position::new(0, 5));
        let b = Range::new(Position::new(0, 3), Position::new(0, 8));
        assert!(
            ranges_intersect(&a, &b),
            "overlapping ranges should intersect"
        );
    }

    #[test]
    fn test_ranges_no_intersect() {
        let a = Range::new(Position::new(0, 0), Position::new(0, 5));
        let b = Range::new(Position::new(1, 0), Position::new(1, 3));
        assert!(
            !ranges_intersect(&a, &b),
            "non-overlapping ranges should not intersect"
        );
    }

    #[test]
    fn test_ranges_adjacent_intersect() {
        let a = Range::new(Position::new(0, 0), Position::new(0, 5));
        let b = Range::new(Position::new(0, 5), Position::new(0, 10));
        // For code action matching, touching ranges count as intersecting.
        assert!(
            ranges_intersect(&a, &b),
            "adjacent ranges should intersect for code action matching"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)] // Test URL is a known-valid constant
    fn test_uri_to_path() {
        let uri = Url::parse("file:///home/user/test.js").unwrap();
        let path = uri_to_path(&uri);
        assert_eq!(
            path,
            PathBuf::from("/home/user/test.js"),
            "should convert file URI to path"
        );
    }
}
