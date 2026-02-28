//! Rule: `typescript/prefer-namespace-keyword`
//!
//! Prefer the `namespace` keyword over `module` for `TypeScript` module
//! declarations. The `module` keyword is ambiguous — it can mean either a
//! namespace or an ambient module declaration. Using `namespace` makes the
//! intent explicit and avoids confusion with ES modules.

use oxc_ast::AstKind;
use oxc_ast::ast::{TSModuleDeclarationKind, TSModuleDeclarationName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `module Foo {}` declarations that should use `namespace` instead.
#[derive(Debug)]
pub struct PreferNamespaceKeyword;

impl NativeRule for PreferNamespaceKeyword {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-namespace-keyword".to_owned(),
            description: "Prefer `namespace` over `module` for TypeScript module declarations"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSModuleDeclaration(decl) = kind else {
            return;
        };

        // Only flag `module` keyword, not `namespace`.
        if decl.kind != TSModuleDeclarationKind::Module {
            return;
        }

        // Ambient module declarations with string literal names
        // (e.g. `declare module "express" {}`) are valid and should not be flagged.
        if matches!(&decl.id, TSModuleDeclarationName::StringLiteral(_)) {
            return;
        }

        ctx.report_warning(
            "typescript/prefer-namespace-keyword",
            "Use `namespace` instead of `module` to declare custom TypeScript modules",
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

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferNamespaceKeyword)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_module_with_identifier() {
        let diags = lint("module Foo { }");
        assert_eq!(diags.len(), 1, "module Foo should be flagged");
    }

    #[test]
    fn test_allows_namespace() {
        let diags = lint("namespace Foo { }");
        assert!(diags.is_empty(), "namespace Foo should not be flagged");
    }

    #[test]
    fn test_allows_ambient_module_with_string_literal() {
        let diags = lint("declare module \"express\" { }");
        assert!(
            diags.is_empty(),
            "declare module with string literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_code() {
        let diags = lint("const x = 1;");
        assert!(diags.is_empty(), "regular code should not be flagged");
    }
}
