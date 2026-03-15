//! Rule: `react/jsx-no-script-url`
//!
//! Error when JSX attributes contain `javascript:` URLs.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-script-url";

/// Flags JSX attributes that contain `javascript:` URLs, which are a security
/// risk (XSS vector).
#[derive(Debug)]
pub struct JsxNoScriptUrl;

/// Check if a string value starts with `javascript:` (case-insensitive).
fn is_script_url(value: &str) -> bool {
    value.trim().to_ascii_lowercase().starts_with("javascript:")
}

impl LintRule for JsxNoScriptUrl {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `javascript:` URLs in JSX attributes".to_owned(),
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

        let Some(value_id) = attr.value else {
            return;
        };

        let has_script_url = match ctx.node(value_id) {
            Some(AstNode::StringLiteral(lit)) => is_script_url(lit.value.as_str()),
            Some(AstNode::JSXExpressionContainer(container)) => container
                .expression
                .and_then(|expr_id| ctx.node(expr_id))
                .is_some_and(|expr_node| {
                    if let AstNode::StringLiteral(lit) = expr_node {
                        is_script_url(lit.value.as_str())
                    } else {
                        false
                    }
                }),
            _ => false,
        };

        if has_script_url {
            let attr_span = Span::new(attr.span.start, attr.span.end);
            let fix = FixBuilder::new("Remove `javascript:` URL attribute", FixKind::SuggestionFix)
                .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Disallow `javascript:` URLs — they are a security risk".to_owned(),
                span: attr_span,
                severity: Severity::Error,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(JsxNoScriptUrl);

    #[test]
    fn test_flags_javascript_url_string() {
        let diags = lint(r#"const el = <a href="javascript:alert('xss')">link</a>;"#);
        assert_eq!(diags.len(), 1, "should flag javascript: URL");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_flags_javascript_url_expression() {
        let diags = lint(r#"const el = <a href={"javascript:void(0)"}>link</a>;"#);
        assert_eq!(
            diags.len(),
            1,
            "should flag javascript: URL in expression container"
        );
    }

    #[test]
    fn test_allows_normal_url() {
        let diags = lint(r#"const el = <a href="https://example.com">link</a>;"#);
        assert!(diags.is_empty(), "should not flag normal URLs");
    }
}
