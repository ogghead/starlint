//! Rule: `no-empty-file` (unicorn)
//!
//! Disallow empty files. An empty file is likely a mistake or leftover
//! from a refactoring and should be removed.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags files that contain no meaningful code.
#[derive(Debug)]
pub struct NoEmptyFile;

impl LintRule for NoEmptyFile {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-empty-file".to_owned(),
            description: "Disallow empty files".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let is_empty = {
            let source = ctx.source_text();
            // A file is "empty" if it only contains whitespace, comments,
            // and/or a shebang line
            let trimmed = source.trim();
            trimmed.is_empty() || is_only_comments(trimmed)
        };

        if is_empty {
            ctx.report(Diagnostic {
                rule_name: "no-empty-file".to_owned(),
                message: "Empty files are not allowed".to_owned(),
                span: Span::new(0, 0),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
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
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoEmptyFile)];
        lint_source(source, "test.js", &rules)
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
