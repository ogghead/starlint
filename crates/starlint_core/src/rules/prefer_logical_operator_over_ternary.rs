//! Rule: `prefer-logical-operator-over-ternary`
//!
//! Prefer `??` / `||` over ternary when the test is a simple
//! truthiness/nullishness check. `a ? a : b` should be `a || b`, and
//! `a !== null ? a : b` / `a !== undefined ? a : b` should be `a ?? b`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;

/// Flags ternary expressions that can be replaced with `||` or `??`.
#[derive(Debug)]
pub struct PreferLogicalOperatorOverTernary;

impl LintRule for PreferLogicalOperatorOverTernary {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-logical-operator-over-ternary".to_owned(),
            description: "Prefer `??` / `||` over ternary for truthiness/nullish checks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ConditionalExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ConditionalExpression(cond) = node else {
            return;
        };

        let source = ctx.source_text();

        // Pattern 1: `a ? a : b` => `a || b`
        if let Some(operator) = check_simple_truthiness(cond.test, cond.consequent, source, ctx) {
            let test_text = node_text(cond.test, source, ctx)
                .unwrap_or_default()
                .to_owned();
            let alt_text = node_text(cond.alternate, source, ctx)
                .unwrap_or_default()
                .to_owned();
            let replacement = format!("{test_text} {operator} {alt_text}");

            ctx.report(Diagnostic {
                rule_name: "prefer-logical-operator-over-ternary".to_owned(),
                message: format!("Use `{operator}` instead of a ternary expression"),
                span: Span::new(cond.span.start, cond.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace with `{operator}`")),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace ternary with `{operator}`"),
                    edits: vec![Edit {
                        span: Span::new(cond.span.start, cond.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
            return;
        }

        // Pattern 2: `a !== null ? a : b` => `a ?? b`
        if let Some(value_text) = check_nullish_value(cond.test, cond.consequent, source, ctx) {
            let alt_text = node_text(cond.alternate, source, ctx)
                .unwrap_or_default()
                .to_owned();
            let replacement = format!("{value_text} ?? {alt_text}");

            ctx.report(Diagnostic {
                rule_name: "prefer-logical-operator-over-ternary".to_owned(),
                message: "Use `??` instead of a ternary expression for nullish checks".to_owned(),
                span: Span::new(cond.span.start, cond.span.end),
                severity: Severity::Warning,
                help: Some("Replace with `??`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Replace ternary with `??`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(cond.span.start, cond.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Extract a slice of source text for a given node by ID.
fn node_text<'s>(id: NodeId, source: &'s str, ctx: &LintContext<'_>) -> Option<&'s str> {
    let node = ctx.node(id)?;
    let sp = node.span();
    let start = usize::try_from(sp.start).ok()?;
    let end = usize::try_from(sp.end).ok()?;
    source.get(start..end)
}

/// Check `a ? a : b` pattern (test == consequent by source text).
fn check_simple_truthiness(
    test_id: NodeId,
    consequent_id: NodeId,
    source: &str,
    ctx: &LintContext<'_>,
) -> Option<&'static str> {
    // Skip if the test is a binary expression (those are comparisons, not simple truthiness)
    if matches!(ctx.node(test_id), Some(AstNode::BinaryExpression(_))) {
        return None;
    }

    let test_text = node_text(test_id, source, ctx)?;
    let cons_text = node_text(consequent_id, source, ctx)?;

    (!test_text.is_empty() && test_text == cons_text).then_some("||")
}

/// Check whether a node is `null` or `undefined`.
fn is_nullish_literal(id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(id) {
        Some(AstNode::NullLiteral(_)) => true,
        Some(AstNode::IdentifierReference(ident)) => ident.name.as_str() == "undefined",
        _ => false,
    }
}

/// Check `a !== null ? a : b` or `a !== undefined ? a : b` or `a != null ? a : b`.
/// Returns the value expression text if the pattern matches.
fn check_nullish_value<'s>(
    test_id: NodeId,
    consequent_id: NodeId,
    source: &'s str,
    ctx: &LintContext<'_>,
) -> Option<&'s str> {
    let Some(AstNode::BinaryExpression(binary)) = ctx.node(test_id) else {
        return None;
    };

    // Must be `!==` or `!=`
    if !matches!(
        binary.operator,
        BinaryOperator::StrictInequality | BinaryOperator::Inequality
    ) {
        return None;
    }

    // Determine which side is the value and which is null/undefined
    let value_id = if is_nullish_literal(binary.right, ctx) {
        binary.left
    } else if is_nullish_literal(binary.left, ctx) {
        binary.right
    } else {
        return None;
    };

    // The value side should match the consequent
    let value_text = node_text(value_id, source, ctx)?;
    let cons_text = node_text(consequent_id, source, ctx)?;

    (!value_text.is_empty() && value_text == cons_text).then_some(value_text)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferLogicalOperatorOverTernary)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_truthiness_ternary() {
        let diags = lint("const x = a ? a : b;");
        assert_eq!(diags.len(), 1, "a ? a : b should be flagged (use ||)");
    }

    #[test]
    fn test_flags_not_null_ternary() {
        let diags = lint("const x = a !== null ? a : b;");
        assert_eq!(
            diags.len(),
            1,
            "a !== null ? a : b should be flagged (use ??)"
        );
    }

    #[test]
    fn test_flags_not_undefined_ternary() {
        let diags = lint("const x = a !== undefined ? a : b;");
        assert_eq!(
            diags.len(),
            1,
            "a !== undefined ? a : b should be flagged (use ??)"
        );
    }

    #[test]
    fn test_flags_loose_not_null_ternary() {
        let diags = lint("const x = a != null ? a : b;");
        assert_eq!(
            diags.len(),
            1,
            "a != null ? a : b should be flagged (use ??)"
        );
    }

    #[test]
    fn test_allows_different_consequent() {
        let diags = lint("const x = a ? b : c;");
        assert!(
            diags.is_empty(),
            "different consequent should not be flagged"
        );
    }

    #[test]
    fn test_allows_logical_or() {
        let diags = lint("const x = a || b;");
        assert!(diags.is_empty(), "already using || should not be flagged");
    }

    #[test]
    fn test_allows_comparison_ternary() {
        let diags = lint("const x = a > 0 ? a : 0;");
        assert!(
            diags.is_empty(),
            "comparison-based ternary should not be flagged by this rule"
        );
    }
}
