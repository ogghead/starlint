//! Rule: `import/group-exports`
//!
//! Prefer use of a single export declaration rather than scattered exports
//! throughout the file. This makes it easier to see what a module provides
//! at a glance.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags modules with multiple named export declarations that could be grouped.
#[derive(Debug)]
pub struct GroupExports;

impl NativeRule for GroupExports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/group-exports".to_owned(),
            description: "Prefer a single export declaration rather than scattered exports"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Program])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Program(program) = kind else {
            return;
        };

        // Collect all named export declarations (excluding re-exports)
        let named_exports: Vec<&oxc_ast::ast::ExportNamedDeclaration<'_>> = program
            .body
            .iter()
            .filter_map(|stmt| {
                if let oxc_ast::ast::Statement::ExportNamedDeclaration(export) = stmt {
                    // Only count local exports, not re-exports like `export { x } from 'y'`
                    if export.source.is_none() {
                        return Some(export.as_ref());
                    }
                }
                None
            })
            .collect();

        // If there are more than one named export declaration, flag all but the first
        if named_exports.len() > 1 {
            for export in named_exports.iter().skip(1) {
                ctx.report_warning(
                    "import/group-exports",
                    "Multiple named export declarations; prefer a single export { ... }",
                    Span::new(export.span.start, export.span.end),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(GroupExports)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_scattered_exports() {
        let source = "export const a = 1;\nexport const b = 2;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "scattered named exports should be flagged");
    }

    #[test]
    fn test_allows_single_export() {
        let source = "const a = 1;\nconst b = 2;\nexport { a, b };";
        let diags = lint(source);
        assert!(diags.is_empty(), "single grouped export should be fine");
    }

    #[test]
    fn test_allows_re_exports() {
        let source = "export { foo } from 'foo';\nexport { bar } from 'bar';";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "re-exports from different modules should not be flagged"
        );
    }
}
