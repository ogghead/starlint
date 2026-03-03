//! Rule: `jest/no-export`
//!
//! Error when test files contain exports. Test files should not export anything.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-export";

/// Flags export declarations in test files.
#[derive(Debug)]
pub struct NoExport;

/// Check if a file path looks like a test file.
fn is_test_file(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains(".test.") || path_str.contains(".spec.")
}

impl NativeRule for NoExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow exports from test files".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ExportAllDeclaration,
            AstType::ExportDefaultDeclaration,
            AstType::ExportNamedDeclaration,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Only apply to test files
        if !is_test_file(ctx.file_path()) {
            return;
        }

        match kind {
            AstKind::ExportNamedDeclaration(decl) => {
                ctx.report_error(
                    RULE_NAME,
                    "Test files should not export anything",
                    Span::new(decl.span.start, decl.span.end),
                );
            }
            AstKind::ExportDefaultDeclaration(decl) => {
                ctx.report_error(
                    RULE_NAME,
                    "Test files should not export anything",
                    Span::new(decl.span.start, decl.span.end),
                );
            }
            AstKind::ExportAllDeclaration(decl) => {
                ctx.report_error(
                    RULE_NAME,
                    "Test files should not export anything",
                    Span::new(decl.span.start, decl.span.end),
                );
            }
            _ => {}
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    fn lint_non_test(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("utils.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("utils.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_named_export_in_test() {
        let diags = lint("export const helper = () => {};");
        assert_eq!(
            diags.len(),
            1,
            "named export in test file should be flagged"
        );
    }

    #[test]
    fn test_flags_default_export_in_test() {
        let diags = lint("export default function() {}");
        assert_eq!(
            diags.len(),
            1,
            "default export in test file should be flagged"
        );
    }

    #[test]
    fn test_allows_export_in_non_test() {
        let diags = lint_non_test("export const helper = () => {};");
        assert!(
            diags.is_empty(),
            "export in non-test file should not be flagged"
        );
    }
}
