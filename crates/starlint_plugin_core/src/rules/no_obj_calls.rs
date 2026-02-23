//! Rule: `no-obj-calls`
//!
//! Disallow calling global objects as functions. `Math`, `JSON`, `Reflect`,
//! and `Atomics` are not function constructors — calling them like
//! `Math()` or `JSON()` throws a `TypeError` at runtime.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Non-callable global objects.
const NON_CALLABLE_GLOBALS: &[&str] = &["Math", "JSON", "Reflect", "Atomics"];

/// Flags calls to non-callable global objects.
#[derive(Debug)]
pub struct NoObjCalls;

impl LintRule for NoObjCalls {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-obj-calls".to_owned(),
            description: "Disallow calling global objects as functions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression, AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::CallExpression(call) => {
                if let Some(name) = callee_global_name(ctx, call.callee) {
                    if NON_CALLABLE_GLOBALS.contains(&name) {
                        ctx.report(Diagnostic {
                            rule_name: "no-obj-calls".to_owned(),
                            message: format!("`{name}` is not a function"),
                            span: Span::new(call.span.start, call.span.end),
                            severity: Severity::Error,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            AstNode::NewExpression(new_expr) => {
                if let Some(name) = callee_global_name(ctx, new_expr.callee) {
                    if NON_CALLABLE_GLOBALS.contains(&name) {
                        ctx.report(Diagnostic {
                            rule_name: "no-obj-calls".to_owned(),
                            message: format!("`{name}` is not a constructor"),
                            span: Span::new(new_expr.span.start, new_expr.span.end),
                            severity: Severity::Error,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

/// Extract a simple identifier name from a callee node id.
fn callee_global_name<'a>(ctx: &'a LintContext<'_>, id: NodeId) -> Option<&'a str> {
    match ctx.node(id) {
        Some(AstNode::IdentifierReference(ident)) => Some(ident.name.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoObjCalls)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_math_call() {
        let diags = lint("var x = Math();");
        assert_eq!(diags.len(), 1, "Math() should be flagged");
    }

    #[test]
    fn test_flags_json_call() {
        let diags = lint("var x = JSON();");
        assert_eq!(diags.len(), 1, "JSON() should be flagged");
    }

    #[test]
    fn test_flags_reflect_call() {
        let diags = lint("var x = Reflect();");
        assert_eq!(diags.len(), 1, "Reflect() should be flagged");
    }

    #[test]
    fn test_flags_atomics_call() {
        let diags = lint("var x = Atomics();");
        assert_eq!(diags.len(), 1, "Atomics() should be flagged");
    }

    #[test]
    fn test_flags_new_math() {
        let diags = lint("var x = new Math();");
        assert_eq!(diags.len(), 1, "new Math() should be flagged");
    }

    #[test]
    fn test_allows_math_method() {
        let diags = lint("var x = Math.floor(1.5);");
        assert!(diags.is_empty(), "Math.floor() should not be flagged");
    }

    #[test]
    fn test_allows_json_method() {
        let diags = lint("var x = JSON.parse('{}');");
        assert!(diags.is_empty(), "JSON.parse() should not be flagged");
    }

    #[test]
    fn test_allows_normal_function_call() {
        let diags = lint("var x = foo();");
        assert!(
            diags.is_empty(),
            "normal function call should not be flagged"
        );
    }
}
