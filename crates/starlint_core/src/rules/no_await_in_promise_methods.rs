//! Rule: `no-await-in-promise-methods`
//!
//! Disallow `await` inside `Promise.all()`, `Promise.race()`,
//! `Promise.allSettled()`, and `Promise.any()` array arguments.
//!
//! When you `await` inside the array passed to these methods, the promises
//! are resolved sequentially instead of in parallel, defeating the purpose
//! of using `Promise.all` and friends.

use oxc_ast::AstKind;
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Promise methods that accept an iterable of promises for parallel resolution.
const PROMISE_METHODS: &[&str] = &["all", "race", "allSettled", "any"];

/// Flags `await` expressions inside array arguments to `Promise.all()`,
/// `Promise.race()`, `Promise.allSettled()`, and `Promise.any()`.
#[derive(Debug)]
pub struct NoAwaitInPromiseMethods;

impl NativeRule for NoAwaitInPromiseMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-await-in-promise-methods".to_owned(),
            description: "Disallow `await` in Promise.all/race/allSettled/any array arguments"
                .to_owned(),
            category: Category::Correctness,
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

        // Check if callee is `Promise.<method>`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let Expression::Identifier(obj) = &member.object else {
            return;
        };

        if obj.name.as_str() != "Promise" {
            return;
        }

        let method_name = member.property.name.as_str();
        if !PROMISE_METHODS.contains(&method_name) {
            return;
        }

        // Check the first argument — should be an array expression
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let Some(first_expr) = first_arg.as_expression() else {
            return;
        };

        let Expression::ArrayExpression(array) = first_expr else {
            return;
        };

        // Check if any element in the array is an `await` expression
        // Collect fix edits: for each `await expr`, remove the `await ` prefix
        let source = ctx.source_text();
        let mut has_await = false;
        let mut edits: Vec<Edit> = Vec::new();
        for element in &array.elements {
            if let ArrayExpressionElement::AwaitExpression(await_expr) = element {
                has_await = true;
                // Remove the `await ` keyword — replace await_expr span with just the argument
                let arg_span = await_expr.argument.span();
                edits.push(Edit {
                    span: Span::new(await_expr.span.start, arg_span.start),
                    replacement: String::new(),
                });
            }
        }
        // Use `source` to prevent "unused variable" warnings
        let _ = source;

        if has_await {
            let fix = (!edits.is_empty()).then(|| Fix {
                message: "Remove `await` from array elements".to_owned(),
                edits,
            });

            ctx.report(Diagnostic {
                rule_name: "no-await-in-promise-methods".to_owned(),
                message: format!(
                    "Avoid using `await` inside `Promise.{method_name}()` — it defeats parallel execution"
                ),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAwaitInPromiseMethods)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_await_in_promise_all() {
        let diags = lint("async function f() { await Promise.all([await p1, await p2]); }");
        assert_eq!(
            diags.len(),
            1,
            "await inside Promise.all array should be flagged"
        );
    }

    #[test]
    fn test_allows_promise_all_without_await() {
        let diags = lint("async function f() { await Promise.all([p1, p2]); }");
        assert!(
            diags.is_empty(),
            "Promise.all without inner await should not be flagged"
        );
    }

    #[test]
    fn test_flags_await_in_promise_race() {
        let diags = lint("async function f() { await Promise.race([await p1]); }");
        assert_eq!(
            diags.len(),
            1,
            "await inside Promise.race array should be flagged"
        );
    }

    #[test]
    fn test_flags_await_in_promise_all_settled() {
        let diags = lint("async function f() { await Promise.allSettled([await p1]); }");
        assert_eq!(
            diags.len(),
            1,
            "await inside Promise.allSettled array should be flagged"
        );
    }

    #[test]
    fn test_flags_await_in_promise_any() {
        let diags = lint("async function f() { await Promise.any([await p1]); }");
        assert_eq!(
            diags.len(),
            1,
            "await inside Promise.any array should be flagged"
        );
    }

    #[test]
    fn test_allows_standalone_await() {
        let diags = lint("async function f() { await p1; }");
        assert!(diags.is_empty(), "standalone await should not be flagged");
    }

    #[test]
    fn test_allows_non_promise_call() {
        let diags = lint("async function f() { Foo.all([await p1]); }");
        assert!(diags.is_empty(), "non-Promise call should not be flagged");
    }
}
