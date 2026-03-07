//! Category-level file guard wrapper for native rules.
//!
//! Wraps a rule with a file-level predicate that skips the rule entirely
//! when the file doesn't match the category's expected patterns (e.g. jest
//! rules skip non-test files). This avoids dispatching category-specific
//! rules to irrelevant files, significantly reducing per-node work.
#![allow(dead_code)]

use std::fmt;
use std::path::Path;

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_plugin_sdk::rule::RuleMeta;

/// File-level predicate: returns `true` when a file is relevant to the category.
type FilePredicate = fn(&str, &Path) -> bool;

/// Wraps a [`LintRule`] with a category-level file guard.
///
/// All trait methods delegate to the inner rule except
/// [`should_run_on_file`](LintRule::should_run_on_file), which first checks
/// the category predicate. If the predicate returns `false`, the rule is
/// skipped for the entire file (both traversal and `run_once`).
struct CategoryGuarded {
    /// The wrapped rule.
    inner: Box<dyn LintRule>,
    /// Category-level file predicate.
    predicate: FilePredicate,
}

impl CategoryGuarded {
    /// Wrap `rule` with a category-level file guard.
    fn new(rule: Box<dyn LintRule>, predicate: FilePredicate) -> Self {
        Self {
            inner: rule,
            predicate,
        }
    }
}

impl fmt::Debug for CategoryGuarded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CategoryGuarded")
            .field("inner", &self.inner)
            .field("predicate", &"<fn>")
            .finish()
    }
}

impl LintRule for CategoryGuarded {
    fn meta(&self) -> RuleMeta {
        self.inner.meta()
    }

    fn run(&self, node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        self.inner.run(node_id, node, ctx);
    }

    fn leave(&self, node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        self.inner.leave(node_id, node, ctx);
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        self.inner.run_once(ctx);
    }

    fn needs_traversal(&self) -> bool {
        self.inner.needs_traversal()
    }

    fn needs_semantic(&self) -> bool {
        self.inner.needs_semantic()
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        self.inner.run_on_types()
    }

    fn leave_on_types(&self) -> Option<&'static [AstNodeType]> {
        self.inner.leave_on_types()
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &Path) -> bool {
        (self.predicate)(source_text, file_path)
            && self.inner.should_run_on_file(source_text, file_path)
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        self.inner.configure(config)
    }
}

/// Wrap each rule in the vec with a category-level file guard.
pub(super) fn guard_all(
    rules: Vec<Box<dyn LintRule>>,
    predicate: FilePredicate,
) -> Vec<Box<dyn LintRule>> {
    rules
        .into_iter()
        .map(|rule| -> Box<dyn LintRule> { Box::new(CategoryGuarded::new(rule, predicate)) })
        .collect()
}
