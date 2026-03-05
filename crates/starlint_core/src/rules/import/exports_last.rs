//! Rule: `import/exports-last`
//!
//! Require all exports to appear after other statements in the module body.
//! Non-export statements interleaved among exports make it harder to see
//! what a module exposes at a glance.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags non-export statements that appear after the first export.
#[derive(Debug)]
pub struct ExportsLast;

impl NativeRule for ExportsLast {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/exports-last".to_owned(),
            description: "Require all exports to appear after other statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();

        // Track the last export line and look for non-export statements after it.
        let mut last_export_line: Option<usize> = None;
        let mut byte_offset: usize = 0;

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
                byte_offset = byte_offset.saturating_add(line.len()).saturating_add(1);
                continue;
            }

            let is_export = trimmed.starts_with("export ") || trimmed.starts_with("export{");

            if is_export {
                last_export_line = Some(line_idx);
            } else if let Some(last) = last_export_line {
                // Non-export statement after an export
                if line_idx > last {
                    let start = u32::try_from(byte_offset).unwrap_or(0);
                    let end = start.saturating_add(u32::try_from(line.len()).unwrap_or(0));
                    ctx.report(Diagnostic {
                        rule_name: "import/exports-last".to_owned(),
                        message: "Non-export statement found after an export — move all exports to the end of the file".to_owned(),
                        span: Span::new(start, end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }

            byte_offset = byte_offset.saturating_add(line.len()).saturating_add(1);
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ExportsLast)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_statement_after_export() {
        let source = "export const a = 1;\nconst b = 2;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "non-export after export should be flagged");
    }

    #[test]
    fn test_allows_exports_at_end() {
        let source = "const a = 1;\nconst b = 2;\nexport { a, b };";
        let diags = lint(source);
        assert!(diags.is_empty(), "exports at end should be allowed");
    }

    #[test]
    fn test_allows_only_exports() {
        let source = "export const a = 1;\nexport const b = 2;";
        let diags = lint(source);
        assert!(diags.is_empty(), "only exports should be allowed");
    }
}
