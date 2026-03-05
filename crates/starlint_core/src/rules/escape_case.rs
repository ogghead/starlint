//! Rule: `escape-case` (unicorn)
//!
//! Require escape sequences to use uppercase hex digits. For example,
//! `\xff` should be `\xFF` and `\u00ff` should be `\u00FF`.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags escape sequences with lowercase hex digits.
#[derive(Debug)]
pub struct EscapeCase;

impl NativeRule for EscapeCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "escape-case".to_owned(),
            description: "Require uppercase hex digits in escape sequences".to_owned(),
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

        let source = ctx.source_text();
        let start = usize::try_from(lit.span.start).unwrap_or(0);
        let end = usize::try_from(lit.span.end).unwrap_or(0);
        let Some(raw) = source.get(start..end) else {
            return;
        };

        // Check for \xNN or \uNNNN with lowercase hex digits
        let finding = has_lowercase_escape(raw);
        if finding {
            let fixed = uppercase_escapes(raw);

            ctx.report(Diagnostic {
                rule_name: "escape-case".to_owned(),
                message: "Use uppercase hex digits in escape sequences (e.g., `\\xFF` not `\\xff`)"
                    .to_owned(),
                span: Span::new(lit.span.start, lit.span.end),
                severity: Severity::Warning,
                help: Some("Uppercase hex digits in escape sequences".to_owned()),
                fix: Some(Fix {
                    message: "Uppercase hex digits".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(lit.span.start, lit.span.end),
                        replacement: fixed,
                    }],
                }),
                labels: vec![],
            });
        }
    }
}

/// Produce a new raw string with all hex digits in escape sequences uppercased.
fn uppercase_escapes(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }
        result.push('\\');
        match chars.peek() {
            Some('x') => {
                if let Some(x) = chars.next() {
                    result.push(x);
                } // push 'x'
                for _ in 0..2 {
                    if let Some(&c) = chars.peek() {
                        if c.is_ascii_hexdigit() {
                            result.push(c.to_ascii_uppercase());
                            let _ = chars.next();
                        } else {
                            break;
                        }
                    }
                }
            }
            Some('u') => {
                if let Some(u) = chars.next() {
                    result.push(u);
                } // push 'u'
                if chars.peek() == Some(&'{') {
                    if let Some(brace) = chars.next() {
                        result.push(brace);
                    } // push '{'
                    while let Some(&c) = chars.peek() {
                        if c == '}' {
                            if let Some(close) = chars.next() {
                                result.push(close);
                            }
                            break;
                        }
                        if c.is_ascii_hexdigit() {
                            result.push(c.to_ascii_uppercase());
                        } else {
                            result.push(c);
                        }
                        let _ = chars.next();
                    }
                } else {
                    for _ in 0..4 {
                        if let Some(&c) = chars.peek() {
                            if c.is_ascii_hexdigit() {
                                result.push(c.to_ascii_uppercase());
                                let _ = chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
            Some(&next) => {
                result.push(next);
                let _ = chars.next();
            }
            None => {}
        }
    }
    result
}

/// Check if a raw string contains escape sequences with lowercase hex digits.
///
/// Looks for `\xNN` or `\uNNNN` where the hex digits contain lowercase a-f.
fn has_lowercase_escape(raw: &str) -> bool {
    // Simple regex-free check: find `\x` or `\u` followed by lowercase hex
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            continue;
        }
        match chars.peek() {
            Some('x') => {
                let _x = chars.next(); // consume 'x'
                // Check next 2 hex digits
                for _ in 0..2 {
                    if let Some(&c) = chars.peek() {
                        if c.is_ascii_hexdigit() && c.is_ascii_lowercase() {
                            return true;
                        }
                        let _skip = chars.next();
                    }
                }
            }
            Some('u') => {
                let _u = chars.next(); // consume 'u'
                if chars.peek() == Some(&'{') {
                    // \u{...} form
                    let _brace = chars.next();
                    while let Some(&c) = chars.peek() {
                        if c == '}' {
                            let _close = chars.next();
                            break;
                        }
                        if c.is_ascii_hexdigit() && c.is_ascii_lowercase() {
                            return true;
                        }
                        let _skip = chars.next();
                    }
                } else {
                    // \uNNNN form — check next 4 hex digits
                    for _ in 0..4 {
                        if let Some(&c) = chars.peek() {
                            if c.is_ascii_hexdigit() && c.is_ascii_lowercase() {
                                return true;
                            }
                            let _skip = chars.next();
                        }
                    }
                }
            }
            Some(_) => {
                let _skip = chars.next();
            }
            None => {}
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(EscapeCase)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_lowercase_hex_escape() {
        let diags = lint(r"var s = '\xff';");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_uppercase_hex_escape() {
        let diags = lint(r"var s = '\xFF';");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_flags_lowercase_unicode_escape() {
        let diags = lint(r"var s = '\u00ff';");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_uppercase_unicode_escape() {
        let diags = lint(r"var s = '\u00FF';");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_normal_string() {
        let diags = lint(r"var s = 'hello';");
        assert!(diags.is_empty());
    }
}
