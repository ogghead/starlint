//! Rule: `promise/no-multiple-resolved`
//!
//! Forbid calling `resolve` or `reject` multiple times in a Promise
//! executor. The second call is silently ignored but usually indicates
//! a logic error.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Promise` executors that call `resolve`/`reject` without
/// guarding against multiple invocations.
///
/// This is a heuristic check: it flags when both `resolve` and `reject`
/// are called at the top level of the executor (no early return between them).
#[derive(Debug)]
pub struct NoMultipleResolved;

impl NativeRule for NoMultipleResolved {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-multiple-resolved".to_owned(),
            description: "Forbid calling `resolve`/`reject` multiple times".to_owned(),
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

        let Expression::Identifier(ident) = &new_expr.callee else {
            return;
        };

        if ident.name.as_str() != "Promise" {
            return;
        }

        // Get the executor function (first argument)
        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };

        let arg_expr = match first_arg {
            oxc_ast::ast::Argument::SpreadElement(_) => return,
            _ => first_arg.to_expression(),
        };

        // Check the source text for multiple resolve/reject calls
        // This is a heuristic: count occurrences in the executor body
        let start = usize::try_from(arg_expr.span().start).unwrap_or(0);
        let end = usize::try_from(arg_expr.span().end).unwrap_or(0);
        let body_text = ctx.source_text().get(start..end).unwrap_or_default();

        let resolve_count = body_text.matches("resolve(").count();
        let reject_count = body_text.matches("reject(").count();
        let total = resolve_count.saturating_add(reject_count);

        if total > 1 {
            ctx.report(Diagnostic {
                rule_name: "promise/no-multiple-resolved".to_owned(),
                message: "Promise executor calls `resolve` or `reject` multiple times".to_owned(),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMultipleResolved)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_double_resolve() {
        let diags = lint("new Promise((resolve) => { resolve(1); resolve(2); });");
        assert_eq!(diags.len(), 1, "should flag multiple resolve calls");
    }

    #[test]
    fn test_flags_resolve_and_reject() {
        let diags = lint("new Promise((resolve, reject) => { resolve(1); reject(2); });");
        assert_eq!(diags.len(), 1, "should flag resolve + reject calls");
    }

    #[test]
    fn test_allows_single_resolve() {
        let diags = lint("new Promise((resolve) => { resolve(1); });");
        assert!(diags.is_empty(), "single resolve should be allowed");
    }
}
