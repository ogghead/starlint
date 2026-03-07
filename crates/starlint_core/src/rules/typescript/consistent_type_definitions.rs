//! Rule: `typescript/consistent-type-definitions`
//!
//! Prefer `interface` over `type` for object type definitions. Interfaces are
//! more performant for the `TypeScript` compiler, support declaration merging,
//! and produce clearer error messages. When a `type` alias wraps an object
//! literal type (e.g. `type Foo = { x: number }`), it can almost always be
//! rewritten as an `interface`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `type` aliases that define object literal types.
#[derive(Debug)]
pub struct ConsistentTypeDefinitions;

impl LintRule for ConsistentTypeDefinitions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/consistent-type-definitions".to_owned(),
            description: "Prefer `interface` over `type` for object type definitions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSTypeAliasDeclaration])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    #[allow(clippy::arithmetic_side_effects)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSTypeAliasDeclaration(decl) = node else {
            return;
        };

        // Only flag when the type body is a plain object literal type `{ ... }`.
        // TSTypeAliasDeclarationNode has no type_annotation field, so we use a
        // source text heuristic: check if the text after `=` starts with `{`.
        #[allow(clippy::as_conversions)]
        let is_object_type = {
            let source = ctx.source_text();
            let decl_text = source
                .get(decl.span.start as usize..decl.span.end as usize)
                .unwrap_or("");
            decl_text
                .find('=')
                .is_some_and(|pos| decl_text[pos + 1..].trim_start().starts_with('{'))
        };
        if !is_object_type {
            return;
        }

        // Fix: `type Foo = { ... }` → `interface Foo { ... }`
        // Replace full declaration span with rewritten text
        let source = ctx.source_text();
        let decl_text = &source[decl.span.start as usize..decl.span.end as usize];
        // Replace "type " with "interface " and remove " = "
        let replacement = decl_text
            .replacen("type ", "interface ", 1)
            .replacen(" = ", " ", 1);

        ctx.report(Diagnostic {
            rule_name: "typescript/consistent-type-definitions".to_owned(),
            message: "Use an `interface` instead of a `type` alias for this object type".to_owned(),
            span: Span::new(decl.span.start, decl.span.end),
            severity: Severity::Warning,
            help: Some("Replace `type` with `interface`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace `type` with `interface`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(decl.span.start, decl.span.end),
                    replacement,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ConsistentTypeDefinitions)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_type_with_object_literal() {
        let diags = lint("type Foo = { x: number };");
        assert_eq!(
            diags.len(),
            1,
            "type alias with object literal type should be flagged"
        );
    }

    #[test]
    fn test_allows_interface() {
        let diags = lint("interface Foo { x: number }");
        assert!(
            diags.is_empty(),
            "`interface` declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_type_alias_to_primitive() {
        let diags = lint("type Foo = string;");
        assert!(
            diags.is_empty(),
            "`type` alias to primitive should not be flagged"
        );
    }

    #[test]
    fn test_allows_union_type() {
        let diags = lint("type Foo = string | number;");
        assert!(
            diags.is_empty(),
            "`type` alias for union should not be flagged"
        );
    }

    #[test]
    fn test_allows_intersection_type() {
        let diags = lint("type Foo = Bar & Baz;");
        assert!(
            diags.is_empty(),
            "`type` alias for intersection should not be flagged"
        );
    }
}
