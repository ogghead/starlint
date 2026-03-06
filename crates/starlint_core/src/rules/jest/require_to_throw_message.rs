//! Rule: `jest/require-to-throw-message`
//!
//! Warn when `.toThrow()` or `.toThrowError()` is called without an argument.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/require-to-throw-message";

/// Matcher names that should have an argument.
const THROW_MATCHERS: &[&str] = &["toThrow", "toThrowError"];

/// Flags `.toThrow()` and `.toThrowError()` calls with no arguments.
#[derive(Debug)]
pub struct RequireToThrowMessage;

impl NativeRule for RequireToThrowMessage {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require `.toThrow()` to have a message argument".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Match `.toThrow()` or `.toThrowError()` pattern
        let matcher_name = match &call.callee {
            Expression::StaticMemberExpression(member) => member.property.name.as_str(),
            _ => return,
        };

        if !THROW_MATCHERS.contains(&matcher_name) {
            return;
        }

        // Verify it's an expect chain
        let is_expect_chain = match &call.callee {
            Expression::StaticMemberExpression(member) => is_expect_call_or_chain(&member.object),
            _ => false,
        };

        if !is_expect_chain {
            return;
        }

        // Flag if no arguments provided
        if call.arguments.is_empty() {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "`.{matcher_name}()` should include a message argument to ensure the correct error is thrown"
                ),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is an `expect(...)` call or chained from one.
fn is_expect_call_or_chain(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::CallExpression(call) => {
            matches!(
                &call.callee,
                Expression::Identifier(id) if id.name.as_str() == "expect"
            )
        }
        Expression::StaticMemberExpression(member) => is_expect_call_or_chain(&member.object),
        _ => false,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireToThrowMessage)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_to_throw_without_message() {
        let diags = lint("expect(() => { throw new Error('x'); }).toThrow();");
        assert_eq!(
            diags.len(),
            1,
            "`.toThrow()` without argument should be flagged"
        );
    }

    #[test]
    fn test_flags_to_throw_error_without_message() {
        let diags = lint("expect(fn).toThrowError();");
        assert_eq!(
            diags.len(),
            1,
            "`.toThrowError()` without argument should be flagged"
        );
    }

    #[test]
    fn test_allows_to_throw_with_message() {
        let diags = lint("expect(fn).toThrow('expected error');");
        assert!(
            diags.is_empty(),
            "`.toThrow()` with message should not be flagged"
        );
    }
}
