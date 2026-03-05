//! Rule: `jest/prefer-to-have-been-called-times`
//!
//! Suggest `toHaveBeenCalledTimes(n)` over `expect(mock.mock.calls.length).toBe(n)`.
//! The dedicated matcher provides clearer failure messages showing the actual
//! call count.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `expect(mock.mock.calls.length).toBe(n)` patterns.
#[derive(Debug)]
pub struct PreferToHaveBeenCalledTimes;

impl NativeRule for PreferToHaveBeenCalledTimes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-to-have-been-called-times".to_owned(),
            description: "Suggest using `toHaveBeenCalledTimes()` instead of asserting on `.mock.calls.length`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Must be `.toBe(n)` or `.toEqual(n)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        let method = member.property.name.as_str();
        if method != "toBe" && method != "toEqual" {
            return;
        }

        // Object must be `expect(...)` call
        let Expression::CallExpression(expect_call) = &member.object else {
            return;
        };
        let is_expect = matches!(
            &expect_call.callee,
            Expression::Identifier(id) if id.name.as_str() == "expect"
        );
        if !is_expect {
            return;
        }

        // The argument to expect() should end in `.calls.length` or `.length`
        // and contain `mock` somewhere in the chain.
        let Some(expect_arg) = expect_call.arguments.first() else {
            return;
        };
        let Some(expect_arg_expr) = expect_arg.as_expression() else {
            return;
        };

        if is_mock_calls_length(expect_arg_expr) {
            ctx.report(Diagnostic {
                rule_name: "jest/prefer-to-have-been-called-times".to_owned(),
                message:
                    "Use `toHaveBeenCalledTimes()` instead of asserting on `.mock.calls.length`"
                        .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression matches `x.mock.calls.length` or `x.calls.length`
/// patterns commonly used to check mock call counts.
fn is_mock_calls_length(expr: &Expression<'_>) -> bool {
    // Must end in `.length`
    let Expression::StaticMemberExpression(length_member) = expr else {
        return false;
    };
    if length_member.property.name.as_str() != "length" {
        return false;
    }

    // Next level should be `.calls`
    let Expression::StaticMemberExpression(calls_member) = &length_member.object else {
        return false;
    };
    if calls_member.property.name.as_str() != "calls" {
        return false;
    }

    // Optionally `.mock` but at minimum there should be an object
    match &calls_member.object {
        Expression::StaticMemberExpression(mock_member) => {
            mock_member.property.name.as_str() == "mock"
        }
        // Also match `mockFn.calls.length` directly
        Expression::Identifier(_) => true,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferToHaveBeenCalledTimes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_mock_calls_length() {
        let diags = lint("expect(mockFn.mock.calls.length).toBe(2);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(mockFn.mock.calls.length).toBe(2)` should be flagged"
        );
    }

    #[test]
    fn test_flags_calls_length_directly() {
        let diags = lint("expect(spy.calls.length).toBe(1);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(spy.calls.length).toBe(1)` should be flagged"
        );
    }

    #[test]
    fn test_allows_to_have_been_called_times() {
        let diags = lint("expect(mockFn).toHaveBeenCalledTimes(2);");
        assert!(
            diags.is_empty(),
            "`toHaveBeenCalledTimes()` should not be flagged"
        );
    }
}
