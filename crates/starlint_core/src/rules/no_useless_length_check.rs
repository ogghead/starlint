//! Rule: `no-useless-length-check` (unicorn)
//!
//! Disallow useless `.length` checks before calling iteration methods.
//! For example, `if (arr.length > 0) { arr.forEach(...) }` is unnecessary
//! because `.forEach()` on an empty array is a no-op.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags useless `.length` checks before iteration methods.
#[derive(Debug)]
pub struct NoUselessLengthCheck;

/// Iteration methods that are no-ops on empty arrays.
const SAFE_ITERATION_METHODS: &[&str] = &[
    "forEach",
    "map",
    "filter",
    "some",
    "every",
    "find",
    "findIndex",
    "flatMap",
    "reduce",
    "reduceRight",
    "flat",
    "fill",
    "copyWithin",
    "entries",
    "keys",
    "values",
];

impl LintRule for NoUselessLengthCheck {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-length-check".to_owned(),
            description: "Disallow useless .length check before iteration".to_owned(),
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

        // Check if the condition is `arr.length > 0` or `arr.length !== 0`
        // or `arr.length` (truthy check)
        let Some(array_name) = get_length_check_target(if_stmt.test, ctx) else {
            return;
        };

        // Check if the body only contains iteration method calls on the same array
        if body_only_calls_iteration_method(if_stmt.consequent, &array_name, ctx) {
            let if_span = Span::new(if_stmt.span.start, if_stmt.span.end);
            // Extract the body text (the consequent statement)
            let body_text = extract_body_text(if_stmt.consequent, ctx);
            ctx.report(Diagnostic {
                rule_name: "no-useless-length-check".to_owned(),
                message: "The `.length` check is unnecessary; iteration methods are no-ops on empty arrays".to_owned(),
                span: if_span,
                severity: Severity::Warning,
                help: Some("Remove the `.length` check and keep the iteration call".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove `.length` check".to_owned(),
                    edits: vec![Edit {
                        span: if_span,
                        replacement: body_text,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Extract the body text from an if-statement consequent.
/// For a block statement like `{ arr.forEach(fn); }`, returns `arr.forEach(fn);`.
/// For a bare expression statement, returns the full statement text.
fn extract_body_text(stmt_id: NodeId, ctx: &LintContext<'_>) -> String {
    let source = ctx.source_text();
    match ctx.node(stmt_id) {
        Some(AstNode::BlockStatement(block)) => {
            if let Some(&inner_id) = block.body.first() {
                let inner_span = ctx.node(inner_id).map_or(
                    starlint_ast::types::Span::EMPTY,
                    starlint_ast::AstNode::span,
                );
                let start = usize::try_from(inner_span.start).unwrap_or(0);
                let end = usize::try_from(inner_span.end).unwrap_or(0);
                source.get(start..end).unwrap_or("").to_owned()
            } else {
                String::new()
            }
        }
        Some(node) => {
            let span = node.span();
            let start = usize::try_from(span.start).unwrap_or(0);
            let end = usize::try_from(span.end).unwrap_or(0);
            source.get(start..end).unwrap_or("").to_owned()
        }
        None => String::new(),
    }
}

/// Extract the array name from a `.length` check expression.
fn get_length_check_target(expr_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    match ctx.node(expr_id)? {
        // `arr.length` (truthy check)
        AstNode::StaticMemberExpression(member) if member.property == "length" => {
            if let Some(AstNode::IdentifierReference(id)) = ctx.node(member.object) {
                Some(id.name.clone())
            } else {
                None
            }
        }
        // `arr.length > 0`, `arr.length !== 0`, etc.
        AstNode::BinaryExpression(bin) => {
            let Some(AstNode::StaticMemberExpression(member)) = ctx.node(bin.left) else {
                return None;
            };

            if member.property != "length" {
                return None;
            }

            let Some(AstNode::IdentifierReference(id)) = ctx.node(member.object) else {
                return None;
            };

            let name = id.name.clone();

            // Right side should be 0
            let Some(AstNode::NumericLiteral(num)) = ctx.node(bin.right) else {
                return None;
            };

            #[allow(clippy::float_cmp)]
            (num.value == 0.0).then_some(name)
        }
        _ => None,
    }
}

/// Check if a statement body only calls iteration methods on the given array.
fn body_only_calls_iteration_method(
    stmt_id: NodeId,
    array_name: &str,
    ctx: &LintContext<'_>,
) -> bool {
    match ctx.node(stmt_id) {
        Some(AstNode::BlockStatement(block)) => {
            block.body.len() == 1
                && block
                    .body
                    .first()
                    .is_some_and(|&s| body_only_calls_iteration_method(s, array_name, ctx))
        }
        Some(AstNode::ExpressionStatement(expr_stmt)) => {
            is_iteration_call(expr_stmt.expression, array_name, ctx)
        }
        _ => false,
    }
}

/// Check if an expression is `arr.forEach(...)`, `arr.map(...)`, etc.
fn is_iteration_call(expr_id: NodeId, array_name: &str, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::CallExpression(call)) = ctx.node(expr_id) else {
        return false;
    };

    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
        return false;
    };

    let Some(AstNode::IdentifierReference(obj)) = ctx.node(member.object) else {
        return false;
    };

    obj.name == array_name && SAFE_ITERATION_METHODS.contains(&member.property.as_str())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUselessLengthCheck)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_length_check_before_foreach() {
        let diags = lint("if (arr.length > 0) { arr.forEach(fn); }");
        assert_eq!(
            diags.len(),
            1,
            "length check before forEach should be flagged"
        );
    }

    #[test]
    fn test_flags_truthy_length_check() {
        let diags = lint("if (arr.length) { arr.map(fn); }");
        assert_eq!(
            diags.len(),
            1,
            "truthy length check before map should be flagged"
        );
    }

    #[test]
    fn test_allows_length_check_with_other_code() {
        let diags = lint("if (arr.length > 0) { console.log('has items'); }");
        assert!(
            diags.is_empty(),
            "length check with non-iteration code should not be flagged"
        );
    }

    #[test]
    fn test_allows_without_length_check() {
        let diags = lint("arr.forEach(fn);");
        assert!(
            diags.is_empty(),
            "forEach without length check should not be flagged"
        );
    }
}
