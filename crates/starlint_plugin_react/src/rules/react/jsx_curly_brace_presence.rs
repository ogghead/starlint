//! Rule: `react/jsx-curly-brace-presence`
//!
//! Suggest removing unnecessary curly braces around string literals in JSX props.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-curly-brace-presence";

/// Flags JSX expression containers that wrap a plain string literal, which is
/// unnecessary since JSX supports string attribute values directly.
///
/// For example: `<Comp prop={"text"} />` should be `<Comp prop="text" />`.
#[derive(Debug)]
pub struct JsxCurlyBracePresence;

impl LintRule for JsxCurlyBracePresence {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unnecessary curly braces around string literals in JSX props"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXExpressionContainer])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXExpressionContainer(container) = node else {
            return;
        };

        let Some(expr_id) = container.expression else {
            return;
        };
        if let Some(AstNode::StringLiteral(lit)) = ctx.node(expr_id) {
            let container_span = Span::new(container.span.start, container.span.end);
            // Build replacement: the string literal value wrapped in double quotes
            let value = lit.value.as_str();
            let replacement = format!("\"{value}\"");

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Unnecessary curly braces around string literal. Use a plain string attribute value instead".to_owned(),
                span: container_span,
                severity: Severity::Warning,
                help: Some("Remove the curly braces and use a plain string".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove unnecessary curly braces".to_owned(),
                    edits: vec![Edit {
                        span: container_span,
                        replacement,
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

    starlint_rule_framework::lint_rule_test!(JsxCurlyBracePresence);

    #[test]
    fn test_flags_string_in_curly_braces() {
        let diags = lint(r#"const el = <div className={"foo"} />;"#);
        assert_eq!(diags.len(), 1, "should flag string literal in curly braces");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_plain_string() {
        let diags = lint(r#"const el = <div className="foo" />;"#);
        assert!(diags.is_empty(), "should not flag plain string attribute");
    }

    #[test]
    fn test_allows_expression() {
        let diags = lint("const el = <div className={styles.foo} />;");
        assert!(
            diags.is_empty(),
            "should not flag non-string expressions in braces"
        );
    }
}
