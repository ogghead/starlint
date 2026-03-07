//! Rule: `react-perf/jsx-no-new-function-as-prop`
//!
//! Warn when inline functions are passed as JSX props, causing unnecessary re-renders.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "react-perf/jsx-no-new-function-as-prop";

/// Warns when inline functions (arrow functions or function expressions) are
/// passed directly as JSX prop values.
///
/// Inline functions create a new closure on every render, preventing
/// `React.memo` and `shouldComponentUpdate` from bailing out.
/// Use `useCallback` or define the handler outside the render path instead.
#[derive(Debug)]
pub struct JsxNoNewFunctionAsProp;

impl LintRule for JsxNoNewFunctionAsProp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prevent inline functions from being passed as JSX props".to_owned(),
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

        let Some(value_id) = attr.value else {
            return;
        };

        let Some(AstNode::JSXExpressionContainer(container)) = ctx.node(value_id) else {
            return;
        };

        let is_inline_function = container
            .expression
            .and_then(|expr_id| ctx.node(expr_id))
            .is_some_and(|expr_node| {
                matches!(
                    expr_node,
                    AstNode::ArrowFunctionExpression(_) | AstNode::Function(_)
                )
            });

        if is_inline_function {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not pass an inline function as a JSX prop — it creates a new closure on every render".to_owned(),
                span: Span::new(container.span.start, container.span.end),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxNoNewFunctionAsProp)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_arrow_function_prop() {
        let diags = lint("const el = <Foo onClick={() => console.log('click')} />;");
        assert_eq!(diags.len(), 1, "should flag inline arrow function prop");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_flags_function_expression_prop() {
        let diags = lint("const el = <Foo onClick={function() { return 1; }} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag inline function expression prop"
        );
    }

    #[test]
    fn test_allows_variable_reference() {
        let diags = lint("const handler = () => {};\nconst el = <Foo onClick={handler} />;");
        assert!(
            diags.is_empty(),
            "should not flag variable reference as prop"
        );
    }
}
