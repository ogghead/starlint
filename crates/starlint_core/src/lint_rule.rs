//! Unified lint rule trait and context using [`AstTree`].
//!
//! [`LintRule`] operates on the flat-indexed [`AstTree`] from `starlint_ast`,
//! enabling rules to be written once for both native and WASM execution.
//! Each rule receives an [`AstNode`] variant during traversal and can emit
//! diagnostics via the [`LintContext`].

use std::fmt::Debug;
use std::path::Path;

use oxc_semantic::Semantic;

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::tree::AstTree;
use starlint_ast::types::NodeId;
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::RuleMeta;

/// Trait implemented by lint rules using the unified [`AstTree`].
///
/// Rules receive AST nodes during single-pass traversal and emit diagnostics.
/// Implement [`run`](LintRule::run) for per-node checks or
/// [`run_once`](LintRule::run_once) for file-level checks.
pub trait LintRule: Debug + Send + Sync {
    /// Metadata describing this rule.
    fn meta(&self) -> RuleMeta;

    /// Called for each AST node when entering during traversal.
    ///
    /// Default implementation does nothing. Override to inspect specific node types.
    fn run(&self, _node_id: NodeId, _node: &AstNode, _ctx: &mut LintContext<'_>) {}

    /// Called for each AST node when leaving during traversal.
    ///
    /// Default implementation does nothing. Override for rules that need scope
    /// tracking (e.g. counting complexity within function boundaries).
    fn leave(&self, _node_id: NodeId, _node: &AstNode, _ctx: &mut LintContext<'_>) {}

    /// Called once per file, after traversal completes.
    ///
    /// Use for file-level checks (e.g. "file must have a default export").
    fn run_once(&self, _ctx: &mut LintContext<'_>) {}

    /// Whether this rule needs per-node traversal.
    ///
    /// Return `false` if the rule only implements [`run_once`](LintRule::run_once).
    fn needs_traversal(&self) -> bool {
        true
    }

    /// Whether this rule requires semantic analysis (scope tree, symbol table).
    ///
    /// Return `true` to indicate that the rule needs access to [`Semantic`] data
    /// via [`LintContext::semantic()`]. When any active rule returns `true`,
    /// the engine runs a semantic pre-pass before traversal.
    fn needs_semantic(&self) -> bool {
        false
    }

    /// Which [`AstNodeType`] variants this rule handles in [`run`](LintRule::run).
    ///
    /// Return `Some(&[AstNodeType::CallExpression, ...])` to only receive those
    /// node types during traversal. Return `None` (default) to receive **all** nodes.
    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        None
    }

    /// Which [`AstNodeType`] variants this rule handles in [`leave`](LintRule::leave).
    ///
    /// Return `Some(&[...])` to only receive matching nodes on leave.
    /// Default is `Some(&[])` (no leave events) since most rules don't
    /// implement [`leave`](LintRule::leave). Rules that override `leave()`
    /// must also override this method.
    fn leave_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[])
    }

    /// File-level guard: return `false` to skip this rule for the current file.
    ///
    /// Called once per file before traversal/`run_once`. Rules can inspect
    /// source text or file path to bail out early.
    fn should_run_on_file(&self, _source_text: &str, _file_path: &Path) -> bool {
        true
    }

    /// Configure this rule from a JSON config value.
    ///
    /// Called during session setup when the config contains options for this rule.
    fn configure(&mut self, _config: &serde_json::Value) -> Result<(), String> {
        Ok(())
    }
}

/// Context provided to [`LintRule`] implementations during linting.
///
/// Provides access to the [`AstTree`], source text, file path, optional
/// semantic analysis data, and a method to report diagnostics.
pub struct LintContext<'a> {
    /// The flat-indexed AST tree.
    tree: &'a AstTree,
    /// Original source text.
    source_text: &'a str,
    /// Path of the file being linted.
    file_path: &'a Path,
    /// Accumulated diagnostics.
    diagnostics: Vec<Diagnostic>,
    /// Optional semantic analysis (scope tree, symbol table, node ancestry).
    semantic: Option<&'a Semantic<'a>>,
}

