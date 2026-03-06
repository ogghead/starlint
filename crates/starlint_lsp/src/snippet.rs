//! Custom LSP snippet types for interactive code actions.
//!
//! The standard `lsp-types` crate (via `tower-lsp`) does not include
//! `SnippetTextEdit`. Following rust-analyzer's approach, we define custom
//! serializable types and deliver snippet edits via a client-side command
//! rather than the standard `WorkspaceEdit.changes` field.
//!
//! The VS Code extension registers a command handler for
//! [`APPLY_SNIPPET_COMMAND`] that applies edits using `SnippetString`.

use std::collections::HashMap;

use serde::Serialize;
use tower_lsp::lsp_types;

/// Command name registered by the VS Code extension.
pub const APPLY_SNIPPET_COMMAND: &str = "starlint.applySnippetWorkspaceEdit";

/// A text edit whose `new_text` may contain LSP snippet syntax
/// (`$1`, `${1:placeholder}`, `$0`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetTextEdit {
    /// Range to replace (same semantics as `TextEdit.range`).
    pub range: lsp_types::Range,
    /// Replacement text, possibly containing snippet tab stops.
    pub new_text: String,
    /// Always `InsertTextFormat::Snippet` (2).
    pub insert_text_format: lsp_types::InsertTextFormat,
}

/// A workspace edit composed of snippet text edits, keyed by document URI.
#[derive(Debug, Clone, Serialize)]
pub struct SnippetWorkspaceEdit {
    /// Map of document URI to snippet edits for that document.
    pub changes: HashMap<lsp_types::Url, Vec<SnippetTextEdit>>,
}
