//! Rule: `vitest/prefer-to-be-object`
//!
//! Suggest `toBeTypeOf('object')` over `typeof` assertions for object checks.
//! Using the Vitest-native `toBeTypeOf` matcher is more readable and provides
//! better error messages than manually comparing `typeof` results.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-to-be-object";

/// Suggest `toBeTypeOf('object')` over manual `typeof` checks.
#[derive(Debug)]
pub struct PreferToBeObject;

impl NativeRule for PreferToBeObject {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `toBeTypeOf('object')` over `typeof` assertions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Match `expect(typeof x).toBe("object")` pattern.
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "toBe" {
            return;
        }

        if call.arguments.len() != 1 {
            return;
        }

        // Check if the argument is the string "object".
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let is_object_string = match first_arg {
            Argument::StringLiteral(lit) => lit.value.as_str() == "object",
            _ => false,
        };

        if !is_object_string {
            return;
        }

        // Check if the `expect()` call wraps a `typeof` expression.
        // The member object should be a CallExpression (the `expect(...)` call).
        let Expression::CallExpression(expect_call) = &member.object else {
            return;
        };

        let is_expect = matches!(&expect_call.callee, Expression::Identifier(id) if id.name.as_str() == "expect");

        if !is_expect {
            return;
        }

        // Check if the expect argument is a `typeof` unary expression.
        if let Some(arg) = expect_call.arguments.first() {
            if matches!(arg, Argument::UnaryExpression(unary) if unary.operator == oxc_ast::ast::UnaryOperator::Typeof)
            {
                ctx.report_warning(
                    RULE_NAME,
                    "Prefer `toBeTypeOf('object')` over `expect(typeof x).toBe('object')`",
                    Span::new(call.span.start, call.span.end),
                );
            }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferToBeObject)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_typeof_object_assertion() {
        let source = r#"expect(typeof value).toBe("object");"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`expect(typeof x).toBe('object')` should be flagged"
        );
    }

    #[test]
    fn test_allows_to_be_type_of() {
        let source = r#"expect(value).toBeTypeOf("object");"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`toBeTypeOf('object')` should not be flagged"
        );
    }

    #[test]
    fn test_allows_to_be_string() {
        let source = r#"expect(typeof value).toBe("string");"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`toBe('string')` check should not be flagged by this rule"
        );
    }
}
