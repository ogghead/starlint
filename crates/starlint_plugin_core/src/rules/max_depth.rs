//! Rule: `max-depth`
//!
//! Enforce a maximum depth of nested control-flow blocks. Deeply nested
//! code is harder to read and understand — prefer extracting into functions.

use std::sync::RwLock;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Default maximum nesting depth.
const DEFAULT_MAX: u32 = 4;

/// Enforces a maximum depth of nested control-flow blocks.
#[derive(Debug)]
pub struct MaxDepth {
    /// Maximum nesting depth allowed.
    max: u32,
    /// Current nesting depth (tracked during traversal).
    depth: RwLock<u32>,
}

impl MaxDepth {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max: DEFAULT_MAX,
            depth: RwLock::new(0),
        }
    }
}

impl Default for MaxDepth {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if the AST node introduces a new nesting level.
const fn is_nesting_node(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::IfStatement(_)
            | AstNode::ForStatement(_)
            | AstNode::ForInStatement(_)
            | AstNode::ForOfStatement(_)
            | AstNode::WhileStatement(_)
            | AstNode::DoWhileStatement(_)
            | AstNode::SwitchStatement(_)
            | AstNode::TryStatement(_)
    )
}

impl LintRule for MaxDepth {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "max-depth".to_owned(),
            description: "Enforce a maximum depth of nested blocks".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(n) = config.get("max").and_then(serde_json::Value::as_u64) {
            self.max = u32::try_from(n).unwrap_or(DEFAULT_MAX);
        }
        Ok(())
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::DoWhileStatement,
            AstNodeType::ForInStatement,
            AstNodeType::ForOfStatement,
            AstNodeType::ForStatement,
            AstNodeType::IfStatement,
            AstNodeType::SwitchStatement,
            AstNodeType::TryStatement,
            AstNodeType::WhileStatement,
        ])
    }

    fn leave_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::DoWhileStatement,
            AstNodeType::ForInStatement,
            AstNodeType::ForOfStatement,
            AstNodeType::ForStatement,
            AstNodeType::IfStatement,
            AstNodeType::SwitchStatement,
            AstNodeType::TryStatement,
            AstNodeType::WhileStatement,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        if !is_nesting_node(node) {
            return;
        }

        let Ok(mut depth_guard) = self.depth.write() else {
            return;
        };
        *depth_guard = depth_guard.saturating_add(1);
        let current = *depth_guard;
        drop(depth_guard);

        if current > self.max {
            let span = match node {
                AstNode::IfStatement(s) => s.span,
                AstNode::ForStatement(s) => s.span,
                AstNode::ForInStatement(s) => s.span,
                AstNode::ForOfStatement(s) => s.span,
                AstNode::WhileStatement(s) => s.span,
                AstNode::DoWhileStatement(s) => s.span,
                AstNode::SwitchStatement(s) => s.span,
                AstNode::TryStatement(s) => s.span,
                _ => return,
            };
            ctx.report(Diagnostic {
                rule_name: "max-depth".to_owned(),
                message: format!(
                    "Blocks are nested too deeply ({current}). Maximum allowed is {}",
                    self.max
                ),
                span: Span::new(span.start, span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }

    fn leave(&self, _node_id: NodeId, node: &AstNode, _ctx: &mut LintContext<'_>) {
        if !is_nesting_node(node) {
            return;
        }

        if let Ok(mut depth_guard) = self.depth.write() {
            *depth_guard = depth_guard.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint_with_max(source: &str, max: u32) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MaxDepth {
            max,
            depth: RwLock::new(0),
        })];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_shallow_nesting() {
        let source = "if (true) { console.log(1); }";
        let diags = lint_with_max(source, 2);
        assert!(diags.is_empty(), "shallow nesting should not be flagged");
    }

    #[test]
    fn test_flags_deep_nesting() {
        let source = "if (a) { if (b) { if (c) { console.log(1); } } }";
        let diags = lint_with_max(source, 2);
        assert_eq!(diags.len(), 1, "third level should be flagged");
    }

    #[test]
    fn test_allows_at_limit() {
        let source = "if (a) { if (b) { console.log(1); } }";
        let diags = lint_with_max(source, 2);
        assert!(diags.is_empty(), "nesting at limit should not be flagged");
    }

    #[test]
    fn test_flags_loop_nesting() {
        let source = "for (var i = 0; i < 10; i++) { while (true) { if (a) { break; } } }";
        let diags = lint_with_max(source, 2);
        assert_eq!(diags.len(), 1, "deeply nested loops should be flagged");
    }

    #[test]
    fn test_sequential_not_nested() {
        let source = "if (a) {} if (b) {} if (c) {}";
        let diags = lint_with_max(source, 1);
        assert!(
            diags.is_empty(),
            "sequential blocks should not count as nested"
        );
    }
}
