//! Rule: `typescript/no-unsafe-type-assertion`
//!
//! Disallow type assertions that cast to `any` or `unknown`. Using `x as any`
//! or `x as unknown` are escape hatches that bypass TypeScript's type system.
//! These assertions hide potential type errors and make refactoring harder.
//! Prefer explicit type narrowing, generics, or proper type guards instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `as any` and `as unknown` type assertions.
#[derive(Debug)]
pub struct NoUnsafeTypeAssertion;

impl LintRule for NoUnsafeTypeAssertion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-type-assertion".to_owned(),
            description: "Disallow type assertions to `any` or `unknown`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSAsExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSAsExpression(expr) = node else {
            return;
        };

        // TSAsExpressionNode in starlint_ast has no type_annotation field.
        // Use source text heuristic to detect `as any` or `as unknown`.
        let source = ctx.source_text();
        let expr_text = source
            .get(expr.span.start as usize..expr.span.end as usize)
            .unwrap_or("");
        let escape_type = if expr_text.contains(" as any") || expr_text.ends_with("as any") {
            "any"
        } else if expr_text.contains(" as unknown") || expr_text.ends_with("as unknown") {
            "unknown"
        } else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "typescript/no-unsafe-type-assertion".to_owned(),
            message: format!(
                "Avoid `as {escape_type}` — it bypasses type checking. Use a type guard or explicit type instead"
            ),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUnsafeTypeAssertion, "test.ts");

    #[test]
    fn test_flags_as_any() {
        let diags = lint("let x = value as any;");
        assert_eq!(diags.len(), 1, "`as any` assertion should be flagged");
    }

    #[test]
    fn test_flags_as_unknown() {
        let diags = lint("let x = value as unknown;");
        assert_eq!(diags.len(), 1, "`as unknown` assertion should be flagged");
    }

    #[test]
    fn test_allows_as_string() {
        let diags = lint("let x = value as string;");
        assert!(
            diags.is_empty(),
            "`as string` assertion should not be flagged"
        );
    }

    #[test]
    fn test_allows_as_number() {
        let diags = lint("let x = value as number;");
        assert!(
            diags.is_empty(),
            "`as number` assertion should not be flagged"
        );
    }

    #[test]
    fn test_flags_nested_as_any() {
        let diags = lint("let x = (foo.bar() as any).baz;");
        assert_eq!(
            diags.len(),
            1,
            "nested `as any` assertion should be flagged"
        );
    }
}
