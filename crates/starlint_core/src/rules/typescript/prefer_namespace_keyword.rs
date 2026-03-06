//! Rule: `typescript/prefer-namespace-keyword`
//!
//! Prefer the `namespace` keyword over `module` for `TypeScript` module
//! declarations. The `module` keyword is ambiguous — it can mean either a
//! namespace or an ambient module declaration. Using `namespace` makes the
//! intent explicit and avoids confusion with ES modules.

use oxc_ast::AstKind;
use oxc_ast::ast::{TSModuleDeclarationKind, TSModuleDeclarationName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
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
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSModuleDeclaration])
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

        // Find the `module` keyword in the source text within the declaration span
        let decl_start = usize::try_from(decl.span.start).unwrap_or(0);
        let decl_end = usize::try_from(decl.span.end).unwrap_or(0);
        let decl_text = ctx.source_text().get(decl_start..decl_end).unwrap_or("");

        if let Some(module_offset) = decl_text.find("module") {
            let module_start = u32::try_from(decl_start.saturating_add(module_offset)).unwrap_or(0);
            let module_end = module_start.saturating_add(6); // "module".len() == 6

            ctx.report(Diagnostic {
                rule_name: "typescript/prefer-namespace-keyword".to_owned(),
                message: "Use `namespace` instead of `module` to declare custom TypeScript modules"
                    .to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
                severity: Severity::Warning,
                help: Some("Replace `module` with `namespace`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace `module` with `namespace`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(module_start, module_end),
                        replacement: "namespace".to_owned(),
                    }],
                    is_snippet: false,
                }),
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
