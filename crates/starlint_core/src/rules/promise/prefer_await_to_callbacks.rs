//! Rule: `promise/prefer-await-to-callbacks`
//!
//! Prefer `async`/`await` over callback-style functions. Encourages
//! modern asynchronous patterns over Node.js-style error-first callbacks.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Common callback parameter names that suggest callback-style code.
const CALLBACK_PARAMS: &[&str] = &["cb", "callback", "done", "next"];

/// Flags functions with callback-named parameters, suggesting `async`/`await`.
#[derive(Debug)]
pub struct PreferAwaitToCallbacks;

impl LintRule for PreferAwaitToCallbacks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/prefer-await-to-callbacks".to_owned(),
            description: "Prefer `async`/`await` over callbacks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrowFunctionExpression, AstNodeType::Function])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let (params, span) = match node {
            AstNode::Function(func) => {
                if func.is_async {
                    return; // Already async, skip
                }
                (&func.params, Span::new(func.span.start, func.span.end))
            }
            AstNode::ArrowFunctionExpression(arrow) => {
                if arrow.is_async {
                    return;
                }
                (&arrow.params, Span::new(arrow.span.start, arrow.span.end))
            }
            _ => return,
        };

        for param_id in params {
            if let Some(AstNode::BindingIdentifier(id)) = ctx.node(*param_id) {
                let name = id.name.as_str();
                if CALLBACK_PARAMS.contains(&name) {
                    ctx.report(Diagnostic {
                        rule_name: "promise/prefer-await-to-callbacks".to_owned(),
                        message: format!(
                            "Function has callback parameter `{name}` — prefer `async`/`await`"
                        ),
                        span,
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                    return; // Only report once per function
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferAwaitToCallbacks)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_callback_param() {
        let diags = lint("function foo(callback) { callback(null, 1); }");
        assert_eq!(diags.len(), 1, "should flag function with callback param");
    }

    #[test]
    fn test_allows_async_function() {
        let diags = lint("async function foo(callback) { }");
        assert!(diags.is_empty(), "async function should not be flagged");
    }

    #[test]
    fn test_allows_normal_params() {
        let diags = lint("function foo(x, y) { return x + y; }");
        assert!(diags.is_empty(), "normal params should not be flagged");
    }
}
