//! Rule: `react-perf/jsx-no-jsx-as-prop`
//!
//! Warn when JSX elements are passed inline as props, causing unnecessary re-renders.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "react-perf/jsx-no-jsx-as-prop";

/// Warns when JSX elements are passed inline as prop values.
///
/// Inline JSX creates a new element on every render, preventing
/// `React.memo` and `shouldComponentUpdate` from bailing out.
/// Extract the JSX to a variable or memoize it with `useMemo` instead.
#[derive(Debug)]
pub struct JsxNoJsxAsProp;

impl LintRule for JsxNoJsxAsProp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prevent JSX elements from being passed inline as props".to_owned(),
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

        // Check ExpressionContainer values: `prop={<Child />}`
        let Some(value_id) = attr.value else {
            return;
        };

        let Some(value_node) = ctx.node(value_id) else {
            return;
        };

        if let AstNode::JSXExpressionContainer(container) = value_node {
            if let Some(expr_id) = container.expression {
                if matches!(
                    ctx.node(expr_id),
                    Some(AstNode::JSXElement(_) | AstNode::JSXFragment(_))
                ) {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message:
                            "Do not pass JSX as a prop value — it creates a new element on every render"
                                .to_owned(),
                        span: Span::new(container.span.start, container.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                    return;
                }
            }
        }

        // Check direct element values: `prop=<Child />`
        // (valid JSX syntax: `<Foo bar=<Baz /> />`)
        if matches!(value_node, AstNode::JSXElement(_) | AstNode::JSXFragment(_)) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Do not pass JSX as a prop value — it creates a new element on every render"
                        .to_owned(),
                span: Span::new(attr.span.start, attr.span.end),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxNoJsxAsProp)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_jsx_element_in_expression_container() {
        let diags = lint("const el = <Foo icon={<Icon />} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag inline JSX element passed as prop"
        );
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_flags_jsx_fragment_in_expression_container() {
        let diags = lint("const el = <Foo content={<>hello</>} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag inline JSX fragment passed as prop"
        );
    }

    #[test]
    fn test_allows_variable_reference() {
        let diags = lint("const icon = <Icon />;\nconst el = <Foo icon={icon} />;");
        assert!(
            diags.is_empty(),
            "should not flag variable reference as prop"
        );
    }
}
