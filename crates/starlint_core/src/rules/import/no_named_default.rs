//! Rule: `import/no-named-default`
//!
//! Forbid named default exports (`export { default }`).
//! Prefer `export default` syntax for clarity.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags re-exports of the `default` name via named export syntax.
#[derive(Debug)]
pub struct NoNamedDefault;

impl NativeRule for NoNamedDefault {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-named-default".to_owned(),
            description: "Forbid named default exports".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let findings: Vec<(u32, u32)> = {
            let source = ctx.source_text();
            source
                .lines()
                .enumerate()
                .filter_map(|(idx, line)| {
                    let trimmed = line.trim();
                    (trimmed.starts_with("import ") && trimmed.contains("{ default ")).then(|| {
                        let line_offset = source
                            .lines()
                            .take(idx)
                            .map(|l| l.len().saturating_add(1))
                            .sum::<usize>();
                        let start = u32::try_from(line_offset).unwrap_or(0);
                        let end =
                            u32::try_from(line_offset.saturating_add(trimmed.len())).unwrap_or(0);
                        (start, end)
                    })
                })
                .collect()
        };

        for (start, end) in findings {
            ctx.report_warning(
                "import/no-named-default",
                "Use default import syntax instead of named `default` import",
                Span::new(start, end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNamedDefault)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_named_default_import() {
        let diags = lint(r#"import { default as foo } from "mod";"#);
        assert_eq!(diags.len(), 1, "named default import should be flagged");
    }

    #[test]
    fn test_allows_regular_default_import() {
        let diags = lint(r#"import foo from "mod";"#);
        assert!(
            diags.is_empty(),
            "regular default import should not be flagged"
        );
    }

    #[test]
    fn test_allows_named_import() {
        let diags = lint(r#"import { foo } from "mod";"#);
        assert!(diags.is_empty(), "named import should not be flagged");
    }
}
