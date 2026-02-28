//! Rule: `no-new-buffer` (unicorn)
//!
//! Disallow `new Buffer()`. The `Buffer` constructor is deprecated — use
//! `Buffer.from()`, `Buffer.alloc()`, or `Buffer.allocUnsafe()` instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Buffer()` calls.
#[derive(Debug)]
pub struct NoNewBuffer;

impl NativeRule for NoNewBuffer {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new-buffer".to_owned(),
            description: "Disallow `new Buffer()` (deprecated)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        let is_buffer = matches!(
            &new_expr.callee,
            Expression::Identifier(id) if id.name.as_str() == "Buffer"
        );

        if is_buffer {
            ctx.report_error(
                "no-new-buffer",
                "`new Buffer()` is deprecated — use `Buffer.from()`, `Buffer.alloc()`, or `Buffer.allocUnsafe()`",
                Span::new(new_expr.span.start, new_expr.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNewBuffer)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_buffer() {
        let diags = lint("var b = new Buffer(10);");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_buffer_from() {
        let diags = lint("var b = Buffer.from([1, 2]);");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_buffer_alloc() {
        let diags = lint("var b = Buffer.alloc(10);");
        assert!(diags.is_empty());
    }
}
