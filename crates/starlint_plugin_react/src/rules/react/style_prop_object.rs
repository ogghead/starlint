//! Rule: `react/style-prop-object`
//!
//! The `style` prop should be an object. Passing a string as the `style` prop
//! in JSX is a common mistake when migrating from HTML -- React requires style
//! to be a JavaScript object.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `style` props with string literal values.
#[derive(Debug)]
pub struct StylePropObject;

impl LintRule for StylePropObject {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/style-prop-object".to_owned(),
            description: "The `style` prop should be an object, not a string".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXAttribute])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXAttribute(attr) = node else {
            return;
        };

        if attr.name.as_str() != "style" {
            return;
        }

        // Check if the value is a string literal
        let is_string_value = attr
            .value
            .and_then(|val_id| ctx.node(val_id))
            .is_some_and(|n| matches!(n, AstNode::StringLiteral(_)));
        if is_string_value {
            ctx.report(Diagnostic {
                rule_name: "react/style-prop-object".to_owned(),
                message: "The `style` prop expects an object, not a string".to_owned(),
                span: Span::new(attr.span.start, attr.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(StylePropObject);

    #[test]
    fn test_flags_style_string() {
        let source = r#"var x = <div style="color: red" />;"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "string style prop should be flagged");
    }

    #[test]
    fn test_allows_style_object() {
        let source = "var x = <div style={{ color: 'red' }} />;";
        let diags = lint(source);
        assert!(diags.is_empty(), "object style prop should not be flagged");
    }

    #[test]
    fn test_allows_other_string_props() {
        let source = r#"var x = <div className="foo" />;"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "other string props should not be flagged");
    }
}
