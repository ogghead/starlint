//! Rule: `typescript/no-non-null-assertion`
//!
//! Disallow non-null assertions using the `!` postfix operator. The non-null
//! assertion operator (`x!`) tells `TypeScript` to treat a value as non-null
//! without any runtime check, which can mask potential `null`/`undefined` bugs.
//! Prefer optional chaining (`?.`) or explicit null checks instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags non-null assertion expressions (`!` postfix operator).
#[derive(Debug)]
pub struct NoNonNullAssertion;

impl LintRule for NoNonNullAssertion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-non-null-assertion".to_owned(),
            description: "Disallow non-null assertions using the `!` postfix operator".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSNonNullExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSNonNullExpression(expr) = node else {
            return;
        };

        // Fix: replace `x!` with `x`
        let inner_node_span = ctx.node(expr.expression).map(starlint_ast::AstNode::span);
        let (inner_start, inner_end) = inner_node_span.map_or((0, 0), |s| (s.start, s.end));
        let inner_text = ctx.source_text()[inner_start as usize..inner_end as usize].to_owned();

        ctx.report(Diagnostic {
            rule_name: "typescript/no-non-null-assertion".to_owned(),
            message:
                "Avoid non-null assertions — use optional chaining or explicit null checks instead"
                    .to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some("Remove the `!` non-null assertion".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove non-null assertion".to_owned(),
                edits: vec![Edit {
                    span: Span::new(expr.span.start, expr.span.end),
                    replacement: inner_text,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNonNullAssertion)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_non_null_member_access() {
        let diags = lint("declare const x: { foo: string } | null; x!.foo;");
        assert_eq!(
            diags.len(),
            1,
            "`x!.foo` non-null assertion should be flagged"
        );
    }

    #[test]
    fn test_flags_non_null_standalone() {
        let diags = lint("declare const x: string | null; x!;");
        assert_eq!(
            diags.len(),
            1,
            "standalone `x!` non-null assertion should be flagged"
        );
    }

    #[test]
    fn test_allows_optional_chaining() {
        let diags = lint("declare const x: { foo: string } | null; x?.foo;");
        assert!(diags.is_empty(), "optional chaining should not be flagged");
    }

    #[test]
    fn test_allows_normal_member_access() {
        let diags = lint("declare const x: { foo: string }; x.foo;");
        assert!(
            diags.is_empty(),
            "normal member access should not be flagged"
        );
    }
}
