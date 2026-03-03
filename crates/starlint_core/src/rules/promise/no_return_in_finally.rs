//! Rule: `promise/no-return-in-finally`
//!
//! Forbid `return` statements in `.finally()` callbacks. Returning from
//! `.finally()` silently swallows the resolved/rejected value.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.finally()` callbacks that contain `return` statements.
///
/// Heuristic: scans the source text of `.finally()` callback arguments
/// for `return` keywords.
#[derive(Debug)]
pub struct NoReturnInFinally;

impl NativeRule for NoReturnInFinally {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-return-in-finally".to_owned(),
            description: "Forbid `return` in `.finally()` callbacks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "finally" {
            return;
        }

        // Check the first argument (the finally callback)
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let arg_expr = match first_arg {
            oxc_ast::ast::Argument::SpreadElement(_) => return,
            _ => first_arg.to_expression(),
        };

        // Expression arrows don't have explicit return, skip
        if let Expression::ArrowFunctionExpression(arrow) = arg_expr {
            if arrow.expression {
                return;
            }
        }

        let start = usize::try_from(arg_expr.span().start).unwrap_or(0);
        let end = usize::try_from(arg_expr.span().end).unwrap_or(0);
        let body_text = ctx.source_text().get(start..end).unwrap_or_default();

        // Heuristic: check for return statement in the body
        // Skip `return;` (empty return) which is less harmful
        if body_text.contains("return ") {
            ctx.report_error(
                "promise/no-return-in-finally",
                "Do not use `return` with a value in `.finally()` — it silently swallows the promise result",
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoReturnInFinally)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_return_in_finally() {
        let diags = lint("p.finally(() => { return 42; });");
        assert_eq!(diags.len(), 1, "should flag return in .finally()");
    }

    #[test]
    fn test_allows_finally_without_return() {
        let diags = lint("p.finally(() => { cleanup(); });");
        assert!(
            diags.is_empty(),
            ".finally() without return should be allowed"
        );
    }

    #[test]
    fn test_allows_expression_arrow_finally() {
        let diags = lint("p.finally(() => cleanup());");
        assert!(
            diags.is_empty(),
            "expression arrow in .finally() should be allowed"
        );
    }
}
