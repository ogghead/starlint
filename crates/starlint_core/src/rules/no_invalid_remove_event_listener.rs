//! Rule: `no-invalid-remove-event-listener`
//!
//! Flag `removeEventListener` calls where the listener argument is an
//! inline function expression or arrow function. Inline functions create
//! a new reference each time, so they can never match a previously added
//! listener — making the `removeEventListener` call a no-op bug.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `removeEventListener` calls with inline function listeners.
#[derive(Debug)]
pub struct NoInvalidRemoveEventListener;

impl LintRule for NoInvalidRemoveEventListener {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-invalid-remove-event-listener".to_owned(),
            description: "Disallow inline function listeners in `removeEventListener` calls"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for `.removeEventListener(...)` or `removeEventListener(...)`
        if !is_remove_event_listener_call(call.callee, ctx) {
            return;
        }

        // The listener is the second argument
        let Some(&second_arg_id) = call.arguments.get(1) else {
            return;
        };

        // Flag if the listener is an inline function or arrow function
        if is_inline_function(second_arg_id, ctx) {
            ctx.report(Diagnostic {
                rule_name: "no-invalid-remove-event-listener".to_owned(),
                message: "Inline function passed to `removeEventListener` will never match a previously added listener".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if a call expression's callee is `removeEventListener` (either
/// as a member property or a direct identifier).
fn is_remove_event_listener_call(callee_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(callee_id) {
        Some(AstNode::StaticMemberExpression(member)) => {
            member.property.as_str() == "removeEventListener"
        }
        Some(AstNode::IdentifierReference(id)) => id.name.as_str() == "removeEventListener",
        _ => false,
    }
}

/// Check if a node is an inline function expression or arrow function.
fn is_inline_function(id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(
        ctx.node(id),
        Some(AstNode::Function(_) | AstNode::ArrowFunctionExpression(_))
    )
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoInvalidRemoveEventListener)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_arrow_function_listener() {
        let diags = lint("el.removeEventListener('click', () => {});");
        assert_eq!(diags.len(), 1, "inline arrow listener should be flagged");
    }

    #[test]
    fn test_flags_function_expression_listener() {
        let diags = lint("el.removeEventListener('click', function() {});");
        assert_eq!(
            diags.len(),
            1,
            "inline function expression listener should be flagged"
        );
    }

    #[test]
    fn test_flags_arrow_with_body() {
        let diags = lint("el.removeEventListener('click', (e) => { console.log(e); });");
        assert_eq!(diags.len(), 1, "inline arrow with body should be flagged");
    }

    #[test]
    fn test_allows_named_handler() {
        let diags = lint("el.removeEventListener('click', handler);");
        assert!(
            diags.is_empty(),
            "named handler reference should not be flagged"
        );
    }

    #[test]
    fn test_allows_method_reference() {
        let diags = lint("el.removeEventListener('click', this.handleClick);");
        assert!(diags.is_empty(), "method reference should not be flagged");
    }

    #[test]
    fn test_allows_add_event_listener_inline() {
        let diags = lint("el.addEventListener('click', () => {});");
        assert!(
            diags.is_empty(),
            "addEventListener with inline function should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_listener_arg() {
        let diags = lint("el.removeEventListener('click');");
        assert!(
            diags.is_empty(),
            "removeEventListener with only one arg should not be flagged"
        );
    }
}