impl<'a> LintContext<'a> {
    /// Create a new lint context without semantic analysis.
    pub const fn new(tree: &'a AstTree, source_text: &'a str, file_path: &'a Path) -> Self {
        Self {
            tree,
            source_text,
            file_path,
            diagnostics: Vec::new(),
            semantic: None,
        }
    }

    /// Create a new lint context with semantic analysis data.
    pub const fn with_semantic(
        tree: &'a AstTree,
        source_text: &'a str,
        file_path: &'a Path,
        semantic: &'a Semantic<'a>,
    ) -> Self {
        Self {
            tree,
            source_text,
            file_path,
            diagnostics: Vec::new(),
            semantic: Some(semantic),
        }
    }

    /// Get the AST tree.
    #[must_use]
    pub const fn tree(&self) -> &AstTree {
        self.tree
    }

    /// Look up a node by ID.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&AstNode> {
        self.tree.get(id)
    }

    /// Get the parent of a node.
    #[must_use]
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.tree.parent(id)
    }

    /// Get the semantic analysis data, if available.
    #[must_use]
    pub const fn semantic(&self) -> Option<&'a Semantic<'a>> {
        self.semantic
    }

    /// Get the source text of the file being linted.
    #[must_use]
    pub const fn source_text(&self) -> &str {
        self.source_text
    }

    /// Get the file path.
    #[must_use]
    pub const fn file_path(&self) -> &Path {
        self.file_path
    }

    /// Report a diagnostic.
    pub fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Report a simple error diagnostic.
    pub fn report_error(&mut self, rule_name: &str, message: &str, span: Span) {
        self.diagnostics.push(Diagnostic {
            rule_name: rule_name.to_owned(),
            message: message.to_owned(),
            span,
            severity: Severity::Error,
            help: None,
            fix: None,
            labels: vec![],
        });
    }

    /// Report a simple warning diagnostic.
    pub fn report_warning(&mut self, rule_name: &str, message: &str, span: Span) {
        self.diagnostics.push(Diagnostic {
            rule_name: rule_name.to_owned(),
            message: message.to_owned(),
            span,
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }

    /// Consume the context and return collected diagnostics.
    #[must_use]
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

/// Parse source and run the given [`LintRule`]s, returning diagnostics.
///
/// Convenience helper for tests so each rule doesn't have to duplicate the
/// parse → convert → dispatch boilerplate.
#[cfg(test)]
pub fn lint_source(source: &str, file_path: &str, rules: &[Box<dyn LintRule>]) -> Vec<Diagnostic> {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use crate::ast_converter;
    use crate::traversal::{LintDispatchTable, traverse_ast_tree};

    let path = Path::new(file_path);
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let tree = ast_converter::convert(&parsed.program);
    let traversal_indices: Vec<usize> = rules
        .iter()
        .enumerate()
        .filter(|(_, r)| r.needs_traversal())
        .map(|(i, _)| i)
        .collect();
    let run_once_indices: Vec<usize> = rules
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.needs_traversal())
        .map(|(i, _)| i)
        .collect();
    let table = LintDispatchTable::build_from_indices(rules, &traversal_indices);
    traverse_ast_tree(&tree, rules, &table, &run_once_indices, source, path, None)
}

#[cfg(test)]
mod tests {
    use starlint_ast::tree::AstTree;

    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_lint_context_report() {
        let tree = AstTree::new();
        let mut ctx = LintContext::new(&tree, "let x = 1;", Path::new("test.ts"));
        ctx.report_error("test/rule", "bad code", Span::new(0, 3));
        let diags = ctx.into_diagnostics();
        assert_eq!(diags.len(), 1, "should have one diagnostic");
        assert_eq!(
            diags.first().map(|d| d.rule_name.as_str()),
            Some("test/rule"),
            "rule name should match"
        );
    }

    #[test]
    fn test_lint_context_source_text() {
        let tree = AstTree::new();
        let ctx = LintContext::new(&tree, "const a = 1;", Path::new("test.js"));
        assert_eq!(
            ctx.source_text(),
            "const a = 1;",
            "source text should match"
        );
    }

    #[test]
    fn test_lint_context_tree_access() {
        let tree = AstTree::new();
        let ctx = LintContext::new(&tree, "", Path::new("test.js"));
        assert!(ctx.tree().is_empty(), "empty tree should be empty");
    }
}
