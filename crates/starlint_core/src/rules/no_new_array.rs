//! Rule: `no-new-array` (unicorn)
//!
//! Disallow `new Array()`. Use array literals `[]` or `Array.from()` instead.
//! `new Array(n)` creates a sparse array which can be confusing.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Array()` calls.
#[derive(Debug)]
pub struct NoNewArray;

impl NativeRule for NoNewArray {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new-array".to_owned(),
            description: "Disallow `new Array()` — use `[]` or `Array.from()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
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

        let is_array = matches!(
            &new_expr.callee,
            Expression::Identifier(id) if id.name.as_str() == "Array"
        );

        if is_array {
            // Remove `new ` prefix: replace whole span with source from callee start
            let source = ctx.source_text();
            let callee_start = usize::try_from(new_expr.callee.span().start).unwrap_or(0);
            let expr_end = usize::try_from(new_expr.span.end).unwrap_or(0);
            let without_new = source.get(callee_start..expr_end).unwrap_or("");

            ctx.report(Diagnostic {
                rule_name: "no-new-array".to_owned(),
                message: "Use `[]` or `Array.from()` instead of `new Array()`".to_owned(),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Warning,
                help: Some("Remove `new` keyword".to_owned()),
                fix: Some(Fix {
                    message: "Remove `new` keyword".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        replacement: without_new.to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNewArray)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_array() {
        let diags = lint("var a = new Array(10);");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_array_literal() {
        let diags = lint("var a = [];");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_array_from() {
        let diags = lint("var a = Array.from({length: 10});");
        assert!(diags.is_empty());
    }
}
