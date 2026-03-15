//! Rule: `jsx-a11y/scope`
//!
//! Enforce `scope` attribute is only used on `<th>` elements.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/scope";

#[derive(Debug)]
pub struct Scope;

impl LintRule for Scope {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `scope` attribute is only used on `<th>` elements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        let element_name = opening.name.as_str();

        // scope is valid on <th>
        if element_name == "th" {
            return;
        }

        let scope_span = opening.attributes.iter().find_map(|attr_id| {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                (attr.name.as_str() == "scope").then(|| Span::new(attr.span.start, attr.span.end))
            } else {
                None
            }
        });

        if let Some(attr_span) = scope_span {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "The `scope` attribute is only valid on `<th>` elements".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove `scope` attribute".to_owned(),
                    edits: vec![Edit {
                        span: attr_span,
                        replacement: String::new(),
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

    use super::*;

    starlint_rule_framework::lint_rule_test!(Scope);

    #[test]
    fn test_flags_scope_on_td() {
        let diags = lint(r#"const el = <td scope="col">header</td>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_scope_on_th() {
        let diags = lint(r#"const el = <th scope="col">header</th>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_td_without_scope() {
        let diags = lint(r"const el = <td>data</td>;");
        assert!(diags.is_empty());
    }
}
