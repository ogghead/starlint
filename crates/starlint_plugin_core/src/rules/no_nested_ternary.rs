//! Rule: `no-nested-ternary`
//!
//! Disallow nested ternary expressions. Nested ternaries are difficult to
//! read and should be refactored into `if`/`else` statements or extracted
//! into separate variables.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags ternary expressions that contain nested ternary sub-expressions.
#[derive(Debug)]
pub struct NoNestedTernary;

impl LintRule for NoNestedTernary {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-nested-ternary".to_owned(),
            description: "Disallow nested ternary expressions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ConditionalExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ConditionalExpression(expr) = node else {
            return;
        };

        let nested_in_consequent = matches!(
            ctx.node(expr.consequent),
            Some(AstNode::ConditionalExpression(_))
        );
        let nested_in_alternate = matches!(
            ctx.node(expr.alternate),
            Some(AstNode::ConditionalExpression(_))
        );

        if !nested_in_consequent && !nested_in_alternate {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-nested-ternary".to_owned(),
            message: "Nested ternary expression".to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some(
                "Refactor into if/else statements or extract into separate variables".to_owned(),
            ),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNestedTernary)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_nested_in_consequent() {
        let diags = lint("const x = a ? b ? 1 : 2 : 3;");
        assert_eq!(diags.len(), 1, "should flag nested ternary in consequent");
    }

    #[test]
    fn test_flags_nested_in_alternate() {
        let diags = lint("const x = a ? 1 : b ? 2 : 3;");
        assert_eq!(diags.len(), 1, "should flag nested ternary in alternate");
    }

    #[test]
    fn test_allows_simple_ternary() {
        let diags = lint("const x = a ? 1 : 2;");
        assert!(diags.is_empty(), "simple ternary should not be flagged");
    }

    #[test]
    fn test_allows_non_ternary() {
        let diags = lint("const x = a || b;");
        assert!(diags.is_empty(), "logical expression should not be flagged");
    }
}
