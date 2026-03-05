//! Rule: `jest/prefer-expect-resolves`
//!
//! Suggest `expect(promise).resolves.toBe()` over `expect(await promise).toBe()`.
//! Using `.resolves` provides better failure messages and makes the async
//! intent more explicit.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `expect(await promise)` in favor of `expect(promise).resolves`.
#[derive(Debug)]
pub struct PreferExpectResolves;

impl NativeRule for PreferExpectResolves {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-expect-resolves".to_owned(),
            description: "Suggest using `expect(...).resolves` instead of `expect(await ...)`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Must be `expect(...)` call
        let is_expect = matches!(
            &call.callee,
            Expression::Identifier(id) if id.name.as_str() == "expect"
        );
        if !is_expect {
            return;
        }

        // Check if the first argument is an await expression
        let Some(first_arg) = call.arguments.first() else {
            return;
        };
        let Some(arg_expr) = first_arg.as_expression() else {
            return;
        };
        let Expression::AwaitExpression(await_expr) = arg_expr else {
            return;
        };

        // Build fix: `expect(await expr)` → `await expect(expr).resolves`
        let source = ctx.source_text();
        let inner_span = await_expr.argument.span();
        let inner_text = source
            .get(inner_span.start as usize..inner_span.end as usize)
            .unwrap_or("")
            .to_owned();

        let fix = if inner_text.is_empty() {
            None
        } else {
            // We need the full outer context to figure out what comes after expect(await expr)
            // e.g. `.toBe(1)` — keep that suffix by only replacing the expect call itself
            let replacement = format!("await expect({inner_text}).resolves");
            Some(Fix {
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(call.span.start, call.span.end),
                    replacement,
                }],
            })
        };

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-expect-resolves".to_owned(),
            message: "Use `expect(promise).resolves` instead of `expect(await promise)`".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Use `.resolves` matcher instead of awaiting inside `expect()`".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferExpectResolves)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_expect_await() {
        let diags = lint("async function t() { expect(await fetchData()).toBe(1); }");
        assert_eq!(diags.len(), 1, "`expect(await ...)` should be flagged");
    }

    #[test]
    fn test_allows_resolves() {
        let diags = lint("async function t() { await expect(fetchData()).resolves.toBe(1); }");
        assert!(
            diags.is_empty(),
            "`.resolves` pattern should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_await_expect() {
        let diags = lint("expect(getValue()).toBe(1);");
        assert!(
            diags.is_empty(),
            "`expect()` without await argument should not be flagged"
        );
    }
}
