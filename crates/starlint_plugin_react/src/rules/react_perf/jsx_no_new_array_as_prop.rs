//! Rule: `react-perf/jsx-no-new-array-as-prop`
//!
//! Warn when array literals are passed as JSX props, causing unnecessary re-renders.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react-perf/jsx-no-new-array-as-prop";

/// Warns when array literals (`[]`) are passed directly as JSX prop values.
///
/// Array literals create a new reference on every render, preventing
/// `React.memo` and `shouldComponentUpdate` from bailing out.
#[derive(Debug)]
pub struct JsxNoNewArrayAsProp;

impl LintRule for JsxNoNewArrayAsProp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prevent array literals from being passed as JSX props".to_owned(),
            category: Category::Performance,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXAttribute])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXAttribute(attr) = node else {
            return;
        };

        // attr.value is Option<NodeId>. Resolve it to check if it's an expression
        // container containing an array expression.
        let Some(val_id) = attr.value else {
            return;
        };
        let Some(AstNode::JSXExpressionContainer(container)) = ctx.node(val_id) else {
            return;
        };
        let container_span = container.span;
        let is_array = container
            .expression
            .and_then(|e| ctx.node(e))
            .is_some_and(|n| matches!(n, AstNode::ArrayExpression(_)));

        if is_array {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not pass an array literal as a JSX prop — it creates a new reference on every render".to_owned(),
                span: Span::new(container_span.start, container_span.end),
                severity: Severity::Warning,
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

    starlint_rule_framework::lint_rule_test!(JsxNoNewArrayAsProp);

    #[test]
    fn test_flags_array_literal_prop() {
        let diags = lint("const el = <Foo items={[1, 2, 3]} />;");
        assert_eq!(diags.len(), 1, "should flag inline array literal prop");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_variable_reference() {
        let diags = lint("const items = [1, 2];\nconst el = <Foo items={items} />;");
        assert!(
            diags.is_empty(),
            "should not flag variable reference as prop"
        );
    }

    #[test]
    fn test_flags_empty_array_prop() {
        let diags = lint("const el = <Foo items={[]} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag even an empty inline array literal"
        );
    }
}
