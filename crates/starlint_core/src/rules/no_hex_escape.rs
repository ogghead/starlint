//! Rule: `no-hex-escape` (unicorn)
//!
//! Disallow hex escape sequences `\xNN` in strings — use Unicode escapes
//! `\uNNNN` instead for consistency and clarity.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
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
            ctx.report_warning(
                "no-hex-escape",
                r"Use Unicode escape `\uNNNN` instead of hex escape `\xNN`",
                Span::new(lit.span.start, lit.span.end),
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
