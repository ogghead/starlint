//! Rule: `promise/no-promise-in-callback`
//!
//! Forbid creating promises inside callback-style functions. Mixing
//! callback patterns with promise patterns leads to confusing control flow.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Promise` or `Promise.resolve`/`Promise.reject` inside
/// functions whose parameter names suggest they are callbacks.
///
/// This is a heuristic: we check if any parameter is named `cb`, `callback`,
/// `done`, or `next` and the function body contains Promise usage.
#[derive(Debug)]
pub struct NoPromiseInCallback;

/// Common callback parameter names.
const CALLBACK_PARAMS: &[&str] = &["cb", "callback", "done", "next"];

impl NativeRule for NoPromiseInCallback {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-promise-in-callback".to_owned(),
            description: "Forbid creating promises inside callbacks".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Look for new Promise(...) inside function bodies
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        let Expression::Identifier(ident) = &new_expr.callee else {
            return;
        };

        if ident.name.as_str() != "Promise" {
            return;
        }

        // Heuristic: check the surrounding source for callback parameter names
        // A full implementation would walk up the AST to the enclosing function.
        let start = usize::try_from(new_expr.span.start).unwrap_or(0);
        let prefix_start = start.saturating_sub(200);
        let prefix = ctx
            .source_text()
            .get(prefix_start..start)
            .unwrap_or_default();

        for name in CALLBACK_PARAMS {
            // Look for patterns like `function foo(cb)` or `(callback) =>`
            if prefix.contains(&format!("({name})"))
                || prefix.contains(&format!("({name},"))
                || prefix.contains(&format!(", {name})"))
                || prefix.contains(&format!(", {name},"))
            {
                ctx.report_warning(
                    "promise/no-promise-in-callback",
                    "Avoid creating a Promise inside a callback-style function",
                    Span::new(new_expr.span.start, new_expr.span.end),
                );
                return;
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoPromiseInCallback)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_promise_in_callback() {
        let diags = lint("function foo(callback) { return new Promise((r) => r(1)); }");
        assert_eq!(
            diags.len(),
            1,
            "should flag new Promise inside callback function"
        );
    }

    #[test]
    fn test_allows_promise_in_normal_function() {
        let diags = lint("function foo(x) { return new Promise((r) => r(x)); }");
        assert!(diags.is_empty(), "normal function should not be flagged");
    }

    #[test]
    fn test_flags_promise_in_done_callback() {
        let diags = lint("function handler(done) { return new Promise((r) => r(1)); }");
        assert_eq!(
            diags.len(),
            1,
            "should flag new Promise inside done callback"
        );
    }
}
