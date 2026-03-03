//! Rule: `prefer-blob-reading-methods`
//!
//! Prefer `Blob` reading methods (`blob.text()`, `blob.arrayBuffer()`,
//! `blob.stream()`) over using `FileReader`. The `Blob` API is simpler,
//! promise-based, and avoids the callback-based `FileReader` pattern.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new FileReader()` — prefer `Blob` reading methods instead.
#[derive(Debug)]
pub struct PreferBlobReadingMethods;

impl NativeRule for PreferBlobReadingMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-blob-reading-methods".to_owned(),
            description: "Prefer `Blob` reading methods over `FileReader`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        if let Expression::Identifier(id) = &new_expr.callee {
            if id.name.as_str() == "FileReader" {
                ctx.report_warning(
                    "prefer-blob-reading-methods",
                    "Prefer `Blob` reading methods (`blob.text()`, `blob.arrayBuffer()`, `blob.stream()`) over `FileReader`",
                    Span::new(new_expr.span.start, new_expr.span.end),
                );
            }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferBlobReadingMethods)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_filereader() {
        let diags = lint("var reader = new FileReader();");
        assert_eq!(diags.len(), 1, "new FileReader() should be flagged");
    }

    #[test]
    fn test_allows_blob_text() {
        let diags = lint("blob.text();");
        assert!(diags.is_empty(), "blob.text() should not be flagged");
    }

    #[test]
    fn test_allows_blob_array_buffer() {
        let diags = lint("blob.arrayBuffer();");
        assert!(diags.is_empty(), "blob.arrayBuffer() should not be flagged");
    }

    #[test]
    fn test_allows_blob_stream() {
        let diags = lint("blob.stream();");
        assert!(diags.is_empty(), "blob.stream() should not be flagged");
    }

    #[test]
    fn test_allows_other_new_expression() {
        let diags = lint("var x = new Map();");
        assert!(diags.is_empty(), "new Map() should not be flagged");
    }
}
