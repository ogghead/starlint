//! Rule: `no-irregular-whitespace`
//!
//! Disallow irregular whitespace characters outside of strings and comments.
//! Characters like non-breaking space (U+00A0), zero-width space (U+200B),
//! and others can cause unexpected behavior and are almost invisible.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Irregular whitespace characters to detect.
const IRREGULAR_WHITESPACE: &[char] = &[
    '\u{00A0}', // NO-BREAK SPACE
    '\u{1680}', // OGHAM SPACE MARK
    '\u{2000}', // EN QUAD
    '\u{2001}', // EM QUAD
    '\u{2002}', // EN SPACE
    '\u{2003}', // EM SPACE
    '\u{2004}', // THREE-PER-EM SPACE
    '\u{2005}', // FOUR-PER-EM SPACE
    '\u{2006}', // SIX-PER-EM SPACE
    '\u{2007}', // FIGURE SPACE
    '\u{2008}', // PUNCTUATION SPACE
    '\u{2009}', // THIN SPACE
    '\u{200A}', // HAIR SPACE
    '\u{200B}', // ZERO WIDTH SPACE
    '\u{202F}', // NARROW NO-BREAK SPACE
    '\u{205F}', // MEDIUM MATHEMATICAL SPACE
    '\u{3000}', // IDEOGRAPHIC SPACE
    '\u{FEFF}', // ZERO WIDTH NO-BREAK SPACE (BOM)
];

/// Flags irregular whitespace in source text (outside of string literals).
#[derive(Debug)]
pub struct NoIrregularWhitespace;

impl NativeRule for NoIrregularWhitespace {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-irregular-whitespace".to_owned(),
            description: "Disallow irregular whitespace".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        // Collect findings first to avoid borrow conflict with ctx
        let findings: Vec<(u32, u32, char)> = {
            let source = ctx.source_text();
            let mut results = Vec::new();
            let mut offset: u32 = 0;
            for ch in source.chars() {
                if IRREGULAR_WHITESPACE.contains(&ch) {
                    let char_len = u32::try_from(ch.len_utf8()).unwrap_or(1);
                    results.push((offset, char_len, ch));
                }
                offset = offset
                    .checked_add(u32::try_from(ch.len_utf8()).unwrap_or(1))
                    .unwrap_or(offset);
            }
            results
        };

        for (offset, char_len, ch) in findings {
            ctx.report_error(
                "no-irregular-whitespace",
                &format!(
                    "Irregular whitespace character U+{:04X} not allowed",
                    u32::from(ch)
                ),
                Span::new(offset, offset.checked_add(char_len).unwrap_or(offset)),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoIrregularWhitespace)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_normal_whitespace() {
        let diags = lint("var x = 1;\nvar y = 2;");
        assert!(diags.is_empty(), "normal whitespace should not be flagged");
    }

    #[test]
    fn test_flags_nbsp() {
        let diags = lint("var\u{00A0}x = 1;");
        assert_eq!(diags.len(), 1, "non-breaking space should be flagged");
    }

    #[test]
    fn test_flags_zero_width_space() {
        let diags = lint("var\u{200B}x = 1;");
        assert_eq!(diags.len(), 1, "zero-width space should be flagged");
    }

    #[test]
    fn test_flags_bom_in_middle() {
        let diags = lint("var x\u{FEFF}= 1;");
        assert_eq!(diags.len(), 1, "BOM in middle of source should be flagged");
    }

    #[test]
    fn test_allows_tabs_and_spaces() {
        let diags = lint("var x\t= 1;  var y = 2;");
        assert!(
            diags.is_empty(),
            "tabs and spaces should not be flagged"
        );
    }
}
