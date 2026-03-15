//! Rule: `typescript/prefer-namespace-keyword`
//!
//! Prefer the `namespace` keyword over `module` for `TypeScript` module
//! declarations. The `module` keyword is ambiguous — it can mean either a
//! namespace or an ambient module declaration. Using `namespace` makes the
//! intent explicit and avoids confusion with ES modules.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::fix_utils::source_text_for_span;
use starlint_rule_framework::{FixBuilder, LintContext, LintRule};

/// Flags `module Foo {}` declarations that should use `namespace` instead.
#[derive(Debug)]
pub struct PreferNamespaceKeyword;

impl LintRule for PreferNamespaceKeyword {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-namespace-keyword".to_owned(),
            description: "Prefer `namespace` over `module` for TypeScript module declarations"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSModuleDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSModuleDeclaration(decl) = node else {
            return;
        };

        // TSModuleDeclarationNode has no `kind` field in starlint_ast.
        // Use source text to detect `module` vs `namespace` keyword.
        let decl_text =
            source_text_for_span(ctx.source_text(), Span::new(decl.span.start, decl.span.end))
                .unwrap_or("");

        // Only flag `module` keyword, not `namespace`.
        if !decl_text.starts_with("module") && !decl_text.starts_with("declare module") {
            return;
        }

        // Ambient module declarations with string literal names
        // (e.g. `declare module "express" {}`) are valid and should not be flagged.
        // Check if the id resolves to a string literal.
        if ctx
            .node(decl.id)
            .is_some_and(|n| matches!(n, AstNode::StringLiteral(_)))
        {
            return;
        }

        // Find the `module` keyword in the source text within the declaration span
        if let Some(module_offset) = decl_text.find("module") {
            let module_start = decl
                .span
                .start
                .saturating_add(u32::try_from(module_offset).unwrap_or(0));
            let module_end = module_start.saturating_add(6); // "module".len() == 6

            ctx.report(Diagnostic {
                rule_name: "typescript/prefer-namespace-keyword".to_owned(),
                message: "Use `namespace` instead of `module` to declare custom TypeScript modules"
                    .to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
                severity: Severity::Warning,
                help: Some("Replace `module` with `namespace`".to_owned()),
                fix: FixBuilder::new("Replace `module` with `namespace`", FixKind::SafeFix)
                    .replace(Span::new(module_start, module_end), "namespace")
                    .build(),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferNamespaceKeyword, "test.ts");

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
