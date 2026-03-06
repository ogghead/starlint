//! Rule: `jest/valid-expect`
//!
//! Error when `expect()` is called without a matcher (e.g., missing `.toBe()`).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/valid-expect";

/// Flags `expect()` calls that are not followed by a matcher method.
#[derive(Debug)]
pub struct ValidExpect;

impl NativeRule for ValidExpect {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require `expect()` calls to have a corresponding matcher".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if this is a direct `expect(...)` call (not `expect(...).toBe(...)`)
        let is_expect = matches!(
            &call.callee,
            Expression::Identifier(id) if id.name.as_str() == "expect"
        );

        if !is_expect {
            return;
        }

        // Check if this expect call is used as a standalone expression statement.
        // If so, it means no matcher was chained. We detect this by checking if the
        // source text after the call's closing paren does NOT start with a `.`.
        let source = ctx.source_text();
        let end = usize::try_from(call.span.end).unwrap_or(0);

        // Look at the character(s) right after the call expression span
        let after_call = source.get(end..).unwrap_or("");
        let next_non_ws = after_call.trim_start().chars().next();

        // If the next meaningful character is not `.`, the expect has no matcher
        if next_non_ws != Some('.') {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`expect()` must be followed by a matcher (e.g., `.toBe()`, `.toEqual()`)"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ValidExpect)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_expect_without_matcher() {
        let diags = lint("expect(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect()` without matcher should be flagged"
        );
    }

    #[test]
    fn test_allows_expect_with_matcher() {
        let diags = lint("expect(true).toBe(true);");
        assert!(
            diags.is_empty(),
            "`expect()` with `.toBe()` should not be flagged"
        );
    }

    #[test]
    fn test_allows_expect_to_equal() {
        let diags = lint("expect(1).toEqual(1);");
        assert!(
            diags.is_empty(),
            "`expect()` with `.toEqual()` should not be flagged"
        );
    }
}
