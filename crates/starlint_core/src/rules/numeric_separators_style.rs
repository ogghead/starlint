//! Rule: `numeric-separators-style`
//!
//! Enforce numeric separators in large numeric literals for readability.
//! Flags literals with 5+ digits that do not contain underscores.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags large numeric literals missing numeric separators.
#[derive(Debug)]
pub struct NumericSeparatorsStyle;

/// Count the number of digits in the integer part of a numeric literal.
fn integer_digit_count(raw: &str) -> usize {
    // Strip prefix (0x, 0o, 0b).
    let digits_part = if raw.len() > 2 {
        match raw.as_bytes().get(1) {
            Some(b'x' | b'X' | b'o' | b'O' | b'b' | b'B') => &raw[2..],
            _ => raw,
        }
    } else {
        raw
    };

    // Take only the integer part (before any `.` or `e`/`E`).
    let integer_part = digits_part
        .split(['.', 'e', 'E'])
        .next()
        .unwrap_or(digits_part);

    // Count only digit characters (ignore existing underscores).
    integer_part
        .chars()
        .filter(|c| c.is_ascii_digit() || c.is_ascii_hexdigit())
        .count()
}

impl NativeRule for NumericSeparatorsStyle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "numeric-separators-style".to_owned(),
            description: "Enforce numeric separators in large numeric literals".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NumericLiteral])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NumericLiteral(lit) = kind else {
            return;
        };

        let start = usize::try_from(lit.span.start).unwrap_or(0);
        let end = usize::try_from(lit.span.end).unwrap_or(0);
        let Some(raw) = ctx.source_text().get(start..end) else {
            return;
        };

        // Already has separators — no issue.
        if raw.contains('_') {
            return;
        }

        // Only flag if there are 5+ digits in the integer part.
        if integer_digit_count(raw) < 5 {
            return;
        }

        ctx.report_warning(
            "numeric-separators-style",
            &format!("Numeric literal `{raw}` should use numeric separators for readability"),
            Span::new(lit.span.start, lit.span.end),
        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NumericSeparatorsStyle)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_large_number_without_separator() {
        let diags = lint("const x = 10000;");
        assert_eq!(diags.len(), 1, "should flag 10000");
    }

    #[test]
    fn test_flags_very_large_number() {
        let diags = lint("const x = 1000000;");
        assert_eq!(diags.len(), 1, "should flag 1000000");
    }

    #[test]
    fn test_allows_number_with_separators() {
        let diags = lint("const x = 10_000;");
        assert!(diags.is_empty(), "10_000 should not be flagged");
    }

    #[test]
    fn test_allows_small_number() {
        let diags = lint("const x = 9999;");
        assert!(diags.is_empty(), "4-digit number should not be flagged");
    }
}
