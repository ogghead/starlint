//! Rule: `no-empty-file` (unicorn)
//!
//! Disallow empty files. An empty file is likely a mistake or leftover
//! from a refactoring and should be removed.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags files that contain no meaningful code.
#[derive(Debug)]
pub struct NoEmptyFile;

impl NativeRule for NoEmptyFile {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-empty-file".to_owned(),
            description: "Disallow empty files".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let is_empty = {
            let source = ctx.source_text();
            // A file is "empty" if it only contains whitespace, comments,
            // and/or a shebang line
            let trimmed = source.trim();
            trimmed.is_empty() || is_only_comments(trimmed)
        };

        if is_empty {
            ctx.report_warning(
                "no-empty-file",
                "Empty files are not allowed",
                Span::new(0, 0),
            );
        }
    }
}

/// Check if the trimmed source is only comments (no actual code).
fn is_only_comments(source: &str) -> bool {
    let mut rest = source;

    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            return true;
        }

        if rest.starts_with("//") {
            // Single-line comment — skip to end of line
            rest = rest
                .find('\n')
                .map_or("", |pos| rest.get(pos.saturating_add(1)..).unwrap_or(""));
        } else if rest.starts_with("/*") {
            // Block comment — skip to */
            let Some(end) = rest.find("*/") else {
                // Unterminated block comment — treat as all-comment
                return true;
            };
            rest = rest.get(end.saturating_add(2)..).unwrap_or("");
        } else if rest.starts_with("#!") {
            // Shebang — skip to end of line
            rest = rest
                .find('\n')
                .map_or("", |pos| rest.get(pos.saturating_add(1)..).unwrap_or(""));
        } else {
            // Found actual code
            return false;
        }
    }

    true
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEmptyFile)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_file() {
        let diags = lint("");
        assert_eq!(diags.len(), 1, "empty file should be flagged");
    }

    #[test]
    fn test_flags_whitespace_only() {
        let diags = lint("   \n\n  \n  ");
        assert_eq!(diags.len(), 1, "whitespace-only file should be flagged");
    }

    #[test]
    fn test_flags_comments_only() {
        let diags = lint("// just a comment\n/* another */");
        assert_eq!(diags.len(), 1, "comments-only file should be flagged");
    }

    #[test]
    fn test_allows_file_with_code() {
        let diags = lint("var x = 1;");
        assert!(diags.is_empty(), "file with code should not be flagged");
    }

    #[test]
    fn test_allows_code_after_comment() {
        let diags = lint("// header comment\nvar x = 1;");
        assert!(
            diags.is_empty(),
            "file with code after comment should not be flagged"
        );
    }
}
