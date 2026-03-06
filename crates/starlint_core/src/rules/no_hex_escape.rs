//! Rule: `no-hex-escape` (unicorn)
//!
//! Disallow hex escape sequences `\xNN` in strings — use Unicode escapes
//! `\uNNNN` instead for consistency and clarity.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags hex escape sequences in string literals.
#[derive(Debug)]
pub struct NoHexEscape;

impl NativeRule for NoHexEscape {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-hex-escape".to_owned(),
            description: r"Disallow `\xNN` hex escapes — use `\uNNNN` instead".to_owned(),
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

        // Check the raw source for \x escapes
        let source = ctx.source_text();
        let start = usize::try_from(lit.span.start).unwrap_or(0);
        let end = usize::try_from(lit.span.end).unwrap_or(0);
        let Some(raw) = source.get(start..end) else {
            return;
        };

        let finding = raw.contains("\\x");
        if finding {
            let fixed = convert_hex_to_unicode(raw);

            ctx.report(Diagnostic {
                rule_name: "no-hex-escape".to_owned(),
                message: r"Use Unicode escape `\uNNNN` instead of hex escape `\xNN`".to_owned(),
                span: Span::new(lit.span.start, lit.span.end),
                severity: Severity::Warning,
                help: Some(r"Replace `\xNN` with `\u00NN`".to_owned()),
                fix: Some(Fix {
                    message: r"Convert hex escapes to Unicode escapes".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(lit.span.start, lit.span.end),
                        replacement: fixed,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Convert `\xNN` hex escapes to `\u00NN` Unicode escapes in raw source.
fn convert_hex_to_unicode(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }
        if chars.peek() == Some(&'x') {
            let _ = chars.next(); // consume 'x'
            // Collect 2 hex digits
            let mut hex = String::new();
            for _ in 0..2 {
                if let Some(&c) = chars.peek() {
                    if c.is_ascii_hexdigit() {
                        hex.push(c);
                        let _ = chars.next();
                    } else {
                        break;
                    }
                }
            }
            if hex.len() == 2 {
                result.push_str("\\u00");
            } else {
                // Incomplete hex escape, keep as-is
                result.push('\\');
                result.push('x');
            }
            result.push_str(&hex);
        } else {
            result.push('\\');
            if let Some(&next) = chars.peek() {
                result.push(next);
                let _ = chars.next();
            }
        }
    }
    result
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoHexEscape)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_hex_escape() {
        let diags = lint(r"var s = '\x41';");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_unicode_escape() {
        let diags = lint(r"var s = '\u0041';");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_normal_string() {
        let diags = lint(r"var s = 'hello';");
        assert!(diags.is_empty());
    }
}
