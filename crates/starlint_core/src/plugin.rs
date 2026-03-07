//! Unified plugin trait for native rules and WASM plugins.
//!
//! Both native Rust rules and WASM plugins implement [`Plugin`], giving the
//! engine a single dispatch interface. Native rules use the Rust types
//! directly (zero serialization); WASM plugins serialize across the boundary.

use std::path::Path;

use starlint_ast::tree::AstTree;
use starlint_plugin_sdk::diagnostic::Diagnostic;
use starlint_plugin_sdk::rule::RuleMeta;
use starlint_scope::ScopeData;

/// Context provided to a plugin when linting a single file.
///
/// Contains the parsed AST tree, source text, file path, and optional
/// scope analysis data. Constructed once per file and shared across all
/// plugins.
pub struct FileContext<'a> {
    /// Path of the file being linted.
    pub file_path: &'a Path,
    /// Original source text of the file.
    pub source_text: &'a str,
    /// File extension without the dot (e.g. `"ts"`, `"tsx"`, `"js"`).
    pub extension: &'a str,
    /// The flat-indexed AST tree produced by `starlint_parser`.
    pub tree: &'a AstTree,
    /// Scope analysis data, available when any plugin sets
    /// [`Plugin::needs_scope_analysis`] to `true`.
    pub scope_data: Option<&'a ScopeData>,
}

/// Unified plugin trait for lint rule providers.
///
/// Mirrors the WASM v2 plugin interface (`plugin-v2` in `plugin.wit`):
/// `get-rules`, `lint-file`, `get-file-patterns`, `configure`.
///
/// Native rule bundles implement this via [`LintRulePlugin`](crate::lint_rule_plugin::LintRulePlugin),
/// which wraps existing [`LintRule`](crate::lint_rule::LintRule) implementations
/// with the same per-node dispatch and interest-based filtering.
///
/// WASM plugins implement this via wrapper structs in `starlint_wasm_host`
/// that handle serialization across the WASM boundary.
pub trait Plugin: Send + Sync {
    /// Return metadata for all rules provided by this plugin.
    fn rules(&self) -> Vec<RuleMeta>;

    /// Lint a single file. Returns diagnostics for all rules in this plugin.
    fn lint_file(&self, ctx: &FileContext<'_>) -> Vec<Diagnostic>;

    /// File-path glob patterns this plugin applies to.
    ///
    /// An empty vec (the default) means the plugin runs on all files.
    fn file_patterns(&self) -> Vec<String> {
        vec![]
    }

    /// Configure the plugin from a JSON string.
    ///
    /// Returns a list of validation errors. An empty list means success.
    fn configure(&mut self, _config: &str) -> Vec<String> {
        vec![]
    }

    /// Whether this plugin requires scope analysis data in [`FileContext`].
    ///
    /// When any plugin returns `true`, the engine builds [`ScopeData`] and
    /// makes it available via [`FileContext::scope_data`]. Scope analysis
    /// is not free, so plugins should only request it when needed.
    fn needs_scope_analysis(&self) -> bool {
        false
    }
}
