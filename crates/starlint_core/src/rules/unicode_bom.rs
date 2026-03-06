//! Rule: `unicode-bom`
//!
//! Require or disallow the Unicode Byte Order Mark (BOM, U+FEFF).
//! By default, this rule requires that files do NOT start with a BOM.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags files that start with (or are missing) a Unicode BOM.
#[derive(Debug)]
pub struct UnicodeBom;

impl NativeRule for UnicodeBom {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "unicode-bom".to_owned(),
            description: "Require or disallow Unicode BOM".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();

        // Default behavior: disallow BOM
        if source.starts_with('\u{FEFF}') {
            ctx.report(Diagnostic {
                rule_name: "unicode-bom".to_owned(),
                message: "Unexpected Unicode BOM (Byte Order Mark)".to_owned(),
                span: Span::new(0, 3),
                severity: Severity::Warning,
                help: Some("Remove the BOM".to_owned()),
                fix: Some(Fix {
                    message: "Remove the BOM".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(0, 3),
                        replacement: String::new(),
                    }],
                    is_snippet: false,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(UnicodeBom)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_bom() {
        let diags = lint("\u{FEFF}var x = 1;");
        assert_eq!(diags.len(), 1, "BOM at start should be flagged");
    }

    #[test]
    fn test_allows_no_bom() {
        let diags = lint("var x = 1;");
        assert!(diags.is_empty(), "no BOM should not be flagged");
    }

    #[test]
    fn test_allows_empty_file() {
        let diags = lint("");
        assert!(diags.is_empty(), "empty file should not be flagged");
    }

    #[test]
    fn test_allows_bom_in_middle() {
        // BOM in the middle of a file is handled by no-irregular-whitespace,
        // not by this rule. This rule only checks the file start.
        let diags = lint("var x = '\u{FEFF}';");
        assert!(
            diags.is_empty(),
            "BOM in middle should not be flagged by this rule"
        );
    }
}
