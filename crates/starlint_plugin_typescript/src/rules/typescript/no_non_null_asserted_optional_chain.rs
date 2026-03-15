//! Rule: `typescript/no-non-null-asserted-optional-chain`
//!
//! Disallow non-null assertions after an optional chain expression. Using `!`
//! after `?.` contradicts the intent of optional chaining — the `?.` says "this
//! might be nullish", while `!` says "this is definitely not nullish". This is
//! almost always a mistake or a misunderstanding of how optional chaining works.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-non-null-asserted-optional-chain";

/// Flags `TSNonNullExpression` wrapping an optional chain (e.g. `foo?.bar!`).
#[derive(Debug)]
pub struct NoNonNullAssertedOptionalChain;

/// Check if an expression node uses optional chaining.
fn is_optional_chain(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(expr_id) {
        Some(AstNode::ChainExpression(_)) => true,
        // oxc may represent `foo?.bar` as a direct member/call expression
        // with `optional: true` rather than wrapping in `ChainExpression`.
        Some(AstNode::CallExpression(call)) => call.optional,
        Some(AstNode::StaticMemberExpression(m)) => m.optional,
        Some(AstNode::ComputedMemberExpression(m)) => m.optional,
        _ => false,
    }
}

impl LintRule for NoNonNullAssertedOptionalChain {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow non-null assertions after an optional chain expression"
                .to_owned(),
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

        if is_optional_chain(expr.expression, ctx) {
            // Remove the `!` by replacing the whole expression with the inner expression text
            let inner_span = ctx.node(expr.expression).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            let inner_start = usize::try_from(inner_span.start).unwrap_or(0);
            let inner_end = usize::try_from(inner_span.end).unwrap_or(0);
            let inner_text = ctx.source_text().get(inner_start..inner_end).unwrap_or("");

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Non-null assertion after optional chain is contradictory — remove `!` or `?.`"
                        .to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: Some("Remove the `!` non-null assertion".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove the `!` non-null assertion".to_owned(),
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

    starlint_rule_framework::lint_rule_test!(NoNonNullAssertedOptionalChain, "test.ts");

    #[test]
    fn test_flags_non_null_after_optional_chain() {
        let diags = lint("declare const foo: { bar: string } | null; foo?.bar!;");
        assert_eq!(
            diags.len(),
            1,
            "`foo?.bar!` should be flagged as contradictory"
        );
    }

    #[test]
    fn test_flags_non_null_after_optional_call() {
        let diags = lint("declare const foo: (() => string) | null; foo?.()!;");
        assert_eq!(
            diags.len(),
            1,
            "`foo?.()!` should be flagged as contradictory"
        );
    }

    #[test]
    fn test_allows_optional_chain_without_assertion() {
        let diags = lint("declare const foo: { bar: string } | null; foo?.bar;");
        assert!(
            diags.is_empty(),
            "`foo?.bar` without `!` should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_null_without_optional_chain() {
        let diags = lint("declare const foo: { bar: string }; foo.bar!;");
        assert!(
            diags.is_empty(),
            "`foo.bar!` without `?.` should not be flagged"
        );
    }

    #[test]
    fn test_allows_plain_member_access() {
        let diags = lint("declare const foo: { bar: string }; foo.bar;");
        assert!(
            diags.is_empty(),
            "plain member access should not be flagged"
        );
    }
}
