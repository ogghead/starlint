//! Rule: `typescript/no-unnecessary-type-constraint`
//!
//! Disallow unnecessary constraints on generic type parameters. When a type
//! parameter extends `any` or `unknown`, the constraint is redundant because
//! these are already the implicit defaults for unconstrained type parameters.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags type parameters with unnecessary `extends any` or `extends unknown` constraints.
#[derive(Debug)]
pub struct NoUnnecessaryTypeConstraint;

impl LintRule for NoUnnecessaryTypeConstraint {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unnecessary-type-constraint".to_owned(),
            description: "Disallow unnecessary constraints on generic type parameters".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSTypeParameter])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSTypeParameter(param) = node else {
            return;
        };

        let Some(constraint_id) = param.constraint else {
            return;
        };

        let Some(constraint_node) = ctx.node(constraint_id) else {
            return;
        };

        let constraint_name = if let AstNode::TSAnyKeyword(_) = constraint_node {
            "any"
        } else {
            // Check source text for "unknown" keyword
            let span = constraint_node.span();
            let start = usize::try_from(span.start).unwrap_or(0);
            let end = usize::try_from(span.end).unwrap_or(0);
            let text = ctx.source_text().get(start..end).unwrap_or("");
            if text.trim() == "unknown" {
                "unknown"
            } else {
                return;
            }
        };

        let constraint_span = constraint_node.span();

        // Delete from the end of the type parameter name to the end of the constraint type.
        // This removes ` extends any` / ` extends unknown`.
        // The name is a String in TSTypeParameterNode, so find it in source text.
        // Use the source text to find where the name ends.
        let source = ctx.source_text();
        let param_start = usize::try_from(param.span.start).unwrap_or(0);
        let param_text_region = source
            .get(param_start..usize::try_from(constraint_span.end).unwrap_or(0))
            .unwrap_or("");

        // Find where "extends" starts to determine delete_start
        let delete_start = if let Some(extends_pos) = param_text_region.find("extends") {
            // Delete from just after the name (before " extends")
            u32::try_from(param_start.saturating_add(extends_pos).saturating_sub(1))
                .unwrap_or(param.span.start)
        } else {
            // Fallback: delete from end of name
            param
                .span
                .start
                .saturating_add(u32::try_from(param.name.len()).unwrap_or(0))
        };

        let delete_end = constraint_span.end;

        ctx.report(Diagnostic {
            rule_name: "typescript/no-unnecessary-type-constraint".to_owned(),
            message: format!(
                "Unnecessary `extends {constraint_name}` constraint — type parameters default to `{constraint_name}` implicitly"
            ),
            span: Span::new(param.span.start, param.span.end),
            severity: Severity::Warning,
            help: Some(format!("Remove the `extends {constraint_name}` constraint")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Remove `extends {constraint_name}`"),
                edits: vec![Edit {
                    span: Span::new(delete_start, delete_end),
                    replacement: String::new(),
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

    starlint_rule_framework::lint_rule_test!(NoUnnecessaryTypeConstraint, "test.ts");

    #[test]
    fn test_flags_extends_any() {
        let diags = lint("function f<T extends any>() {}");
        assert_eq!(
            diags.len(),
            1,
            "`T extends any` should be flagged as unnecessary"
        );
    }

    #[test]
    fn test_flags_extends_unknown() {
        let diags = lint("function f<T extends unknown>() {}");
        assert_eq!(
            diags.len(),
            1,
            "`T extends unknown` should be flagged as unnecessary"
        );
    }

    #[test]
    fn test_allows_extends_string() {
        let diags = lint("function f<T extends string>() {}");
        assert!(diags.is_empty(), "`T extends string` should not be flagged");
    }

    #[test]
    fn test_allows_unconstrained() {
        let diags = lint("function f<T>() {}");
        assert!(
            diags.is_empty(),
            "unconstrained type parameter should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_params_one_bad() {
        let diags = lint("function f<T extends any, U extends string>() {}");
        assert_eq!(
            diags.len(),
            1,
            "only `T extends any` should be flagged, not `U extends string`"
        );
    }

    #[test]
    fn test_flags_type_alias() {
        let diags = lint("type Box<T extends any> = { value: T };");
        assert_eq!(
            diags.len(),
            1,
            "`extends any` on type alias should be flagged"
        );
    }
}
