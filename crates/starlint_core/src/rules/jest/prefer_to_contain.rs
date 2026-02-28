//! Rule: `jest/prefer-to-contain`
//!
//! Suggest `expect(arr).toContain(x)` over `expect(arr.includes(x)).toBe(true)`.
//! The `toContain` matcher provides a clearer failure message showing the
//! array contents and the missing element.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `expect(arr.includes(x)).toBe(true)` patterns.
#[derive(Debug)]
pub struct PreferToContain;

impl NativeRule for PreferToContain {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-to-contain".to_owned(),
            description:
                "Suggest using `toContain()` instead of `expect(arr.includes(x)).toBe(true)`"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Must be `.toBe(true)` or `.toBe(false)` or `.toEqual(true)` / `.toEqual(false)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        let method = member.property.name.as_str();
        if method != "toBe" && method != "toEqual" {
            return;
        }

        // Check the argument is a boolean literal
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

        // The argument to `expect()` must be `something.includes(x)`
        let Some(expect_arg) = expect_call.arguments.first() else {
            return;
        };
        let Some(expect_arg_expr) = expect_arg.as_expression() else {
            return;
        };
        let Expression::CallExpression(includes_call) = expect_arg_expr else {
            return;
        };
        let Expression::StaticMemberExpression(includes_member) = &includes_call.callee else {
            return;
        };
        if includes_member.property.name.as_str() != "includes" {
            return;
        }

        ctx.report_warning(
            "jest/prefer-to-contain",
            "Use `toContain()` instead of `expect(arr.includes(x)).toBe(true/false)`",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferToContain)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_includes_to_be_true() {
        let diags = lint("expect(arr.includes(1)).toBe(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(arr.includes(1)).toBe(true)` should be flagged"
        );
    }

    #[test]
    fn test_flags_includes_to_be_false() {
        let diags = lint("expect(arr.includes(1)).toBe(false);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(arr.includes(1)).toBe(false)` should be flagged"
        );
    }

    #[test]
    fn test_allows_to_contain() {
        let diags = lint("expect(arr).toContain(1);");
        assert!(diags.is_empty(), "`toContain()` should not be flagged");
    }
}
