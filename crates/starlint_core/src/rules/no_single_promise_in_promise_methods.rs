//! Rule: `no-single-promise-in-promise-methods`
//!
//! Disallow passing single-element arrays to `Promise.all()`, `Promise.race()`,
//! `Promise.allSettled()`, and `Promise.any()`. These methods are designed to
//! operate on multiple promises — passing a single-element array is likely a
//! mistake and should be replaced with the promise itself or `Promise.resolve()`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Promise methods that expect multiple promises.
const PROMISE_AGGREGATE_METHODS: &[&str] = &["all", "race", "allSettled", "any"];

/// Flags `Promise.all([x])`, `Promise.race([x])`, etc. with a single element.
#[derive(Debug)]
pub struct NoSinglePromiseInPromiseMethods;

impl NativeRule for NoSinglePromiseInPromiseMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-single-promise-in-promise-methods".to_owned(),
            description: "Disallow passing single-element arrays to Promise aggregate methods"
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

        // Check for `Promise.<method>(...)` pattern via static member access.
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        // Object must be `Promise`.
        if !matches!(&member.object, Expression::Identifier(ident) if ident.name == "Promise") {
            return;
        }

        let method_name = member.property.name.as_str();
        if !PROMISE_AGGREGATE_METHODS.contains(&method_name) {
            return;
        }

        // Must have exactly one argument, and it must be an array expression.
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let Argument::ArrayExpression(array) = first_arg else {
            return;
        };

        // Flag if the array has exactly one element (and no spread).
        if array.elements.len() == 1 {
            ctx.report_warning(
                "no-single-promise-in-promise-methods",
                &format!(
                    "Unnecessary single-element array in `Promise.{method_name}()` — use the value directly"
                ),
                Span::new(call.span.start, call.span.end),
            );
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoSinglePromiseInPromiseMethods)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_promise_all_single() {
        let diags = lint("Promise.all([p1])");
        assert_eq!(
            diags.len(),
            1,
            "Promise.all with single element should be flagged"
        );
    }

    #[test]
    fn test_flags_promise_race_single() {
        let diags = lint("Promise.race([p1])");
        assert_eq!(
            diags.len(),
            1,
            "Promise.race with single element should be flagged"
        );
    }

    #[test]
    fn test_flags_promise_all_settled_single() {
        let diags = lint("Promise.allSettled([p1])");
        assert_eq!(
            diags.len(),
            1,
            "Promise.allSettled with single element should be flagged"
        );
    }

    #[test]
    fn test_flags_promise_any_single() {
        let diags = lint("Promise.any([p1])");
        assert_eq!(
            diags.len(),
            1,
            "Promise.any with single element should be flagged"
        );
    }

    #[test]
    fn test_allows_promise_all_multiple() {
        let diags = lint("Promise.all([p1, p2])");
        assert!(
            diags.is_empty(),
            "Promise.all with multiple elements should not be flagged"
        );
    }

    #[test]
    fn test_allows_promise_resolve() {
        let diags = lint("Promise.resolve(p1)");
        assert!(diags.is_empty(), "Promise.resolve should not be flagged");
    }

    #[test]
    fn test_allows_promise_all_non_array() {
        let diags = lint("Promise.all(promises)");
        assert!(
            diags.is_empty(),
            "Promise.all with non-array argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_promise_all_empty_array() {
        let diags = lint("Promise.all([])");
        assert!(
            diags.is_empty(),
            "Promise.all with empty array should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_promise_callee() {
        let diags = lint("MyLib.all([p1])");
        assert!(diags.is_empty(), "non-Promise callee should not be flagged");
    }
}
