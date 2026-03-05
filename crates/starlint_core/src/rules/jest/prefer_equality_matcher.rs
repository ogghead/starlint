//! Rule: `jest/prefer-equality-matcher`
//!
//! Suggest `toBe(x)` / `toEqual(x)` over `expect(a === b).toBe(true)`.
//! The dedicated equality matchers produce clearer failure messages with
//! expected vs received values.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `expect(a === b).toBe(true)` patterns.
#[derive(Debug)]
pub struct PreferEqualityMatcher;

impl NativeRule for PreferEqualityMatcher {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-equality-matcher".to_owned(),
            description: "Suggest using equality matchers instead of `expect(a === b).toBe(true)`"
                .to_owned(),
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

        // Must be `.toBe(true)` or `.toBe(false)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        let method = member.property.name.as_str();
        if method != "toBe" && method != "toEqual" {
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

        // First arg of expect() must be `a === b` or `a == b` or `a !== b` or `a != b`
        let Some(expect_arg) = expect_call.arguments.first() else {
            return;
        };
        let Some(expect_arg_expr) = expect_arg.as_expression() else {
            return;
        };
        let Expression::BinaryExpression(binary) = expect_arg_expr else {
            return;
        };

        let is_equality_op = matches!(
            binary.operator,
            BinaryOperator::StrictEquality
                | BinaryOperator::StrictInequality
                | BinaryOperator::Equality
                | BinaryOperator::Inequality
        );
        if !is_equality_op {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-equality-matcher".to_owned(),
            message: "Use `toBe()` or `toEqual()` directly instead of `expect(a === b).toBe(true)`"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferEqualityMatcher)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_strict_equality() {
        let diags = lint("expect(a === b).toBe(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(a === b).toBe(true)` should be flagged"
        );
    }

    #[test]
    fn test_flags_inequality() {
        let diags = lint("expect(a !== b).toBe(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(a !== b).toBe(true)` should be flagged"
        );
    }

    #[test]
    fn test_allows_direct_to_be() {
        let diags = lint("expect(a).toBe(b);");
        assert!(
            diags.is_empty(),
            "`expect(a).toBe(b)` should not be flagged"
        );
    }
}
