//! Rule: `jsx-a11y/anchor-ambiguous-text`
//!
//! Forbid ambiguous link text like "click here" or "read more".

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::jsx_utils::get_jsx_attr_string_value;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/anchor-ambiguous-text";

/// Ambiguous phrases that should not be used as standalone link text.
const AMBIGUOUS_PHRASES: &[&str] = &[
    "click here",
    "here",
    "read more",
    "learn more",
    "more",
    "link",
];

#[derive(Debug)]
pub struct AnchorAmbiguousText;

/// Check if text is an ambiguous link phrase.
fn is_ambiguous(text: &str) -> bool {
    let normalized = text.trim().to_lowercase();
    AMBIGUOUS_PHRASES.iter().any(|phrase| normalized == *phrase)
}

impl LintRule for AnchorAmbiguousText {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid ambiguous link text like \"click here\" or \"read more\""
                .to_owned(),
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

        if opening.name.as_str() != "a" {
            return;
        }

        // Check aria-label for ambiguous text
        if let Some(label) = get_jsx_attr_string_value(&opening.attributes, "aria-label", ctx) {
            if is_ambiguous(&label) {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Ambiguous link text \"{label}\". Use text that describes the link destination"
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
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(AnchorAmbiguousText);

    #[test]
    fn test_flags_ambiguous_aria_label() {
        let diags = lint(r#"const el = <a href="/about" aria-label="click here">x</a>;"#);
        assert_eq!(diags.len(), 1, "should flag ambiguous aria-label");
    }

    #[test]
    fn test_allows_descriptive_aria_label() {
        let diags = lint(r#"const el = <a href="/about" aria-label="About our company">x</a>;"#);
        assert!(diags.is_empty(), "should allow descriptive aria-label");
    }

    #[test]
    fn test_allows_anchor_without_aria_label() {
        let diags = lint(r#"const el = <a href="/about">About</a>;"#);
        assert!(diags.is_empty(), "should allow anchor without aria-label");
    }
}
