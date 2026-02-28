//! Rule: `promise/no-return-wrap`
//!
//! Forbid wrapping return values in `Promise.resolve()` or `Promise.reject()`
//! inside `.then()` and `.catch()` handlers. These handlers already wrap
//! return values in promises automatically.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Promise.resolve(val)` or `Promise.reject(err)` used inside
/// `.then()` or `.catch()` callback arguments.
///
/// Heuristic: scans the source text of callback arguments for these patterns.
#[derive(Debug)]
pub struct NoReturnWrap;

impl NativeRule for NoReturnWrap {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-return-wrap".to_owned(),
            description: "Forbid wrapping return values in `Promise.resolve`/`Promise.reject`"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method = member.property.name.as_str();
        if method != "then" && method != "catch" {
            return;
        }

        // Check callback arguments for Promise.resolve/reject patterns
        for arg in &call.arguments {
            let arg_expr = match arg {
                oxc_ast::ast::Argument::SpreadElement(_) => continue,
                _ => arg.to_expression(),
            };

            let start = usize::try_from(arg_expr.span().start).unwrap_or(0);
            let end = usize::try_from(arg_expr.span().end).unwrap_or(0);
            let body_text = ctx.source_text().get(start..end).unwrap_or_default();

            if body_text.contains("Promise.resolve(") || body_text.contains("Promise.reject(") {
                ctx.report_error(
                    "promise/no-return-wrap",
                    &format!(
                        "Unnecessary `Promise.resolve`/`Promise.reject` in `.{method}()` — return the value directly"
                    ),
                    Span::new(call.span.start, call.span.end),
                );
                return; // Only report once per call
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoReturnWrap)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_promise_resolve_in_then() {
        let diags = lint("p.then(val => Promise.resolve(val));");
        assert_eq!(diags.len(), 1, "should flag Promise.resolve in .then()");
    }

    #[test]
    fn test_flags_promise_reject_in_catch() {
        let diags = lint("p.catch(err => Promise.reject(err));");
        assert_eq!(diags.len(), 1, "should flag Promise.reject in .catch()");
    }

    #[test]
    fn test_allows_direct_return() {
        let diags = lint("p.then(val => val * 2);");
        assert!(diags.is_empty(), "direct return should not be flagged");
    }
}
