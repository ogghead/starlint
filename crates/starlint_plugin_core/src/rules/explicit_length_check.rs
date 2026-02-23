//! Rule: `explicit-length-check`
//!
//! Require explicit comparison when checking `.length` or `.size`.
//! Truthy/falsy checks on `.length` are confusing because `0` is falsy
//! but is a valid length. Prefer `arr.length > 0` or `arr.length === 0`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Property names that should be compared explicitly.
const LENGTH_PROPERTIES: &[&str] = &["length", "size"];

/// Flags implicit truthy/falsy checks on `.length` or `.size`.
#[derive(Debug)]
pub struct ExplicitLengthCheck;

impl LintRule for ExplicitLengthCheck {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "explicit-length-check".to_owned(),
            description: "Require explicit comparison when checking `.length` or `.size`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ConditionalExpression,
            AstNodeType::IfStatement,
            AstNodeType::WhileStatement,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let (test_id, container_span) = match node {
            AstNode::IfStatement(stmt) => (stmt.test, stmt.span),
            AstNode::WhileStatement(stmt) => (stmt.test, stmt.span),
            AstNode::ConditionalExpression(expr) => (expr.test, expr.span),
            _ => return,
        };

        check_condition(test_id, container_span, ctx);
    }
}

/// Check a condition expression for implicit `.length`/`.size` usage.
fn check_condition(
    test_id: NodeId,
    container_span: starlint_ast::types::Span,
    ctx: &mut LintContext<'_>,
) {
    let report_span = Span::new(container_span.start, container_span.end);

    let Some(expr) = ctx.node(test_id) else {
        return;
    };

    // Case 1: `if (foo.length)` — direct member expression as condition
    if is_length_or_size_member(expr) {
        // Fix: replace `foo.length` with `foo.length > 0`
        let expr_span = expr.span();
        let member_text = ctx
            .source_text()
            .get(
                usize::try_from(expr_span.start).unwrap_or(0)
                    ..usize::try_from(expr_span.end).unwrap_or(0),
            )
            .unwrap_or("")
            .to_owned();
        let condition_span = Span::new(expr_span.start, expr_span.end);
        ctx.report(Diagnostic {
            rule_name: "explicit-length-check".to_owned(),
            message: "Use an explicit comparison (`> 0` or `=== 0`) instead of a truthy check on `.length`/`.size`".to_owned(),
            span: report_span,
            severity: Severity::Warning,
            help: Some("Use `> 0` for a non-empty check".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace with `> 0` comparison".to_owned(),
                edits: vec![Edit {
                    span: condition_span,
                    replacement: format!("{member_text} > 0"),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
        return;
    }

    // Case 2: `if (!foo.length)` — negated member expression
    if let AstNode::UnaryExpression(unary) = expr {
        if unary.operator == UnaryOperator::LogicalNot
            && is_length_or_size_member_id(unary.argument, ctx)
        {
            // Fix: replace `!foo.length` with `foo.length === 0`
            let inner_ast_span = ctx.node(unary.argument).map(starlint_ast::AstNode::span);
            let inner_span = inner_ast_span.map_or(Span::new(0, 0), |s| Span::new(s.start, s.end));
            let member_text = ctx
                .source_text()
                .get(
                    usize::try_from(inner_span.start).unwrap_or(0)
                        ..usize::try_from(inner_span.end).unwrap_or(0),
                )
                .unwrap_or("")
                .to_owned();
            // Replace the whole unary expression `!foo.length` with `foo.length === 0`
            let unary_span = Span::new(unary.span.start, unary.span.end);
            ctx.report(Diagnostic {
                rule_name: "explicit-length-check".to_owned(),
                message: "Use `=== 0` instead of negating `.length`/`.size`".to_owned(),
                span: report_span,
                severity: Severity::Warning,
                help: Some("Use `=== 0` for an empty check".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace with `=== 0` comparison".to_owned(),
                    edits: vec![Edit {
                        span: unary_span,
                        replacement: format!("{member_text} === 0"),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if an `AstNode` is a static member access to `.length` or `.size`.
fn is_length_or_size_member(expr: &AstNode) -> bool {
    let AstNode::StaticMemberExpression(member) = expr else {
        return false;
    };
    let name = member.property.as_str();
    LENGTH_PROPERTIES.contains(&name)
}

/// Check if a `NodeId` resolves to a `.length` or `.size` member expression.
fn is_length_or_size_member_id(id: NodeId, ctx: &LintContext<'_>) -> bool {
    ctx.node(id).is_some_and(is_length_or_size_member)
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ExplicitLengthCheck)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_truthy_length() {
        let diags = lint("if (arr.length) {}");
        assert_eq!(diags.len(), 1, "truthy .length check should be flagged");
    }

    #[test]
    fn test_allows_explicit_greater_than() {
        let diags = lint("if (arr.length > 0) {}");
        assert!(
            diags.is_empty(),
            "explicit > 0 comparison should not be flagged"
        );
    }

    #[test]
    fn test_flags_negated_length() {
        let diags = lint("if (!arr.length) {}");
        assert_eq!(diags.len(), 1, "negated .length check should be flagged");
    }

    #[test]
    fn test_allows_explicit_equals_zero() {
        let diags = lint("if (arr.length === 0) {}");
        assert!(
            diags.is_empty(),
            "explicit === 0 comparison should not be flagged"
        );
    }

    #[test]
    fn test_flags_truthy_size() {
        let diags = lint("if (map.size) {}");
        assert_eq!(diags.len(), 1, "truthy .size check should be flagged");
    }

    #[test]
    fn test_allows_not_equals_zero() {
        let diags = lint("if (str.length !== 0) {}");
        assert!(
            diags.is_empty(),
            "explicit !== 0 comparison should not be flagged"
        );
    }

    #[test]
    fn test_flags_while_truthy_length() {
        let diags = lint("while (arr.length) {}");
        assert_eq!(diags.len(), 1, "truthy .length in while should be flagged");
    }

    #[test]
    fn test_flags_ternary_truthy_length() {
        let diags = lint("var x = arr.length ? 'yes' : 'no';");
        assert_eq!(
            diags.len(),
            1,
            "truthy .length in ternary should be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_property() {
        let diags = lint("if (arr.count) {}");
        assert!(diags.is_empty(), "unrelated property should not be flagged");
    }
}
