//! Rule: `prefer-type-error` (unicorn)
//!
//! Prefer throwing `TypeError` in type-checking `if` statements.
//! When checking the type of a value, throwing a `TypeError` is more
//! appropriate than a generic `Error`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UnaryOperator};
use starlint_ast::types::NodeId;

/// Flags throw statements in type-checking blocks that throw `Error`
/// instead of `TypeError`.
#[derive(Debug)]
pub struct PreferTypeError;

impl LintRule for PreferTypeError {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-type-error".to_owned(),
            description: "Prefer TypeError for type checking".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IfStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IfStatement(if_stmt) = node else {
            return;
        };

        // Check if the condition is a type check (typeof, instanceof)
        let Some(test_node) = ctx.node(if_stmt.test) else {
            return;
        };
        if !is_type_check(ctx, test_node) {
            return;
        }

        // Check if the body throws a generic Error
        let Some(cons_node) = ctx.node(if_stmt.consequent) else {
            return;
        };
        if let Some(error_id_span) = find_error_callee_span(ctx, cons_node) {
            let span = Span::new(if_stmt.span.start, if_stmt.span.end);
            ctx.report(Diagnostic {
                rule_name: "prefer-type-error".to_owned(),
                message: "Use `new TypeError()` instead of `new Error()` for type checks"
                    .to_owned(),
                span,
                severity: Severity::Warning,
                help: Some("Replace `Error` with `TypeError`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace `Error` with `TypeError`".to_owned(),
                    edits: vec![Edit {
                        span: error_id_span,
                        replacement: "TypeError".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if an expression node is a type check (typeof x === '...' or x instanceof Y).
fn is_type_check(ctx: &LintContext<'_>, node: &AstNode) -> bool {
    match node {
        AstNode::BinaryExpression(bin) => {
            // typeof x === '...'
            let left_is_typeof = ctx.node(bin.left).is_some_and(
                |n| matches!(n, AstNode::UnaryExpression(u) if u.operator == UnaryOperator::Typeof),
            );
            let right_is_typeof = ctx.node(bin.right).is_some_and(
                |n| matches!(n, AstNode::UnaryExpression(u) if u.operator == UnaryOperator::Typeof),
            );
            // x instanceof Y
            let is_instanceof = bin.operator == BinaryOperator::Instanceof;

            left_is_typeof || right_is_typeof || is_instanceof
        }
        AstNode::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => ctx
            .node(unary.argument)
            .is_some_and(|n| is_type_check(ctx, n)),
        AstNode::LogicalExpression(logical) => {
            let left_is = ctx
                .node(logical.left)
                .is_some_and(|n| is_type_check(ctx, n));
            let right_is = ctx
                .node(logical.right)
                .is_some_and(|n| is_type_check(ctx, n));
            left_is || right_is
        }
        _ => false,
    }
}

/// Find the span of the `Error` identifier in a `throw new Error(...)` statement.
/// Returns `None` if the statement doesn't throw a generic `Error`.
fn find_error_callee_span(ctx: &LintContext<'_>, node: &AstNode) -> Option<Span> {
    match node {
        AstNode::BlockStatement(block) => {
            if block.body.len() == 1 {
                let first_id = *block.body.first()?;
                let first_node = ctx.node(first_id)?;
                find_error_callee_span(ctx, first_node)
            } else {
                None
            }
        }
        AstNode::ThrowStatement(throw) => {
            let new_node = ctx.node(throw.argument)?;
            if let AstNode::NewExpression(new_expr) = new_node {
                let callee_node = ctx.node(new_expr.callee)?;
                if let AstNode::IdentifierReference(id) = callee_node {
                    if id.name == "Error" {
                        return Some(Span::new(id.span.start, id.span.end));
                    }
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferTypeError)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_typeof_with_error() {
        let diags = lint("if (typeof x !== 'string') { throw new Error('msg'); }");
        assert_eq!(diags.len(), 1, "typeof check with Error should be flagged");
    }

    #[test]
    fn test_allows_typeof_with_type_error() {
        let diags = lint("if (typeof x !== 'string') { throw new TypeError('msg'); }");
        assert!(
            diags.is_empty(),
            "typeof check with TypeError should not be flagged"
        );
    }

    #[test]
    fn test_flags_instanceof_with_error() {
        let diags = lint("if (x instanceof Foo) { throw new Error('msg'); }");
        assert_eq!(
            diags.len(),
            1,
            "instanceof check with Error should be flagged"
        );
    }

    #[test]
    fn test_allows_non_type_check() {
        let diags = lint("if (x > 0) { throw new Error('msg'); }");
        assert!(diags.is_empty(), "non-type-check should not be flagged");
    }
}
