//! Plugin host trait for external lint plugins.
//!
//! Native Rust rules implement [`LintRule`](crate::lint_rule::LintRule).
//! External plugin hosts (e.g., WASM) implement [`PluginHost`] and are
//! injected into [`LintSession`](crate::engine::LintSession).

use std::path::Path;

use starlint_ast::tree::AstTree;
use starlint_plugin_sdk::diagnostic::Diagnostic;

/// Trait for external plugin hosts (WASM, etc.).
///
/// A plugin host receives a parsed AST and source text for each file,
/// runs its plugins, and returns diagnostics.
pub trait PluginHost: Send + Sync {
    /// Lint a single file using all loaded plugins.
    ///
    /// The caller provides the `AstTree`, source text, and file path.
    /// The implementor can traverse the tree to collect relevant nodes
    /// for its plugins.
    fn lint_file(&self, file_path: &Path, source_text: &str, tree: &AstTree) -> Vec<Diagnostic>;
}
