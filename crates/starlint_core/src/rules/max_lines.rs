//! Rule: `max-lines`
//!
//! Enforce a maximum number of lines per file. Very large files are harder
//! to understand and maintain — prefer splitting into smaller modules.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Default maximum lines per file.
const DEFAULT_MAX: u32 = 300;

/// Flags files exceeding a maximum number of lines.
#[derive(Debug)]
pub struct MaxLines {
    /// Maximum number of lines allowed per file.
    max: u32,
}

impl MaxLines {
    #[must_use]
    pub const fn new() -> Self {
        Self { max: DEFAULT_MAX }
    }
}

impl Default for MaxLines {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRule for MaxLines {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "max-lines".to_owned(),
            description: "Enforce a maximum number of lines per file".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(n) = config.get("max").and_then(serde_json::Value::as_u64) {
            self.max = u32::try_from(n).unwrap_or(DEFAULT_MAX);
        }
        Ok(())
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();
        let raw_count = u32::try_from(source.lines().count()).unwrap_or(0);
        // Account for trailing newline (lines() strips it)
        let line_count = if source.ends_with('\n') {
            raw_count.saturating_add(1)
        } else {
            raw_count
        };

        if line_count > self.max {
            let source_len = u32::try_from(source.len()).unwrap_or(0);
            ctx.report(Diagnostic {
                rule_name: "max-lines".to_owned(),
                message: format!(
                    "File has too many lines ({line_count}). Maximum allowed is {}",
                    self.max
                ),
                span: Span::new(0, source_len),
                severity: Severity::Warning,
                help: None,
                fix: None,
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

    fn lint_with_max(source: &str, max: u32) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MaxLines { max })];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_short_file() {
        let diags = lint_with_max("var x = 1;\n", 10);
        assert!(diags.is_empty(), "short file should not be flagged");
    }

    #[test]
    fn test_flags_long_file() {
        let source = "var a = 1;\nvar b = 2;\nvar c = 3;\nvar d = 4;\nvar e = 5;\n";
        let diags = lint_with_max(source, 3);
        assert_eq!(diags.len(), 1, "long file should be flagged");
    }

    #[test]
    fn test_allows_at_limit() {
        let source = "var a = 1;\nvar b = 2;\nvar c = 3;\n";
        // 3 lines of content + trailing newline = 4 lines
        let diags = lint_with_max(source, 4);
        assert!(diags.is_empty(), "file at limit should not be flagged");
    }

    #[test]
    fn test_single_line() {
        let diags = lint_with_max("var x = 1;", 1);
        assert!(diags.is_empty(), "single line should not be flagged");
    }
}
