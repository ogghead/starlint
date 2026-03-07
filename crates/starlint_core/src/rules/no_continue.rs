//! Rule: `no-continue`
//!
//! Disallow `continue` statements. Some style guides forbid `continue`
//! because it can make control flow harder to follow.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `continue` statements.
#[derive(Debug)]
pub struct NoContinue;

impl LintRule for NoContinue {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-continue".to_owned(),
            description: "Disallow `continue` statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ContinueStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ContinueStatement(stmt) = node else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "no-continue".to_owned(),
            message: "Unexpected use of `continue` statement".to_owned(),
            span: Span::new(stmt.span.start, stmt.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoContinue)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_continue() {
        let diags = lint("for (var i = 0; i < 10; i++) { if (i === 5) continue; }");
        assert_eq!(diags.len(), 1, "continue should be flagged");
    }

    #[test]
    fn test_allows_loop_without_continue() {
        let diags = lint("for (var i = 0; i < 10; i++) { foo(i); }");
        assert!(
            diags.is_empty(),
            "loop without continue should not be flagged"
        );
    }
}
