//! Rule: `import/no-anonymous-default-export`
//!
//! Disallow anonymous default exports. Named default exports improve
//! stack traces, refactoring tools, and make the module's API clearer.

use oxc_ast::AstKind;
use oxc_ast::ast::ExportDefaultDeclarationKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags anonymous default export declarations.
#[derive(Debug)]
pub struct NoAnonymousDefaultExport;

impl NativeRule for NoAnonymousDefaultExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-anonymous-default-export".to_owned(),
            description: "Disallow anonymous default exports".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ExportDefaultDeclaration(export) = kind else {
            return;
        };

        let is_anonymous = match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(f)
            | ExportDefaultDeclarationKind::FunctionExpression(f) => f.id.is_none(),
            ExportDefaultDeclarationKind::ClassDeclaration(c)
            | ExportDefaultDeclarationKind::ClassExpression(c) => c.id.is_none(),
            // TS interfaces and identifier references are named
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_)
            | ExportDefaultDeclarationKind::Identifier(_) => false,
            // Everything else (arrow functions, literals, objects, etc.) is anonymous
            _ => true,
        };

        if is_anonymous {
            ctx.report_warning(
                "import/no-anonymous-default-export",
                "Assign a name to the default export for better debugging and refactoring",
                Span::new(export.span.start, export.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAnonymousDefaultExport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_anonymous_arrow_function() {
        let diags = lint("export default () => {};");
        assert_eq!(
            diags.len(),
            1,
            "anonymous arrow function default export should be flagged"
        );
    }

    #[test]
    fn test_flags_anonymous_object() {
        let diags = lint("export default {};");
        assert_eq!(
            diags.len(),
            1,
            "anonymous object default export should be flagged"
        );
    }

    #[test]
    fn test_allows_named_function() {
        let diags = lint("export default function myFunc() {}");
        assert!(
            diags.is_empty(),
            "named function default export should not be flagged"
        );
    }
}
