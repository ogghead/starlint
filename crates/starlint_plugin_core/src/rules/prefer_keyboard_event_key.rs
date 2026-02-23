//! Rule: `prefer-keyboard-event-key`
//!
//! Prefer `KeyboardEvent.key` over the deprecated `keyCode`, `charCode`, and
//! `which` properties. The `key` property provides a human-readable string
//! and is supported in all modern browsers.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Deprecated `KeyboardEvent` properties that should be replaced with `key`.
const DEPRECATED_PROPERTIES: &[&str] = &["keyCode", "charCode", "which"];

/// Flags access to deprecated `KeyboardEvent` properties.
#[derive(Debug)]
pub struct PreferKeyboardEventKey;

impl LintRule for PreferKeyboardEventKey {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-keyboard-event-key".to_owned(),
            description:
                "Prefer `KeyboardEvent.key` over deprecated `keyCode`, `charCode`, and `which`"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StaticMemberExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StaticMemberExpression(member) = node else {
            return;
        };

        let prop = member.property.as_str();
        if !DEPRECATED_PROPERTIES.contains(&prop) {
            return;
        }

        // property is a String, compute span from the end of the member span
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        let prop_start = member.span.end.saturating_sub(member.property.len() as u32);
        let prop_span = Span::new(prop_start, member.span.end);
        ctx.report(Diagnostic {
            rule_name: "prefer-keyboard-event-key".to_owned(),
            message: format!("Use `KeyboardEvent.key` instead of deprecated `{prop}`"),
            span: Span::new(member.span.start, member.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace `{prop}` with `key`")),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: format!("Replace `{prop}` with `key`"),
                edits: vec![Edit {
                    span: prop_span,
                    replacement: "key".to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferKeyboardEventKey)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_key_code() {
        let diags = lint("var code = event.keyCode;");
        assert_eq!(diags.len(), 1, "event.keyCode should be flagged");
    }

    #[test]
    fn test_flags_char_code() {
        let diags = lint("var code = e.charCode;");
        assert_eq!(diags.len(), 1, "e.charCode should be flagged");
    }

    #[test]
    fn test_flags_which() {
        let diags = lint("var code = e.which;");
        assert_eq!(diags.len(), 1, "e.which should be flagged");
    }

    #[test]
    fn test_flags_any_object() {
        let diags = lint("var code = obj.keyCode;");
        assert_eq!(diags.len(), 1, "obj.keyCode should be flagged");
    }

    #[test]
    fn test_allows_key() {
        let diags = lint("var k = event.key;");
        assert!(diags.is_empty(), "event.key should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_property() {
        let diags = lint("var v = event.target;");
        assert!(diags.is_empty(), "event.target should not be flagged");
    }
}
