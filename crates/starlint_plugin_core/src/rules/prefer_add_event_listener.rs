//! Rule: `prefer-add-event-listener`
//!
//! Prefer `addEventListener` over assigning to `on*` event-handler
//! properties. Using `addEventListener` allows multiple handlers and
//! provides more control over event handling.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `on*` event-handler property assignments.
#[derive(Debug)]
pub struct PreferAddEventListener;

impl LintRule for PreferAddEventListener {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-add-event-listener".to_owned(),
            description: "Prefer `addEventListener` over `on*` property assignment".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AssignmentExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::AssignmentExpression(assign) = node else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(assign.left) else {
            return;
        };

        let prop_name = member.property.as_str();
        let obj_id = member.object;
        let rhs_id = assign.right;

        if is_event_handler_property(prop_name) {
            // Extract event name: "onclick" -> "click"
            let event_name = prop_name.strip_prefix("on").unwrap_or(prop_name);

            // Build fix: el.onclick = handler -> el.addEventListener('click', handler)
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let obj_span = ctx
                    .node(obj_id)
                    .map_or(starlint_ast::types::Span::new(0, 0), AstNode::span);
                let rhs_span = ctx
                    .node(rhs_id)
                    .map_or(starlint_ast::types::Span::new(0, 0), AstNode::span);
                let obj_text = source
                    .get(obj_span.start as usize..obj_span.end as usize)
                    .unwrap_or("");
                let rhs_text = source
                    .get(rhs_span.start as usize..rhs_span.end as usize)
                    .unwrap_or("");
                (!obj_text.is_empty() && !rhs_text.is_empty()).then(|| Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!(
                        "Replace with `{obj_text}.addEventListener('{event_name}', {rhs_text})`"
                    ),
                    edits: vec![Edit {
                        span: Span::new(assign.span.start, assign.span.end),
                        replacement: format!(
                            "{obj_text}.addEventListener('{event_name}', {rhs_text})"
                        ),
                    }],
                    is_snippet: false,
                })
            };

            ctx.report(Diagnostic {
                rule_name: "prefer-add-event-listener".to_owned(),
                message: format!("Prefer `addEventListener` over assigning to `.{prop_name}`"),
                span: Span::new(assign.span.start, assign.span.end),
                severity: Severity::Warning,
                help: Some(format!(
                    "Use `addEventListener('{event_name}', handler)` instead"
                )),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if a property name matches the `on<event>` pattern.
///
/// The property must start with `on` followed by a lowercase ASCII letter
/// (e.g. `onclick`, `onload`, `onchange`). Properties like `onFoo` with
/// an uppercase letter after `on` are not considered standard DOM events.
fn is_event_handler_property(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("on") else {
        return false;
    };

    rest.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferAddEventListener);

    #[test]
    fn test_flags_onclick_assignment() {
        let diags = lint("el.onclick = handler;");
        assert_eq!(diags.len(), 1, "el.onclick assignment should be flagged");
    }

    #[test]
    fn test_flags_window_onload() {
        let diags = lint("window.onload = init;");
        assert_eq!(diags.len(), 1, "window.onload assignment should be flagged");
    }

    #[test]
    fn test_flags_onchange() {
        let diags = lint("input.onchange = validate;");
        assert_eq!(
            diags.len(),
            1,
            "input.onchange assignment should be flagged"
        );
    }

    #[test]
    fn test_allows_add_event_listener() {
        let diags = lint("el.addEventListener('click', handler);");
        assert!(diags.is_empty(), "addEventListener should not be flagged");
    }

    #[test]
    fn test_allows_uppercase_after_on() {
        let diags = lint("el.onFoo = bar;");
        assert!(
            diags.is_empty(),
            "onFoo with uppercase F is not a standard event handler"
        );
    }

    #[test]
    fn test_allows_non_on_property() {
        let diags = lint("el.value = 'test';");
        assert!(
            diags.is_empty(),
            "non-on property assignment should not be flagged"
        );
    }

    #[test]
    fn test_allows_identifier_assignment() {
        let diags = lint("onclick = handler;");
        assert!(
            diags.is_empty(),
            "bare identifier assignment should not be flagged"
        );
    }
}
