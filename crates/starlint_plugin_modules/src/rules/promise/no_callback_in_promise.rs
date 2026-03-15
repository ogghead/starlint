//! Rule: `promise/no-callback-in-promise`
//!
//! Forbid calling callbacks (e.g. `cb`, `callback`, `done`, `next`)
//! inside `.then()` or `.catch()`. Mixing callbacks with promises leads
//! to confusing control flow and potential double-resolution bugs.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Common callback parameter names.
const CALLBACK_NAMES: &[&str] = &["cb", "callback", "done", "next"];

/// Flags callback invocations inside `.then()`/`.catch()` handlers.
#[derive(Debug)]
pub struct NoCallbackInPromise;

impl LintRule for NoCallbackInPromise {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-callback-in-promise".to_owned(),
            description: "Forbid callbacks inside `.then()`/`.catch()`".to_owned(),
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

        // Check if this is a .then() or .catch() call — extract method name as owned
        // to avoid holding an immutable borrow on ctx across ctx.report().
        let method = match ctx.node(call.callee) {
            Some(AstNode::StaticMemberExpression(member)) => member.property.clone(),
            _ => return,
        };

        if method != "then" && method != "catch" {
            return;
        }

        // Collect callback names from arguments first to avoid borrow conflicts.
        let mut callbacks: Vec<String> = Vec::new();
        for arg in &call.arguments {
            if let Some(AstNode::IdentifierReference(ident)) = ctx.node(*arg) {
                if CALLBACK_NAMES.contains(&ident.name.as_str()) {
                    callbacks.push(ident.name.clone());
                }
            }
        }

        let call_span = Span::new(call.span.start, call.span.end);
        for cb_name in callbacks {
            ctx.report(Diagnostic {
                rule_name: "promise/no-callback-in-promise".to_owned(),
                message: format!(
                    "Do not pass callback `{cb_name}` into `.{method}()` — avoid mixing callbacks and promises"
                ),
                span: call_span,
                severity: Severity::Error,
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

    starlint_rule_framework::lint_rule_test!(NoCallbackInPromise);

    #[test]
    fn test_flags_callback_in_then() {
        let diags = lint("promise.then(cb);");
        assert_eq!(diags.len(), 1, "should flag callback passed to .then()");
    }

    #[test]
    fn test_flags_done_in_catch() {
        let diags = lint("promise.catch(done);");
        assert_eq!(diags.len(), 1, "should flag done passed to .catch()");
    }

    #[test]
    fn test_allows_normal_then() {
        let diags = lint("promise.then(val => val * 2);");
        assert!(diags.is_empty(), "normal .then() should not be flagged");
    }
}
