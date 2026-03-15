//! Rule: `jsx-a11y/label-has-associated-control`
//!
//! Enforce `<label>` elements have an associated control via `htmlFor` or nesting.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::jsx_utils::has_jsx_attribute;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/label-has-associated-control";

#[derive(Debug)]
pub struct LabelHasAssociatedControl;

impl LintRule for LabelHasAssociatedControl {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Enforce `<label>` elements have an associated control via `htmlFor` or nesting"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXElement(element) = node else {
            return;
        };

        let Some(AstNode::JSXOpeningElement(opening)) = ctx.node(element.opening_element) else {
            return;
        };

        if opening.name.as_str() != "label" {
            return;
        }

        // Check for htmlFor attribute
        let has_html_for = has_jsx_attribute(&opening.attributes, "htmlFor", ctx);

        let opening_span_start = opening.span.start;
        let opening_span_end = opening.span.end;

        // If no children and no htmlFor, the label has no associated control
        if element.children.is_empty() && !has_html_for {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "`<label>` must have an associated control via `htmlFor` or by nesting an input"
                        .to_owned(),
                span: Span::new(opening_span_start, opening_span_end),
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

    starlint_rule_framework::lint_rule_test!(LabelHasAssociatedControl);

    #[test]
    fn test_flags_bare_label_without_htmlfor() {
        let diags = lint(r"const el = <label />;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_label_with_htmlfor() {
        let diags = lint(r#"const el = <label htmlFor="input-id">Name</label>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_label_with_children() {
        let diags = lint(r"const el = <label>Name <input /></label>;");
        assert!(diags.is_empty());
    }
}
