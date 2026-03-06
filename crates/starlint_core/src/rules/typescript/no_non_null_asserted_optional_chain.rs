//! Rule: `typescript/no-non-null-asserted-optional-chain`
//!
//! Disallow non-null assertions after an optional chain expression. Using `!`
//! after `?.` contradicts the intent of optional chaining — the `?.` says "this
//! might be nullish", while `!` says "this is definitely not nullish". This is
//! almost always a mistake or a misunderstanding of how optional chaining works.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-non-null-asserted-optional-chain";

/// Flags `TSNonNullExpression` wrapping an optional chain (e.g. `foo?.bar!`).
#[derive(Debug)]
pub struct NoNonNullAssertedOptionalChain;

/// Check if an expression uses optional chaining.
fn is_optional_chain(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::ChainExpression(_) => true,
        // oxc may represent `foo?.bar` as a direct member/call expression
        // with `optional: true` rather than wrapping in `ChainExpression`.
        Expression::CallExpression(call) => call.optional,
        Expression::StaticMemberExpression(m) => m.optional,
        Expression::ComputedMemberExpression(m) => m.optional,
        Expression::PrivateFieldExpression(m) => m.optional,
        _ => false,
    }
}

impl NativeRule for NoNonNullAssertedOptionalChain {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow non-null assertions after an optional chain expression"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSNonNullExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSNonNullExpression(expr) = kind else {
            return;
        };

        if is_optional_chain(&expr.expression) {
            // Remove the `!` by replacing the whole expression with the inner expression text
            let inner_start = usize::try_from(expr.expression.span().start).unwrap_or(0);
            let inner_end = usize::try_from(expr.expression.span().end).unwrap_or(0);
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNonNullAssertedOptionalChain)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

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
