//! Rule: `promise/no-callback-in-promise`
//!
//! Forbid calling callbacks (e.g. `cb`, `callback`, `done`, `next`)
//! inside `.then()` or `.catch()`. Mixing callbacks with promises leads
//! to confusing control flow and potential double-resolution bugs.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Common callback parameter names.
const CALLBACK_NAMES: &[&str] = &["cb", "callback", "done", "next"];

/// Flags callback invocations inside `.then()`/`.catch()` handlers.
#[derive(Debug)]
pub struct NoCallbackInPromise;

impl NativeRule for NoCallbackInPromise {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-callback-in-promise".to_owned(),
            description: "Forbid callbacks inside `.then()`/`.catch()`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if this is a .then() or .catch() call
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method = member.property.name.as_str();
        if method != "then" && method != "catch" {
            return;
        }

        // Check each argument for callback-named identifiers being passed
        for arg in &call.arguments {
            let arg_expr = match arg {
                oxc_ast::ast::Argument::SpreadElement(_) => continue,
                _ => arg.to_expression(),
            };

            if let Expression::Identifier(ident) = arg_expr {
                if CALLBACK_NAMES.contains(&ident.name.as_str()) {
                    ctx.report_error(
                        "promise/no-callback-in-promise",
                        &format!(
                            "Do not pass callback `{}` into `.{method}()` — avoid mixing callbacks and promises",
                            ident.name
                        ),
                        Span::new(call.span.start, call.span.end),
                    );
                }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoCallbackInPromise)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_callback_in_then() {
        let diags = lint("promise.then(cb);");
        assert_eq!(diags.len(), 1, "should flag callback passed to .then()");
    }

    #[test]
    fn test_flags_done_in_catch() {
        let diags = lint("promise.catch(done);");
        assert_eq!(diags.len(), 1, "should flag done passed to .catch()");
    }

    #[test]
    fn test_allows_normal_then() {
        let diags = lint("promise.then(val => val * 2);");
        assert!(diags.is_empty(), "normal .then() should not be flagged");
    }
}
