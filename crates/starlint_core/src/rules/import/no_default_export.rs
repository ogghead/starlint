//! Rule: `import/no-default-export`
//!
//! Disallow default exports. Named exports are preferable because they
//! enforce a consistent import name, improve refactoring tooling, and
//! make tree-shaking more effective.

use oxc_ast::AstKind;
use oxc_ast::ast::ExportDefaultDeclarationKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags any `export default` declaration.
#[derive(Debug)]
pub struct NoDefaultExport;

impl NativeRule for NoDefaultExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-default-export".to_owned(),
            description: "Disallow default exports".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ExportDefaultDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ExportDefaultDeclaration(export) = kind else {
            return;
        };

        // For named function/class declarations, suggest removing `default`.
        // `export default function foo()` → `export function foo()`
        // `export default class Foo` → `export class Foo`
        // Skip anonymous/expression exports since they can't become named exports.
        let fix = match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(func) if func.id.is_some() => {
                // Replace "export default " (15 chars) with "export "
                let kw_end = export.span.start.saturating_add(15);
                FixBuilder::new("Convert to named export")
                    .replace(Span::new(export.span.start, kw_end), "export ")
                    .build()
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) if class.id.is_some() => {
                let kw_end = export.span.start.saturating_add(15);
                FixBuilder::new("Convert to named export")
                    .replace(Span::new(export.span.start, kw_end), "export ")
                    .build()
            }
            _ => None,
        };

        ctx.report(Diagnostic {
            rule_name: "import/no-default-export".to_owned(),
            message: "Prefer named exports over default exports".to_owned(),
            span: Span::new(export.span.start, export.span.end),
            severity: Severity::Warning,
            help: Some("Use a named export instead".to_owned()),
            fix,
            labels: vec![],
        });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDefaultExport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_default_export_function() {
        let diags = lint("export default function foo() {}");
        assert_eq!(diags.len(), 1, "default export function should be flagged");
    }

    #[test]
    fn test_flags_default_export_value() {
        let diags = lint("export default 42;");
        assert_eq!(diags.len(), 1, "default export value should be flagged");
    }

    #[test]
    fn test_allows_named_export() {
        let diags = lint("export const foo = 42;");
        assert!(diags.is_empty(), "named export should not be flagged");
    }
}
