//! Rule: `no-multi-str`
//!
//! Disallow multiline strings created with `\` at the end of a line.
//! Use template literals or string concatenation instead.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags multiline strings using backslash continuation.
#[derive(Debug)]
pub struct NoMultiStr;

impl NativeRule for NoMultiStr {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-multi-str".to_owned(),
            description: "Disallow multiline strings".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::StringLiteral])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StringLiteral(lit) = kind else {
            return;
        };

        // Check the raw source text of the string for backslash-newline continuation
        let source = ctx.source_text();
        let start = usize::try_from(lit.span.start).unwrap_or(0);
        let end = usize::try_from(lit.span.end).unwrap_or(0);
        let raw_text = source.get(start..end).unwrap_or("");

        // A multiline string has a backslash immediately before a newline
        let has_continuation = raw_text.contains("\\\n") || raw_text.contains("\\\r\n");
        let span_start = lit.span.start;
        let span_end = lit.span.end;

        if has_continuation {
            // Fix: convert to template literal — replace quotes with backticks
            // and remove backslash-newline continuations
            let fix = {
                let mut converted = raw_text.to_owned();
                // Remove backslash-newline continuations
                converted = converted.replace("\\\r\n", "\n");
                converted = converted.replace("\\\n", "\n");
                // Replace outer quotes with backticks
                if converted.starts_with('\'') || converted.starts_with('"') {
                    converted.replace_range(..1, "`");
                }
                if converted.ends_with('\'') || converted.ends_with('"') {
                    let last = converted.len().saturating_sub(1);
                    converted.replace_range(last.., "`");
                }
                Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Convert to template literal".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(span_start, span_end),
                        replacement: converted,
                    }],
                    is_snippet: false,
                })
            };

            ctx.report(Diagnostic {
                rule_name: "no-multi-str".to_owned(),
                message: "Multiline strings using `\\` are not recommended".to_owned(),
                span: Span::new(span_start, span_end),
                severity: Severity::Warning,
                help: None,
                fix,
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMultiStr)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_multiline_string() {
        let diags = lint("var x = 'hello \\\nworld';");
        assert_eq!(
            diags.len(),
            1,
            "multiline string with backslash continuation should be flagged"
        );
    }

    #[test]
    fn test_allows_single_line_string() {
        let diags = lint("var x = 'hello world';");
        assert!(diags.is_empty(), "single line string should not be flagged");
    }

    #[test]
    fn test_allows_template_literal() {
        let diags = lint("var x = `hello\nworld`;");
        assert!(diags.is_empty(), "template literal should not be flagged");
    }
}
