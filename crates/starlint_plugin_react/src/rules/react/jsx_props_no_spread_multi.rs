//! Rule: `react/jsx-props-no-spread-multi`
//!
//! Warn when a JSX element has multiple spread attributes.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-props-no-spread-multi";

/// Flags JSX elements with more than one spread attribute. Multiple spreads
/// make prop resolution order confusing and error-prone.
#[derive(Debug)]
pub struct JsxPropsNoSpreadMulti;

impl LintRule for JsxPropsNoSpreadMulti {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow multiple spread attributes on a JSX element".to_owned(),
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

        let spread_count = opening
            .attributes
            .iter()
            .filter(|&&attr_id| matches!(ctx.node(attr_id), Some(AstNode::JSXSpreadAttribute(_))))
            .count();

        if spread_count > 1 {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "JSX element has {spread_count} spread attributes — use at most one to avoid confusing prop resolution order"
                ),
                span: Span::new(opening.span.start, opening.span.end),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxPropsNoSpreadMulti)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_multiple_spreads() {
        let diags = lint("const el = <div {...a} {...b} />;");
        assert_eq!(diags.len(), 1, "should flag element with multiple spreads");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_single_spread() {
        let diags = lint("const el = <div {...props} />;");
        assert!(diags.is_empty(), "should not flag element with one spread");
    }

    #[test]
    fn test_allows_no_spreads() {
        let diags = lint(r#"const el = <div className="foo" />;"#);
        assert!(diags.is_empty(), "should not flag element with no spreads");
    }
}
