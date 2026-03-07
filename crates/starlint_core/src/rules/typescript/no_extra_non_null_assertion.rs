//! Rule: `typescript/no-extra-non-null-assertion`
//!
//! Disallow extra non-null assertions. Writing `x!!` applies two `!` postfix
//! operators, but the second assertion is always redundant — if `x!` is
//! non-null, asserting it again adds no value and suggests a typo.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `TSNonNullExpression` nodes whose inner expression is also a
/// `TSNonNullExpression` (i.e. `x!!`).
#[derive(Debug)]
pub struct NoExtraNonNullAssertion;

impl LintRule for NoExtraNonNullAssertion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-extra-non-null-assertion".to_owned(),
            description: "Disallow extra non-null assertions (`!!`)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSNonNullExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSNonNullExpression(expr) = node else {
            return;
        };

        let inner_node = ctx.node(expr.expression);
        if matches!(inner_node, Some(AstNode::TSNonNullExpression(_))) {
            // Replace `x!!` with `x!` by keeping only the inner expression's span
            let inner_span = inner_node.map_or(Span::new(0, 0), |n| {
                let s = n.span();
                Span::new(s.start, s.end)
            });
            let inner_start = usize::try_from(inner_span.start).unwrap_or(0);
            let inner_end = usize::try_from(inner_span.end).unwrap_or(0);
            let inner_text = ctx.source_text().get(inner_start..inner_end).unwrap_or("");

            ctx.report(Diagnostic {
                rule_name: "typescript/no-extra-non-null-assertion".to_owned(),
                message: "Extra non-null assertion — `x!` is sufficient, `x!!` is redundant"
                    .to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: Some("Remove the extra `!` assertion".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove the extra `!` assertion".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement: inner_text.to_owned(),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExtraNonNullAssertion)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_double_non_null() {
        let diags = lint("declare const x: string | null; x!!;");
        assert_eq!(diags.len(), 1, "`x!!` should be flagged");
    }

    #[test]
    fn test_flags_double_non_null_with_member_access() {
        let diags = lint("declare const x: { foo: string } | null; x!!.foo;");
        assert_eq!(diags.len(), 1, "`x!!.foo` should be flagged");
    }

    #[test]
    fn test_allows_single_non_null() {
        let diags = lint("declare const x: string | null; x!;");
        assert!(diags.is_empty(), "single `x!` should not be flagged");
    }

    #[test]
    fn test_allows_single_non_null_with_member_access() {
        let diags = lint("declare const x: { foo: string } | null; x!.foo;");
        assert!(diags.is_empty(), "`x!.foo` should not be flagged");
    }
}
