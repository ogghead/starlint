//! Rule: `no-new-buffer` (unicorn)
//!
//! Disallow `new Buffer()`. The `Buffer` constructor is deprecated — use
//! `Buffer.from()`, `Buffer.alloc()`, or `Buffer.allocUnsafe()` instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
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

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
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
            let source = ctx.source_text();
            // Extract arguments source
            let callee_start = usize::try_from(new_expr.callee.span().start).unwrap_or(0);
            let expr_end = usize::try_from(new_expr.span.end).unwrap_or(0);
            // Get "Buffer(...)" from callee start to end
            let callee_to_end = source.get(callee_start..expr_end).unwrap_or("");
            // Determine method: alloc for numeric arg, from otherwise
            let method = new_expr.arguments.first().map_or("from", |arg| {
                if matches!(arg, oxc_ast::ast::Argument::NumericLiteral(_)) {
                    "alloc"
                } else {
                    "from"
                }
            });
            // Replace "Buffer" in callee_to_end with "Buffer.method"
            let replacement = callee_to_end.replacen("Buffer", &format!("Buffer.{method}"), 1);

            ctx.report(Diagnostic {
                rule_name: "no-new-buffer".to_owned(),
                message: "`new Buffer()` is deprecated — use `Buffer.from()`, `Buffer.alloc()`, or `Buffer.allocUnsafe()`".to_owned(),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Error,
                help: Some(format!("Replace with `Buffer.{method}()`")),
                fix: Some(Fix {
                    message: format!("Replace with `Buffer.{method}()`"),
                    edits: vec![Edit {
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        replacement,
                    }],
                }),
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
