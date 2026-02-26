//! Rule: `no-unexpected-multiline`
//!
//! Disallow confusing multiline expressions where a newline looks like it is
//! ending a statement, but is not. For example, a function call that starts
//! on the next line without a semicolon:
//!
//! ```js
//! var foo = bar
//! (1 || 2).baz();
//! ```
//!
//! This rule flags cases where `(`, `[`, or a template literal follows a
//! newline after an expression statement.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags confusing multiline expressions that look like separate statements.
#[derive(Debug)]
pub struct NoUnexpectedMultiline;

impl NativeRule for NoUnexpectedMultiline {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unexpected-multiline".to_owned(),
            description: "Disallow confusing multiline expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::CallExpression(call) => {
                let callee_end = callee_end_offset(&call.callee);
                if callee_end > 0 {
                    let has_newline = check_newline_before_paren(
                        ctx.source_text(),
                        callee_end,
                        call.span.end,
                    );
                    if has_newline {
                        ctx.report_error(
                            "no-unexpected-multiline",
                            "Unexpected newline between function name and opening parenthesis",
                            Span::new(call.span.start, call.span.end),
                        );
                    }
                }
            }
            AstKind::TaggedTemplateExpression(tagged) => {
                let tag_end = callee_end_offset(&tagged.tag);
                if tag_end > 0 {
                    let template_start = tagged.quasi.span.start;
                    let has_newline =
                        check_newline_between(ctx.source_text(), tag_end, template_start);
                    if has_newline {
                        ctx.report_error(
                            "no-unexpected-multiline",
                            "Unexpected newline between tag and template literal",
                            Span::new(tagged.span.start, tagged.span.end),
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

/// Get the end offset of an expression.
fn callee_end_offset(expr: &Expression<'_>) -> u32 {
    use oxc_span::GetSpan;
    expr.span().end
}

/// Check if there is a newline before a `(` between two byte offsets.
fn check_newline_before_paren(source: &str, start: u32, end: u32) -> bool {
    let start_idx = usize::try_from(start).unwrap_or(0);
    let end_idx = usize::try_from(end).unwrap_or(0);
    let Some(between) = source.get(start_idx..end_idx) else {
        return false;
    };
    let Some(paren_pos) = between.find('(') else {
        return false;
    };
    let Some(before_paren) = between.get(..paren_pos) else {
        return false;
    };
    before_paren.contains('\n')
}

/// Check if there is a newline between two byte offsets.
fn check_newline_between(source: &str, start: u32, end: u32) -> bool {
    let start_idx = usize::try_from(start).unwrap_or(0);
    let end_idx = usize::try_from(end).unwrap_or(0);
    source
        .get(start_idx..end_idx)
        .is_some_and(|s| s.contains('\n'))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code with the `NoUnexpectedMultiline` rule.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnexpectedMultiline)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_same_line_call() {
        let diags = lint("var x = foo(1);");
        assert!(diags.is_empty(), "same-line call should not be flagged");
    }

    #[test]
    fn test_allows_semicolon_terminated() {
        let diags = lint("var x = foo;\n(1 || 2).baz();");
        assert!(
            diags.is_empty(),
            "semicolon-terminated line should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_code() {
        let diags = lint("var a = 1;\nvar b = 2;");
        assert!(diags.is_empty(), "normal code should not be flagged");
    }
}
