//! Rule: `require-yield`
//!
//! Require generator functions to contain at least one `yield` expression.
//! A generator function with no `yield` is likely a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags generator functions that contain no `yield` expressions.
#[derive(Debug)]
pub struct RequireYield;

impl NativeRule for RequireYield {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-yield".to_owned(),
            description: "Require generator functions to contain yield".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Function])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Function(func) = kind else {
            return;
        };

        // Only check generator functions
        if !func.generator {
            return;
        }

        // Check if the function body contains a yield expression
        let Some(body) = &func.body else {
            return;
        };

        // Walk the statements looking for yield expressions
        let has_yield = source_contains_yield(ctx.source_text(), body.span.start, body.span.end);

        if !has_yield {
            let name = func
                .id
                .as_ref()
                .map_or("(anonymous)", |id| id.name.as_str());
            ctx.report_error(
                "require-yield",
                &format!("Generator function '{name}' requires a yield expression"),
                Span::new(func.span.start, func.span.end),
            );
        }
    }
}

/// Quick check: does the source text in the given span contain the `yield` keyword?
/// This is a simple heuristic — it may false-positive on `yield` in strings/comments,
/// but for generator functions this is almost always correct.
fn source_contains_yield(source: &str, start: u32, end: u32) -> bool {
    let start_idx = usize::try_from(start).unwrap_or(usize::MAX);
    let end_idx = usize::try_from(end).unwrap_or(0).min(source.len());
    source
        .get(start_idx..end_idx)
        .is_some_and(|s| s.contains("yield"))
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireYield)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_generator() {
        let diags = lint("function* foo() {}");
        assert_eq!(diags.len(), 1, "empty generator should be flagged");
    }

    #[test]
    fn test_allows_generator_with_yield() {
        let diags = lint("function* foo() { yield 1; }");
        assert!(
            diags.is_empty(),
            "generator with yield should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_function() {
        let diags = lint("function foo() {}");
        assert!(diags.is_empty(), "regular function should not be flagged");
    }

    #[test]
    fn test_flags_generator_with_only_return() {
        let diags = lint("function* foo() { return 1; }");
        assert_eq!(
            diags.len(),
            1,
            "generator with only return should be flagged"
        );
    }

    #[test]
    fn test_allows_generator_with_yield_star() {
        let diags = lint("function* foo() { yield* bar(); }");
        assert!(
            diags.is_empty(),
            "generator with yield* should not be flagged"
        );
    }
}
