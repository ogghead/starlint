//! Rule: `no-nonoctal-decimal-escape`
//!
//! Disallow `\8` and `\9` escape sequences in string literals. These are not
//! valid octal escapes (octal digits are 0-7), and while they are technically
//! legal in non-strict mode (producing `"8"` and `"9"`), they are deprecated,
//! confusing, and forbidden in strict mode.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `\8` and `\9` escape sequences in string literals.
#[derive(Debug)]
pub struct NoNonoctalDecimalEscape;

impl NativeRule for NoNonoctalDecimalEscape {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-nonoctal-decimal-escape".to_owned(),
            description: "Disallow `\\8` and `\\9` escape sequences in string literals".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::StringLiteral])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StringLiteral(lit) = kind else {
            return;
        };

        // We need to check the raw source text, since the parsed value
        // would already have resolved `\8` to `8`.
        let start = usize::try_from(lit.span.start).unwrap_or(0);
        let end = usize::try_from(lit.span.end).unwrap_or(0);
        let source = ctx.source_text();
        let Some(raw) = source.get(start..end) else {
            return;
        };

        if contains_nonoctal_escape(raw) {
            // Fix: remove backslash before 8 or 9
            let fixed = remove_nonoctal_escapes(raw);
            let fix = Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Remove the backslash before `8` or `9`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(lit.span.start, lit.span.end),
                    replacement: fixed,
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "no-nonoctal-decimal-escape".to_owned(),
                message: "Don't use `\\8` or `\\9` escape sequences in string literals".to_owned(),
                span: Span::new(lit.span.start, lit.span.end),
                severity: Severity::Error,
                help: Some(
                    "Remove the backslash — `\\8` and `\\9` are not valid octal escapes".to_owned(),
                ),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Remove `\8` and `\9` escape sequences by stripping the backslash.
fn remove_nonoctal_escapes(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes.get(i).copied() == Some(b'\\') {
            let next = i.checked_add(1).unwrap_or(len);
            let next_byte = bytes.get(next).copied();
            if next_byte == Some(b'8') || next_byte == Some(b'9') {
                // Skip the backslash, keep the digit
                i = next;
                continue;
            }
            // Keep the backslash and skip the escaped char
            result.push('\\');
            if let Some(&b) = bytes.get(next) {
                result.push(char::from(b));
            }
            i = next.checked_add(1).unwrap_or(len);
        } else {
            if let Some(&b) = bytes.get(i) {
                result.push(char::from(b));
            }
            i = i.checked_add(1).unwrap_or(len);
        }
    }
    result
}

/// Check if a raw string source contains `\8` or `\9` escape sequences.
fn contains_nonoctal_escape(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes.get(i).copied() == Some(b'\\') {
            let next = i.checked_add(1).unwrap_or(len);
            let next_byte = bytes.get(next).copied();
            if next_byte == Some(b'8') || next_byte == Some(b'9') {
                return true;
            }
            // Skip next char to avoid treating `\\8` as `\8`
            i = next.checked_add(1).unwrap_or(len);
        } else {
            i = i.checked_add(1).unwrap_or(len);
        }
    }
    false
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNonoctalDecimalEscape)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_backslash_8() {
        let diags = lint(r#"var x = "\8";"#);
        assert_eq!(diags.len(), 1, "\\8 escape should be flagged");
    }

    #[test]
    fn test_flags_backslash_9() {
        let diags = lint(r#"var x = "\9";"#);
        assert_eq!(diags.len(), 1, "\\9 escape should be flagged");
    }

    #[test]
    fn test_allows_normal_string() {
        let diags = lint(r#"var x = "hello";"#);
        assert!(diags.is_empty(), "normal string should not be flagged");
    }

    #[test]
    fn test_allows_valid_octal() {
        let diags = lint(r#"var x = "\0";"#);
        assert!(diags.is_empty(), "valid octal escape should not be flagged");
    }

    #[test]
    fn test_allows_literal_digit() {
        let diags = lint(r#"var x = "8";"#);
        assert!(diags.is_empty(), "literal digit 8 should not be flagged");
    }

    #[test]
    fn test_allows_double_backslash() {
        // `\\8` is an escaped backslash followed by literal 8
        let diags = lint(r#"var x = "\\8";"#);
        assert!(
            diags.is_empty(),
            "double backslash followed by 8 should not be flagged"
        );
    }
}
