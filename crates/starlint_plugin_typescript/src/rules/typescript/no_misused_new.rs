//! Rule: `typescript/no-misused-new`
//!
//! Disallow `new` in an interface and `constructor` in a type alias.
//! Interfaces should not declare construct signatures (`new(): T`) because
//! they cannot be instantiated with `new` — use a class instead. Similarly,
//! type aliases wrapping object literal types should not contain construct
//! signatures, as this is almost always a mistake.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-misused-new";

/// Flags `new()` construct signatures in interfaces and type aliases.
#[derive(Debug)]
pub struct NoMisusedNew;

impl LintRule for NoMisusedNew {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `new` in an interface and `constructor` in a type alias"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::TSInterfaceDeclaration,
            AstNodeType::TSTypeAliasDeclaration,
        ])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::TSInterfaceDeclaration(decl) => {
                // Check body members for construct signatures.
                // Since starlint_ast does not have a TSConstructSignatureDeclaration variant,
                // we use source text to detect `new(` patterns in interface body members.
                // Collect spans of construct signatures, then report.
                let construct_spans: Vec<(u32, u32)> = {
                    let source = ctx.source_text();
                    decl.body
                        .iter()
                        .filter_map(|member_id| {
                            ctx.node(*member_id).map(starlint_ast::AstNode::span)
                        })
                        .filter(|sp| {
                            let text = source.get(sp.start as usize..sp.end as usize).unwrap_or("");
                            text.trim_start().starts_with("new") && text.contains('(')
                        })
                        .map(|sp| (sp.start, sp.end))
                        .collect()
                };

                for (start, end) in construct_spans {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Interfaces cannot be constructed — use a class instead of `new()` in an interface".to_owned(),
                        span: Span::new(start, end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::TSTypeAliasDeclaration(decl) => {
                // Check if the aliased type is a type literal with construct signatures.
                let Some(type_ann_id) = decl.type_annotation else {
                    return;
                };
                let Some(AstNode::TSTypeLiteral(type_lit)) = ctx.node(type_ann_id) else {
                    return;
                };
                let construct_spans: Vec<(u32, u32)> = {
                    let source = ctx.source_text();
                    type_lit
                        .members
                        .iter()
                        .filter_map(|member_id| {
                            ctx.node(*member_id).map(starlint_ast::AstNode::span)
                        })
                        .filter(|sp| {
                            let text = source.get(sp.start as usize..sp.end as usize).unwrap_or("");
                            text.trim_start().starts_with("new") && text.contains('(')
                        })
                        .map(|sp| (sp.start, sp.end))
                        .collect()
                };
                for (start, end) in construct_spans {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Do not use `new()` in a type alias — use a class instead"
                            .to_owned(),
                        span: Span::new(start, end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoMisusedNew)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_new_in_interface() {
        let diags = lint("interface I { new(): I }");
        assert_eq!(
            diags.len(),
            1,
            "construct signature in interface should be flagged"
        );
    }

    #[test]
    fn test_flags_new_in_type_alias() {
        let diags = lint("type T = { new(): T }");
        assert_eq!(
            diags.len(),
            1,
            "construct signature in type alias should be flagged"
        );
    }

    #[test]
    fn test_allows_interface_with_methods() {
        let diags = lint("interface I { foo(): void }");
        assert!(
            diags.is_empty(),
            "interface with regular methods should not be flagged"
        );
    }

    #[test]
    fn test_allows_class_with_constructor() {
        let diags = lint("class C { constructor() {} }");
        assert!(
            diags.is_empty(),
            "class with constructor should not be flagged"
        );
    }

    #[test]
    fn test_allows_interface_with_properties() {
        let diags = lint("interface I { x: number; y: string }");
        assert!(
            diags.is_empty(),
            "interface with properties should not be flagged"
        );
    }
}
