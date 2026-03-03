//! Rule: `jest/prefer-to-have-been-called`
//!
//! Suggest `toHaveBeenCalled()` over `toBe(true)` on mock `.called` property.
//! Using the dedicated matcher provides more descriptive failure messages.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `expect(mock.called).toBe(true)` patterns.
#[derive(Debug)]
pub struct PreferToHaveBeenCalled;

impl NativeRule for PreferToHaveBeenCalled {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-to-have-been-called".to_owned(),
            description: "Suggest using `toHaveBeenCalled()` over `toBe(true)` on `.called`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Must be `.toBe(true)` or `.toBe(false)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        if member.property.name.as_str() != "toBe" {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };
        let Some(arg_expr) = first_arg.as_expression() else {
            return;
        };
        let is_bool = matches!(arg_expr, Expression::BooleanLiteral(_));
        if !is_bool {
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

        // First arg of expect() must be `something.called`
        let Some(expect_arg) = expect_call.arguments.first() else {
            return;
        };
        let Some(expect_arg_expr) = expect_arg.as_expression() else {
            return;
        };
        let Expression::StaticMemberExpression(arg_member) = expect_arg_expr else {
            return;
        };
        if arg_member.property.name.as_str() != "called" {
            return;
        }

        ctx.report_warning(
            "jest/prefer-to-have-been-called",
            "Use `toHaveBeenCalled()` instead of asserting on `.called` with `toBe()`",
            Span::new(call.span.start, call.span.end),
        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferToHaveBeenCalled)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_called_to_be_true() {
        let diags = lint("expect(mockFn.called).toBe(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(mockFn.called).toBe(true)` should be flagged"
        );
    }

    #[test]
    fn test_flags_called_to_be_false() {
        let diags = lint("expect(mockFn.called).toBe(false);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(mockFn.called).toBe(false)` should be flagged"
        );
    }

    #[test]
    fn test_allows_to_have_been_called() {
        let diags = lint("expect(mockFn).toHaveBeenCalled();");
        assert!(
            diags.is_empty(),
            "`toHaveBeenCalled()` should not be flagged"
        );
    }
}
