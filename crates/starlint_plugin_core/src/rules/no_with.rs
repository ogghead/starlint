//! Rule: `no-with`
//!
//! Disallow `with` statements. The `with` statement is deprecated in strict
//! mode and creates confusing scope semantics.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags `with` statements.
#[derive(Debug)]
pub struct NoWith;

impl LintRule for NoWith {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-with".to_owned(),
            description: "Disallow `with` statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::WithStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::WithStatement(stmt) = node else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "no-with".to_owned(),
            message: "Unexpected use of `with` statement".to_owned(),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoWith)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_with_statement() {
        // Note: with statement only parses in non-strict (sloppy) mode
        let diags = lint("with (obj) { foo; }");
        assert_eq!(diags.len(), 1, "with statement should be flagged");
    }

    #[test]
    fn test_allows_normal_code() {
        let diags = lint("var x = obj.foo;");
        assert!(
            diags.is_empty(),
            "normal property access should not be flagged"
        );
    }
}
