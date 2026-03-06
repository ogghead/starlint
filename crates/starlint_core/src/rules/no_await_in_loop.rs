//! Rule: `no-await-in-loop`
//!
//! Flag `await` expressions inside loops. Using `await` in a loop causes
//! sequential execution — each iteration waits for the previous one to
//! complete. Use `Promise.all()` or similar patterns to run iterations in
//! parallel instead.

use std::sync::RwLock;

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Marker for whether a scope boundary is a function or a loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    /// A function or arrow function boundary (resets loop context).
    Function,
    /// A loop boundary (`for`, `for-in`, `for-of`, `while`, `do-while`).
    Loop,
}

/// Flags `await` expressions that appear inside loop bodies.
#[derive(Debug)]
pub struct NoAwaitInLoop {
    /// Stack of scope boundaries encountered during traversal.
    ///
    /// On entering a function/arrow, `Function` is pushed.
    /// On entering a loop, `Loop` is pushed.
    /// On leaving either, the top is popped.
    scopes: RwLock<Vec<ScopeKind>>,
}

impl NoAwaitInLoop {
    /// Create a new `NoAwaitInLoop` rule instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            scopes: RwLock::new(Vec::new()),
        }
    }
}

impl Default for NoAwaitInLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// Check whether an `AstKind` introduces a loop scope.
const fn is_loop(kind: &AstKind<'_>) -> bool {
    matches!(
        kind,
        AstKind::ForStatement(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
    )
}

/// Check whether an `AstKind` introduces a function boundary.
const fn is_function_boundary(kind: &AstKind<'_>) -> bool {
    matches!(
        kind,
        AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
    )
}

impl NativeRule for NoAwaitInLoop {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-await-in-loop".to_owned(),
            description: "Disallow `await` inside loops — use `Promise.all()` instead".to_owned(),
            category: Category::Performance,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::AwaitExpression,
            AstType::DoWhileStatement,
            AstType::ForInStatement,
            AstType::ForOfStatement,
            AstType::ForStatement,
            AstType::Function,
            AstType::WhileStatement,
        ])
    }

    fn leave_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::AwaitExpression,
            AstType::DoWhileStatement,
            AstType::ForInStatement,
            AstType::ForOfStatement,
            AstType::ForStatement,
            AstType::Function,
            AstType::WhileStatement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Push scope markers for functions and loops.
        if is_function_boundary(kind) {
            if let Ok(mut stack) = self.scopes.write() {
                stack.push(ScopeKind::Function);
            }
            return;
        }

        if is_loop(kind) {
            if let Ok(mut stack) = self.scopes.write() {
                stack.push(ScopeKind::Loop);
            }
            return;
        }

        // Check await expressions.
        let AstKind::AwaitExpression(await_expr) = kind else {
            return;
        };

        let Ok(stack) = self.scopes.read() else {
            return;
        };

        // The most recent scope boundary tells us whether we are directly
        // inside a loop (not separated by a nested function boundary).
        let in_loop = stack.last().is_some_and(|scope| *scope == ScopeKind::Loop);

        if in_loop {
            ctx.report(Diagnostic {
                rule_name: "no-await-in-loop".to_owned(),
                message: "Unexpected `await` inside a loop — iterations run sequentially, consider `Promise.all()`".to_owned(),
                span: Span::new(await_expr.span.start, await_expr.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }

    fn leave(&self, kind: &AstKind<'_>, _ctx: &mut NativeLintContext<'_>) {
        if is_function_boundary(kind) || is_loop(kind) {
            if let Ok(mut stack) = self.scopes.write() {
                let _ = stack.pop();
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAwaitInLoop::new())];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_await_in_for_of_loop() {
        let diags = lint("async function foo() { for (const x of items) { await bar(x); } }");
        assert_eq!(diags.len(), 1, "await in for-of loop should be flagged");
    }

    #[test]
    fn test_allows_await_outside_loop() {
        let diags = lint("async function foo() { await bar(); }");
        assert!(diags.is_empty(), "await outside loop should not be flagged");
    }

    #[test]
    fn test_allows_await_in_nested_async_function_in_loop() {
        let diags = lint(
            "async function foo() { for (const x of items) { items.forEach(async (y) => { await bar(y); }); } }",
        );
        assert!(
            diags.is_empty(),
            "await in nested async arrow inside loop should not be flagged"
        );
    }

    #[test]
    fn test_allows_loop_without_await() {
        let diags = lint("for (const x of items) { use(x); }");
        assert!(diags.is_empty(), "loop without await should not be flagged");
    }

    #[test]
    fn test_flags_await_in_while_loop() {
        let diags = lint("async function foo() { while (true) { await bar(); } }");
        assert_eq!(diags.len(), 1, "await in while loop should be flagged");
    }

    #[test]
    fn test_flags_await_in_for_loop() {
        let diags = lint("async function foo() { for (let i = 0; i < 10; i++) { await bar(i); } }");
        assert_eq!(diags.len(), 1, "await in for loop should be flagged");
    }

    #[test]
    fn test_flags_await_in_do_while_loop() {
        let diags = lint("async function foo() { do { await bar(); } while (true); }");
        assert_eq!(diags.len(), 1, "await in do-while loop should be flagged");
    }
}
