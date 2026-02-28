//! Rule: `jest/no-done-callback`
//!
//! Warn when a `done` callback parameter is used in test/hook callbacks.
//! Prefer async/await patterns instead.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, BindingPattern, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-done-callback";

/// Test and hook function names to check.
const CALLBACK_FUNS: &[&str] = &[
    "it",
    "test",
    "beforeEach",
    "afterEach",
    "beforeAll",
    "afterAll",
];

/// Flags test/hook callbacks that use a `done` parameter.
#[derive(Debug)]
pub struct NoDoneCallback;

impl NativeRule for NoDoneCallback {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `done` callback in tests — use async/await instead".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check callee is a test/hook function
        let callee_name = match &call.callee {
            Expression::Identifier(id) => id.name.as_str(),
            _ => return,
        };

        if !CALLBACK_FUNS.contains(&callee_name) {
            return;
        }

        // For it/test, callback is the second arg; for hooks, it's the first
        let callback_idx = usize::from(callee_name == "it" || callee_name == "test");

        let Some(callback) = call.arguments.get(callback_idx) else {
            return;
        };

        // Check if the callback has a parameter named `done`
        let has_done = match callback {
            Argument::ArrowFunctionExpression(arrow) => {
                arrow.params.items.iter().any(|p| {
                    matches!(&p.pattern, BindingPattern::BindingIdentifier(id) if id.name.as_str() == "done")
                })
            }
            Argument::FunctionExpression(func) => {
                func.params.items.iter().any(|p| {
                    matches!(&p.pattern, BindingPattern::BindingIdentifier(id) if id.name.as_str() == "done")
                })
            }
            _ => false,
        };

        if has_done {
            ctx.report_warning(
                RULE_NAME,
                &format!(
                    "Avoid using a `done` callback in `{callee_name}()` — use async/await instead"
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDoneCallback)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_done_in_test() {
        let diags = lint("test('async', (done) => { done(); });");
        assert_eq!(diags.len(), 1, "done callback in test should be flagged");
    }

    #[test]
    fn test_flags_done_in_before_each() {
        let diags = lint("beforeEach((done) => { done(); });");
        assert_eq!(
            diags.len(),
            1,
            "done callback in beforeEach should be flagged"
        );
    }

    #[test]
    fn test_allows_async_test() {
        let diags = lint("test('async', async () => { await something(); });");
        assert!(
            diags.is_empty(),
            "async test without done should not be flagged"
        );
    }
}
