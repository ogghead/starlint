//! Rule: `no-anonymous-default-export`
//!
//! Disallow anonymous default exports. Named exports improve discoverability
//! and make refactoring safer because tools can track references by name.

use oxc_ast::AstKind;
use oxc_ast::ast::ExportDefaultDeclarationKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags anonymous default exports (functions, classes, and expressions).
#[derive(Debug)]
pub struct NoAnonymousDefaultExport;

impl NativeRule for NoAnonymousDefaultExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-anonymous-default-export".to_owned(),
            description: "Disallow anonymous default exports".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ExportDefaultDeclaration(decl) = kind else {
            return;
        };

        let is_anonymous = match &decl.declaration {
            // Named function/class declarations and expressions are fine
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
                "no-anonymous-default-export",
                "Assign a name to this default export for better discoverability",
                Span::new(decl.span.start, decl.span.end),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAnonymousDefaultExport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_anonymous_function() {
        let diags = lint("export default function() {}");
        assert_eq!(
            diags.len(),
            1,
            "anonymous default function should be flagged"
        );
    }

    #[test]
    fn test_allows_named_function() {
        let diags = lint("export default function foo() {}");
        assert!(
            diags.is_empty(),
            "named default function should not be flagged"
        );
    }

    #[test]
    fn test_flags_anonymous_class() {
        let diags = lint("export default class {}");
        assert_eq!(diags.len(), 1, "anonymous default class should be flagged");
    }

    #[test]
    fn test_allows_named_class() {
        let diags = lint("export default class Foo {}");
        assert!(
            diags.is_empty(),
            "named default class should not be flagged"
        );
    }

    #[test]
    fn test_flags_arrow_function() {
        let diags = lint("export default () => {}");
        assert_eq!(
            diags.len(),
            1,
            "arrow function default export should be flagged"
        );
    }

    #[test]
    fn test_flags_literal_expression() {
        let diags = lint("export default 42");
        assert_eq!(diags.len(), 1, "literal default export should be flagged");
    }

    #[test]
    fn test_allows_identifier_reference() {
        let diags = lint("const foo = 42; export default foo;");
        assert!(
            diags.is_empty(),
            "identifier reference default export should not be flagged"
        );
    }
}
