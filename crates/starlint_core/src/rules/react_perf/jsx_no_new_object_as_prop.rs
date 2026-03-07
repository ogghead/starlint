//! Rule: `react-perf/jsx-no-new-object-as-prop`
//!
//! Warn when object literals are passed as JSX props, causing unnecessary re-renders.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "react-perf/jsx-no-new-object-as-prop";

/// Warns when object literals (`{}`) are passed directly as JSX prop values.
///
/// Object literals create a new reference on every render, preventing
/// `React.memo` and `shouldComponentUpdate` from bailing out.
#[derive(Debug)]
pub struct JsxNoNewObjectAsProp;

impl LintRule for JsxNoNewObjectAsProp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prevent object literals from being passed as JSX props".to_owned(),
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

        let Some(val_id) = attr.value else {
            return;
        };
        let Some(AstNode::JSXExpressionContainer(container)) = ctx.node(val_id) else {
            return;
        };
        let container_span = container.span;
        let is_object = container
            .expression
            .and_then(|e| ctx.node(e))
            .is_some_and(|n| matches!(n, AstNode::ObjectExpression(_)));

        if is_object {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not pass an object literal as a JSX prop — it creates a new reference on every render".to_owned(),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxNoNewObjectAsProp)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_object_literal_prop() {
        let diags = lint(r#"const el = <Foo style={{ color: "red" }} />;"#);
        assert_eq!(diags.len(), 1, "should flag inline object literal prop");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_variable_reference() {
        let diags = lint("const style = {};\nconst el = <Foo style={style} />;");
        assert!(
            diags.is_empty(),
            "should not flag variable reference as prop"
        );
    }

    #[test]
    fn test_flags_multiple_object_props() {
        let diags = lint(r#"const el = <Foo style={{ color: "red" }} data={{ id: 1 }} />;"#);
        assert_eq!(
            diags.len(),
            2,
            "should flag each inline object literal prop"
        );
    }
}
