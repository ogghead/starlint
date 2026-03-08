//! Rule: `no-delete-var`
//!
//! Disallow deleting variables. The `delete` operator is meant for removing
//! properties from objects. Using `delete` on a variable is either a mistake
//! or produces confusing, implementation-dependent behavior.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags `delete` applied directly to a variable identifier.
#[derive(Debug)]
pub struct NoDeleteVar;

impl LintRule for NoDeleteVar {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-delete-var".to_owned(),
            description: "Disallow deleting variables".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::UnaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::UnaryExpression(expr) = node else {
            return;
        };

        if expr.operator != UnaryOperator::Delete {
            return;
        }

        // Only flag when the operand is a plain identifier (i.e. a variable).
        // `delete obj.prop` is fine — that's the intended usage.
        if matches!(
            ctx.node(expr.argument),
            Some(AstNode::IdentifierReference(_))
        ) {
            ctx.report(Diagnostic {
                rule_name: "no-delete-var".to_owned(),
                message: "Variables should not be deleted".to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDeleteVar)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_delete_variable() {
        let diags = lint("var x = 1; delete x;");
        assert_eq!(diags.len(), 1, "delete x should be flagged");
    }

    #[test]
    fn test_allows_delete_property() {
        let diags = lint("delete obj.prop;");
        assert!(diags.is_empty(), "delete obj.prop should not be flagged");
    }

    #[test]
    fn test_allows_delete_computed_property() {
        let diags = lint("delete obj['key'];");
        assert!(diags.is_empty(), "delete obj['key'] should not be flagged");
    }

    #[test]
    fn test_allows_non_delete_unary() {
        let diags = lint("typeof x;");
        assert!(diags.is_empty(), "typeof should not be flagged");
    }
}
