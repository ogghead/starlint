//! Rule: `no-extra-boolean-cast`
//!
//! Disallow unnecessary boolean casts. In contexts where the result is
//! already coerced to a boolean (e.g. `if`, `while`, `for`, ternary test,
//! logical `!`), wrapping in `Boolean()` or `!!` is redundant.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags unnecessary boolean casts like `!!x` in boolean contexts.
#[derive(Debug)]
pub struct NoExtraBooleanCast;

impl LintRule for NoExtraBooleanCast {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-extra-boolean-cast".to_owned(),
            description: "Disallow unnecessary boolean casts".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ConditionalExpression,
            AstNodeType::DoWhileStatement,
            AstNodeType::ForStatement,
            AstNodeType::IfStatement,
            AstNodeType::WhileStatement,
        ])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Extract the test condition's NodeId from various statement/expression types
        let test_id: NodeId = match node {
            AstNode::IfStatement(stmt) => stmt.test,
            AstNode::WhileStatement(stmt) => stmt.test,
            AstNode::DoWhileStatement(stmt) => stmt.test,
            AstNode::ForStatement(stmt) => {
                let Some(id) = stmt.test else { return };
                id
            }
            AstNode::ConditionalExpression(cond) => cond.test,
            _ => return,
        };

        let Some(test_node) = ctx.node(test_id) else {
            return;
        };

        if !is_double_negation(ctx, test_node) && !is_boolean_call(ctx, test_node) {
            return;
        }

        let inner_span = unwrap_boolean_cast(ctx, test_node);
        let inner_text = ctx
            .source_text()
            .get(inner_span.start as usize..inner_span.end as usize)
            .unwrap_or("x");

        let test_span = test_node.span();
        ctx.report(Diagnostic {
            rule_name: "no-extra-boolean-cast".to_owned(),
            message: "Redundant double negation in boolean context".to_owned(),
            span: Span::new(test_span.start, test_span.end),
            severity: Severity::Warning,
            help: Some("Remove the unnecessary boolean cast".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove unnecessary boolean cast".to_owned(),
                edits: vec![Edit {
                    span: Span::new(test_span.start, test_span.end),
                    replacement: inner_text.to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Check if node is `!!x`.
fn is_double_negation(ctx: &LintContext<'_>, node: &AstNode) -> bool {
    let AstNode::UnaryExpression(outer) = node else {
        return false;
    };
    if outer.operator != UnaryOperator::LogicalNot {
        return false;
    }
    matches!(
        ctx.node(outer.argument),
        Some(AstNode::UnaryExpression(inner)) if inner.operator == UnaryOperator::LogicalNot
    )
}

/// Check if node is `Boolean(x)`.
fn is_boolean_call(ctx: &LintContext<'_>, node: &AstNode) -> bool {
    let AstNode::CallExpression(call) = node else {
        return false;
    };
    matches!(
        ctx.node(call.callee),
        Some(AstNode::IdentifierReference(id)) if id.name == "Boolean"
    )
}

/// Extract the span of the inner expression from `!!x` or `Boolean(x)`.
fn unwrap_boolean_cast(ctx: &LintContext<'_>, node: &AstNode) -> Span {
    // !!x → get inner.argument span
    if let AstNode::UnaryExpression(outer) = node {
        if outer.operator == UnaryOperator::LogicalNot {
            if let Some(AstNode::UnaryExpression(inner)) = ctx.node(outer.argument) {
                if inner.operator == UnaryOperator::LogicalNot {
                    if let Some(arg_node) = ctx.node(inner.argument) {
                        let s = arg_node.span();
                        return Span::new(s.start, s.end);
                    }
                }
            }
        }
    }
    // Boolean(x) → get first argument span
    if let AstNode::CallExpression(call) = node {
        if let Some(first_arg_id) = call.arguments.first() {
            if let Some(arg_node) = ctx.node(*first_arg_id) {
                let s = arg_node.span();
                return Span::new(s.start, s.end);
            }
        }
    }
    // Fallback: return the node's own span
    let s = node.span();
    Span::new(s.start, s.end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExtraBooleanCast)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_double_negation_in_if() {
        let diags = lint("if (!!x) {}");
        assert_eq!(diags.len(), 1, "!!x in if condition should be flagged");
    }

    #[test]
    fn test_flags_boolean_call_in_if() {
        let diags = lint("if (Boolean(x)) {}");
        assert_eq!(
            diags.len(),
            1,
            "Boolean(x) in if condition should be flagged"
        );
    }

    #[test]
    fn test_allows_simple_condition() {
        let diags = lint("if (x) {}");
        assert!(diags.is_empty(), "simple condition should not be flagged");
    }

    #[test]
    fn test_flags_double_negation_in_ternary() {
        let diags = lint("var r = !!x ? 1 : 0;");
        assert_eq!(diags.len(), 1, "!!x in ternary should be flagged");
    }

    #[test]
    fn test_allows_single_negation() {
        let diags = lint("if (!x) {}");
        assert!(diags.is_empty(), "single negation should not be flagged");
    }
}
