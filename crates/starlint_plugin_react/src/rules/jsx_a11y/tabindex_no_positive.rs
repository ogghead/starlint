//! Rule: `jsx-a11y/tabindex-no-positive`
//!
//! Forbid positive `tabIndex` values.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/tabindex-no-positive";

#[derive(Debug)]
pub struct TabindexNoPositive;

impl LintRule for TabindexNoPositive {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid positive `tabIndex` values".to_owned(),
            category: Category::Suggestion,
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

        for &attr_id in &*opening.attributes {
            let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) else {
                continue;
            };

            if attr.name.as_str() != "tabIndex" {
                continue;
            }

            let Some(value_id) = attr.value else {
                continue;
            };
            if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
                let val = lit.value.as_str();
                if let Ok(n) = val.parse::<i32>() {
                    if n > 0 {
                        ctx.report(Diagnostic {
                            rule_name: RULE_NAME.to_owned(),
                            message: "Avoid positive `tabIndex` values. They disrupt the natural tab order".to_owned(),
                            span: Span::new(opening.span.start, opening.span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: Some(Fix {
                                kind: FixKind::SuggestionFix,
                                message: "Replace with `tabIndex=\"0\"`".to_owned(),
                                edits: vec![Edit {
                                    span: Span::new(lit.span.start, lit.span.end),
                                    replacement: "\"0\"".to_owned(),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(TabindexNoPositive);

    #[test]
    fn test_flags_positive_tabindex() {
        let diags = lint(r#"const el = <div tabIndex="5">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_zero_tabindex() {
        let diags = lint(r#"const el = <div tabIndex="0">content</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_negative_tabindex() {
        let diags = lint(r#"const el = <div tabIndex="-1">content</div>;"#);
        assert!(diags.is_empty());
    }
}
