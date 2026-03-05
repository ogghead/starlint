//! Rule: `text-encoding-identifier-case` (unicorn)
//!
//! Enforce consistent casing for text encoding identifiers. Prefer `'utf-8'`
//! over `'UTF-8'`, `'utf8'`, `'Utf8'`, etc.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags non-canonical text encoding identifier casing.
#[derive(Debug)]
pub struct TextEncodingIdentifierCase;

/// Known encoding identifiers and their canonical forms.
fn canonical_encoding(value: &str) -> Option<&'static str> {
    let lower = value.to_ascii_lowercase();
    match lower.as_str() {
        "utf-8" | "utf8" => {
            if value == "utf-8" {
                None // already canonical
            } else {
                Some("utf-8")
            }
        }
        "ascii" => {
            if value == "ascii" {
                None
            } else {
                Some("ascii")
            }
        }
        "utf-16le" | "utf16le" => {
            if value == "utf-16le" {
                None
            } else {
                Some("utf-16le")
            }
        }
        "utf-16be" | "utf16be" => {
            if value == "utf-16be" {
                None
            } else {
                Some("utf-16be")
            }
        }
        _ => None,
    }
}

impl NativeRule for TextEncodingIdentifierCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "text-encoding-identifier-case".to_owned(),
            description: "Enforce consistent casing for text encoding identifiers".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::StringLiteral])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StringLiteral(lit) = kind else {
            return;
        };

        let value = lit.value.as_str();
        if let Some(canonical) = canonical_encoding(value) {
            // Replace the string content inside the quotes (span includes quotes).
            let content_start = lit.span.start.saturating_add(1);
            let content_end = lit.span.end.saturating_sub(1);

            ctx.report(Diagnostic {
                rule_name: "text-encoding-identifier-case".to_owned(),
                message: format!("Prefer `'{canonical}'` over `'{value}'`"),
                span: Span::new(lit.span.start, lit.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace `'{value}'` with `'{canonical}'`")),
                fix: Some(Fix {
                    message: format!("Replace with `'{canonical}'`"),
                    edits: vec![Edit {
                        span: Span::new(content_start, content_end),
                        replacement: canonical.to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(TextEncodingIdentifierCase)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_uppercase_utf8() {
        let diags = lint("const enc = 'UTF-8';");
        assert_eq!(diags.len(), 1, "UTF-8 should be flagged");
    }

    #[test]
    fn test_flags_utf8_without_hyphen() {
        let diags = lint("const enc = 'utf8';");
        assert_eq!(diags.len(), 1, "utf8 should be flagged");
    }

    #[test]
    fn test_allows_lowercase_utf8() {
        let diags = lint("const enc = 'utf-8';");
        assert!(diags.is_empty(), "utf-8 should not be flagged");
    }

    #[test]
    fn test_flags_uppercase_ascii() {
        let diags = lint("const enc = 'ASCII';");
        assert_eq!(diags.len(), 1, "ASCII should be flagged");
    }

    #[test]
    fn test_allows_lowercase_ascii() {
        let diags = lint("const enc = 'ascii';");
        assert!(diags.is_empty(), "ascii should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_string() {
        let diags = lint("const x = 'hello';");
        assert!(diags.is_empty(), "unrelated strings should not be flagged");
    }
}
