//! Rule: `typescript/no-confusing-non-null-assertion`
//!
//! Disallow non-null assertions (`!`) in positions where they can be confused
//! with comparison operators. Writing `x! == y` or `x! === y` is visually
//! confusing because the `!` blends with the equality operator. The reader
//! may interpret it as `x !== y` instead of `(x!) == y`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags binary equality expressions where the left operand is a
/// `TSNonNullExpression`, making the `!` look like part of `!=` or `!==`.
#[derive(Debug)]
pub struct NoConfusingNonNullAssertion;

impl LintRule for NoConfusingNonNullAssertion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-confusing-non-null-assertion".to_owned(),
            description:
                "Disallow non-null assertions in confusing positions next to equality operators"
                    .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        if !expr.operator.is_equality() {
            return;
        }

        if let Some(AstNode::TSNonNullExpression(non_null)) = ctx.node(expr.left) {
            // Wrap the non-null assertion in parentheses: `x! == y` → `(x!) == y`
            let source = ctx.source_text();
            let non_null_span = non_null.span;
            let left_start = non_null_span.start as usize;
            let left_end = non_null_span.end as usize;
            let left_text = source.get(left_start..left_end).unwrap_or("");
            let replacement = format!("({left_text})");

            let expr_span_start = expr.span.start;
            let expr_span_end = expr.span.end;

            ctx.report(Diagnostic {
                rule_name: "typescript/no-confusing-non-null-assertion".to_owned(),
                message: "Non-null assertion `!` next to an equality operator is confusing — it may look like `!=` or `!==`".to_owned(),
                span: Span::new(expr_span_start, expr_span_end),
                severity: Severity::Warning,
                help: Some("Wrap the non-null assertion in parentheses to clarify intent".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Wrap in parentheses: `(x!)`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(non_null_span.start, non_null_span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoConfusingNonNullAssertion)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_non_null_before_equality() {
        let diags = lint("declare const x: number | null; x! == 1;");
        assert_eq!(diags.len(), 1, "`x! == 1` should be flagged as confusing");
    }

    #[test]
    fn test_flags_non_null_before_strict_equality() {
        let diags = lint("declare const x: number | null; x! === 1;");
        assert_eq!(diags.len(), 1, "`x! === 1` should be flagged as confusing");
    }

    #[test]
    fn test_flags_non_null_before_inequality() {
        let diags = lint("declare const x: number | null; x! != 1;");
        assert_eq!(diags.len(), 1, "`x! != 1` should be flagged as confusing");
    }

    #[test]
    fn test_allows_normal_equality() {
        let diags = lint("const x = 1; x == 1;");
        assert!(diags.is_empty(), "normal equality should not be flagged");
    }

    #[test]
    fn test_allows_strict_inequality() {
        let diags = lint("const x = 1; x !== null;");
        assert!(
            diags.is_empty(),
            "normal strict inequality should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_null_in_non_equality() {
        let diags = lint("declare const x: number | null; const y = x! + 1;");
        assert!(
            diags.is_empty(),
            "non-null assertion with arithmetic should not be flagged"
        );
    }
}
