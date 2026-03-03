//! Rule: `typescript/no-namespace`
//!
//! Disallow `TypeScript` `namespace` and `module` declarations. Namespaces are
//! a legacy `TypeScript` feature that predates ES modules. Modern code should
//! use standard ES module imports/exports instead.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `namespace` and `module` declarations.
#[derive(Debug)]
pub struct NoNamespace;

impl NativeRule for NoNamespace {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-namespace".to_owned(),
            description: "Disallow TypeScript `namespace` and `module` declarations".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSModuleDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSModuleDeclaration(decl) = kind else {
            return;
        };

        ctx.report_warning(
            "typescript/no-namespace",
            "Do not use TypeScript namespaces — use ES modules instead",
            Span::new(decl.span.start, decl.span.end),
        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNamespace)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_namespace() {
        let diags = lint("namespace Foo { }");
        assert_eq!(diags.len(), 1, "`namespace` declaration should be flagged");
    }

    #[test]
    fn test_flags_module() {
        let diags = lint("module Foo { }");
        assert_eq!(diags.len(), 1, "`module` declaration should be flagged");
    }

    #[test]
    fn test_allows_regular_code() {
        let diags = lint("const x = 1;");
        assert!(diags.is_empty(), "regular code should not be flagged");
    }
}
