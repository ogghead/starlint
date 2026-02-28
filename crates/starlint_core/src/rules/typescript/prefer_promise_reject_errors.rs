//! Rule: `typescript/prefer-promise-reject-errors`
//!
//! Prefer using `Error` objects in `Promise.reject()`. Rejecting with non-Error
//! values (string literals, numbers, booleans, `null`, `undefined`) makes
//! debugging harder because stack traces are lost.
//!
//! Simplified syntax-only version — full checking requires type information.
//!
//! Flagged patterns:
//! - `Promise.reject("message")`
//! - `Promise.reject(42)`
//! - `Promise.reject(true)`
//! - `Promise.reject(null)`
//! - `Promise.reject(undefined)`

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/prefer-promise-reject-errors";

/// Flags `Promise.reject()` calls where the argument is a non-Error value.
#[derive(Debug)]
pub struct PreferPromiseRejectErrors;

impl NativeRule for PreferPromiseRejectErrors {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer using `Error` objects in `Promise.reject()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if callee is `Promise.reject`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "reject" {
            return;
        }

        let Expression::Identifier(obj_id) = &member.object else {
            return;
        };

        if obj_id.name.as_str() != "Promise" {
            return;
        }

        // Check the first argument — flag if it is a non-Error literal value
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        if is_non_error_argument(first_arg) {
            ctx.report_warning(
                RULE_NAME,
                "Expected an `Error` object in `Promise.reject()` — do not reject with a literal value",
                Span::new(call.span.start, call.span.end),
            );
        }
    }
}

/// Returns `true` if the argument is a literal value that is not an Error:
/// string, number, boolean, null, undefined, bigint, or template literal.
fn is_non_error_argument(arg: &Argument<'_>) -> bool {
    matches!(
        arg,
        Argument::StringLiteral(_)
            | Argument::NumericLiteral(_)
            | Argument::BooleanLiteral(_)
            | Argument::NullLiteral(_)
            | Argument::BigIntLiteral(_)
            | Argument::TemplateLiteral(_)
    ) || is_undefined_argument(arg)
}

/// Returns `true` if the argument is the identifier `undefined`.
fn is_undefined_argument(arg: &Argument<'_>) -> bool {
    let Argument::Identifier(ident) = arg else {
        return false;
    };
    ident.name == "undefined"
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferPromiseRejectErrors)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_reject_with_string() {
        let diags = lint("Promise.reject(\"error message\");");
        assert_eq!(
            diags.len(),
            1,
            "Promise.reject with string should be flagged"
        );
    }

    #[test]
    fn test_flags_reject_with_number() {
        let diags = lint("Promise.reject(42);");
        assert_eq!(
            diags.len(),
            1,
            "Promise.reject with number should be flagged"
        );
    }

    #[test]
    fn test_flags_reject_with_undefined() {
        let diags = lint("Promise.reject(undefined);");
        assert_eq!(
            diags.len(),
            1,
            "Promise.reject with undefined should be flagged"
        );
    }

    #[test]
    fn test_allows_reject_with_new_error() {
        let diags = lint("Promise.reject(new Error('something failed'));");
        assert!(
            diags.is_empty(),
            "Promise.reject with new Error should not be flagged"
        );
    }

    #[test]
    fn test_allows_reject_with_variable() {
        let diags = lint("Promise.reject(err);");
        assert!(
            diags.is_empty(),
            "Promise.reject with a variable should not be flagged"
        );
    }
}
