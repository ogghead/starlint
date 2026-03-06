//! Rule: `no-empty-function`
//!
//! Disallow empty function bodies. Empty functions are often indicators
//! of missing implementation or leftover stubs.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags functions with empty bodies.
#[derive(Debug)]
pub struct NoEmptyFunction;

impl NativeRule for NoEmptyFunction {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-empty-function".to_owned(),
            description: "Disallow empty function bodies".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::FunctionBody])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::FunctionBody(body) = kind else {
            return;
        };

        // Empty body: no statements and no directives
        if body.statements.is_empty() && body.directives.is_empty() {
            // Check if there are any comments inside the function body by
            // looking at the raw source. If the body contains a comment,
            // it's intentionally empty (placeholder).
            let source = ctx.source_text();
            let start = usize::try_from(body.span.start).unwrap_or(0);
            let end = usize::try_from(body.span.end).unwrap_or(0);
            let body_text = source.get(start..end).unwrap_or("");
            let has_comment = body_text.contains("//") || body_text.contains("/*");
            let span_start = body.span.start;
            let span_end = body.span.end;

            if !has_comment {
                // Fix: insert a placeholder comment inside the empty body
                let fix = Some(Fix {
                    message: "Add `/* empty */` comment".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(span_start, span_end),
                        replacement: "{ /* empty */ }".to_owned(),
                    }],
                });

                ctx.report(Diagnostic {
                    rule_name: "no-empty-function".to_owned(),
                    message: "Unexpected empty function body".to_owned(),
                    span: Span::new(span_start, span_end),
                    severity: Severity::Warning,
                    help: None,
                    fix,
                    labels: vec![],
                });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEmptyFunction)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_function() {
        let diags = lint("function foo() {}");
        assert_eq!(diags.len(), 1, "empty function should be flagged");
    }

    #[test]
    fn test_flags_empty_arrow() {
        let diags = lint("var f = () => {};");
        assert_eq!(diags.len(), 1, "empty arrow function should be flagged");
    }

    #[test]
    fn test_allows_function_with_body() {
        let diags = lint("function foo() { return 1; }");
        assert!(diags.is_empty(), "function with body should not be flagged");
    }

    #[test]
    fn test_allows_function_with_comment() {
        let diags = lint("function foo() { /* intentionally empty */ }");
        assert!(
            diags.is_empty(),
            "function with comment should not be flagged"
        );
    }
}
