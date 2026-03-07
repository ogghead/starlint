//! Rule: `no-unsafe-negation`
//!
//! Disallow negating the left operand of relational operators. Writing
//! `!a in b` is parsed as `(!a) in b`, not `!(a in b)`. This is almost
//! always a mistake — the same applies to `instanceof`.

use starlint_plugin_sdk::diagnostic::{Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UnaryOperator};
use starlint_ast::types::NodeId;

/// Flags `!x in y` and `!x instanceof y` patterns.
#[derive(Debug)]
pub struct NoUnsafeNegation;

impl LintRule for NoUnsafeNegation {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unsafe-negation".to_owned(),
            description: "Disallow negating the left operand of relational operators".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        // Only check `in` and `instanceof` operators
        if expr.operator != BinaryOperator::In && expr.operator != BinaryOperator::Instanceof {
            return;
        }

        // Check if the left side is a `!` unary expression
        if let Some(AstNode::UnaryExpression(unary)) = ctx.node(expr.left) {
            if unary.operator == UnaryOperator::LogicalNot {
                let op_name = if expr.operator == BinaryOperator::In {
                    "in"
                } else {
                    "instanceof"
                };

                // Fix: `!a in b` → `!(a in b)`
                let expr_span = expr.span;
                let unary_arg = unary.argument;
                #[allow(clippy::as_conversions)]
                let fix = {
                    let source = ctx.source_text();
                    let inner_span = ctx.node(unary_arg).map_or(
                        starlint_ast::types::Span::new(0, 0),
                        starlint_ast::AstNode::span,
                    );
                    let right_span = ctx.node(expr.right).map_or(
                        starlint_ast::types::Span::new(0, 0),
                        starlint_ast::AstNode::span,
                    );
                    let inner_text = source
                        .get(inner_span.start as usize..inner_span.end as usize)
                        .unwrap_or("");
                    let right_text = source
                        .get(right_span.start as usize..right_span.end as usize)
                        .unwrap_or("");
                    let replacement = format!("!({inner_text} {op_name} {right_text})");
                    Some(Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(expr_span.start, expr_span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    })
                };

                ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                    rule_name: "no-unsafe-negation".to_owned(),
                    message: format!(
                        "Unexpected negating the left operand of `{op_name}` operator"
                    ),
                    span: Span::new(expr_span.start, expr_span.end),
                    severity: Severity::Error,
                    help: Some(format!(
                        "Use `!(a {op_name} b)` instead of `!a {op_name} b`"
                    )),
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnsafeNegation)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_negation_in() {
        let diags = lint("if (!key in obj) {}");
        assert_eq!(diags.len(), 1, "!key in obj should be flagged");
    }

    #[test]
    fn test_flags_negation_instanceof() {
        let diags = lint("if (!obj instanceof Foo) {}");
        assert_eq!(diags.len(), 1, "!obj instanceof Foo should be flagged");
    }

    #[test]
    fn test_allows_negated_result() {
        let diags = lint("if (!(key in obj)) {}");
        assert!(diags.is_empty(), "!(key in obj) should not be flagged");
    }

    #[test]
    fn test_allows_normal_in() {
        let diags = lint("if (key in obj) {}");
        assert!(diags.is_empty(), "normal in should not be flagged");
    }

    #[test]
    fn test_allows_normal_instanceof() {
        let diags = lint("if (obj instanceof Foo) {}");
        assert!(diags.is_empty(), "normal instanceof should not be flagged");
    }
}
