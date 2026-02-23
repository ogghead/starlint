//! Rule: `react/jsx-boolean-value`
//!
//! Suggest omitting `={true}` for boolean JSX attributes.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-boolean-value";

/// Suggests omitting `={true}` from boolean JSX attributes since
/// `<Comp disabled />` is equivalent to `<Comp disabled={true} />`.
#[derive(Debug)]
pub struct JsxBooleanValue;

impl LintRule for JsxBooleanValue {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce omitting `={true}` for boolean JSX attributes".to_owned(),
            category: Category::Suggestion,
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

        // Check if the value is a JSXExpressionContainer wrapping a BooleanLiteral(true)
        let Some(value_id) = attr.value else {
            return;
        };

        let Some(AstNode::JSXExpressionContainer(container)) = ctx.node(value_id) else {
            return;
        };

        let Some(expr_id) = container.expression else {
            return;
        };

        if let Some(AstNode::BooleanLiteral(lit)) = ctx.node(expr_id) {
            if lit.value {
                let prop_name = attr.name.as_str();
                let attr_span = Span::new(attr.span.start, attr.span.end);
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Unnecessary `={{true}}` for boolean attribute `{prop_name}` — just use `{prop_name}`"
                    ),
                    span: attr_span,
                    severity: Severity::Warning,
                    help: Some(format!("Remove `={{true}}` — just use `{prop_name}`")),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Remove explicit `={true}`".to_owned(),
                        edits: vec![Edit {
                            span: attr_span,
                            replacement: prop_name.to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxBooleanValue)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_explicit_true() {
        let diags = lint("const el = <button disabled={true} />;");
        assert_eq!(diags.len(), 1, "should flag explicit true value");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_shorthand() {
        let diags = lint("const el = <button disabled />;");
        assert!(diags.is_empty(), "should not flag shorthand boolean");
    }

    #[test]
    fn test_allows_explicit_false() {
        let diags = lint("const el = <button disabled={false} />;");
        assert!(
            diags.is_empty(),
            "should not flag explicit false (it's necessary)"
        );
    }
}
