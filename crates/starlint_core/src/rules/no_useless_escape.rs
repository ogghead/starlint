//! Rule: `no-useless-escape`
//!
//! Disallow unnecessary escape characters in strings and regular expressions.
//! Characters that don't need escaping produce unnecessary visual noise.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unnecessary escape characters in string literals.
#[derive(Debug)]
pub struct NoUselessEscape;

/// Check if a character needs escaping in a string literal.
const fn is_meaningful_escape(ch: char) -> bool {
    matches!(
        ch,
        '\\' | 'n' | 'r' | 't' | 'b' | 'f' | 'v' | 'u' | '0' | '\'' | '"' | '`' | '\n' | '\r'
    ) || ch.is_ascii_digit()
        || ch == 'x'
}

impl NativeRule for NoUselessEscape {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-escape".to_owned(),
            description: "Disallow unnecessary escape characters".to_owned(),
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
        let raw = source.get(start..end).unwrap_or("");

        // Need at least 2 chars for opening and closing quotes
        if raw.len() < 2 {
            return;
        }

        // Strip the quote characters (first and last)
        let inner = &raw[1..raw.len().saturating_sub(1)];
        let has_useless = has_useless_escape(inner);
        let span_start = lit.span.start;
        let span_end = lit.span.end;

        if has_useless {
            ctx.report_warning(
                "no-useless-escape",
                "Unnecessary escape character",
                Span::new(span_start, span_end),
            );
        }
    }
}

/// Scan inner string content (without quotes) for useless backslash escapes.
fn has_useless_escape(inner: &str) -> bool {
    let mut chars = inner.chars();
    loop {
        match chars.next() {
            None => return false,
            Some('\\') => match chars.next() {
                None => return false,
                Some(next_ch) => {
                    if !is_meaningful_escape(next_ch) {
                        return true;
                    }
                }
            },
            Some(_) => {}
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessEscape)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_useless_escape() {
        let diags = lint(r#"var x = "hell\o";"#);
        assert!(!diags.is_empty(), "useless escape of 'o' should be flagged");
    }

    #[test]
    fn test_allows_needed_escape() {
        let diags = lint(r#"var x = "hello\nworld";"#);
        assert!(diags.is_empty(), "newline escape should not be flagged");
    }

    #[test]
    fn test_allows_quote_escape() {
        let diags = lint(r#"var x = "it\'s";"#);
        assert!(diags.is_empty(), "quote escape should not be flagged");
    }
}
