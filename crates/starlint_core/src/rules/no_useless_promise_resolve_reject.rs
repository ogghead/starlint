//! Rule: `no-useless-promise-resolve-reject` (unicorn)
//!
//! Disallow wrapping values in `Promise.resolve()` or `Promise.reject()`
//! unnecessarily within async functions, where you can simply return/throw
//! the value directly.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags unnecessary `Promise.resolve()`/`Promise.reject()` in async functions.
#[derive(Debug)]
pub struct NoUselessPromiseResolveReject;

impl LintRule for NoUselessPromiseResolveReject {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-promise-resolve-reject".to_owned(),
            description: "Disallow unnecessary Promise.resolve/reject in async functions"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_semantic(&self) -> bool {
        false
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ReturnStatement])
    }

    fn run(&self, node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Look for return statements
        let AstNode::ReturnStatement(ret) = node else {
            return;
        };

        let Some(arg_id) = ret.argument else {
            return;
        };

        // Check if the return value is Promise.resolve(...) or Promise.reject(...)
        let Some(method_name) = is_promise_resolve_or_reject(arg_id, ctx) else {
            return;
        };

        // Extract the inner argument text for the fix
        let inner_arg_text = extract_promise_inner_arg(arg_id, ctx);

        // Walk ancestors to check if we're inside an async function
        let mut current = node_id;
        loop {
            let Some(parent_id) = ctx.parent(current) else {
                break;
            };
            match ctx.node(parent_id) {
                Some(AstNode::Function(func)) if func.is_async => {
                    report_promise_fix(ctx, ret.span, &method_name, &inner_arg_text);
                    return;
                }
                Some(AstNode::ArrowFunctionExpression(arrow)) if arrow.is_async => {
                    report_promise_fix(ctx, ret.span, &method_name, &inner_arg_text);
                    return;
                }
                // Hit a non-async function boundary, stop
                Some(AstNode::Function(_) | AstNode::ArrowFunctionExpression(_)) => {
                    return;
                }
                None => break,
                _ => {}
            }
            current = parent_id;
        }
    }
}

/// Extract the inner argument text from `Promise.resolve(x)` or `Promise.reject(x)`.
/// Returns the argument source text, or an empty string if no arguments.
fn extract_promise_inner_arg(expr_id: NodeId, ctx: &LintContext<'_>) -> String {
    let Some(AstNode::CallExpression(call)) = ctx.node(expr_id) else {
        return String::new();
    };

    if let Some(&first_arg_id) = call.arguments.first() {
        let arg_span = ctx.node(first_arg_id).map_or(
            starlint_ast::types::Span::EMPTY,
            starlint_ast::AstNode::span,
        );
        let start = usize::try_from(arg_span.start).unwrap_or(0);
        let end = usize::try_from(arg_span.end).unwrap_or(0);
        ctx.source_text().get(start..end).unwrap_or("").to_owned()
    } else {
        String::new()
    }
}

/// Report the diagnostic with a fix for Promise.resolve/reject.
fn report_promise_fix(
    ctx: &mut LintContext<'_>,
    ret_span: starlint_ast::types::Span,
    method_name: &str,
    inner_arg_text: &str,
) {
    let span = Span::new(ret_span.start, ret_span.end);
    let replacement = if method_name == "resolve" {
        if inner_arg_text.is_empty() {
            "return".to_owned()
        } else {
            format!("return {inner_arg_text}")
        }
    } else if inner_arg_text.is_empty() {
        "throw undefined".to_owned()
    } else {
        format!("throw {inner_arg_text}")
    };
    let fix_message = if method_name == "resolve" {
        "Replace with `return` value directly".to_owned()
    } else {
        "Replace with `throw` directly".to_owned()
    };
    ctx.report(Diagnostic {
        rule_name: "no-useless-promise-resolve-reject".to_owned(),
        message: format!(
            "Unnecessary `Promise.{method_name}()` in async function; \
             use `return` or `throw` directly"
        ),
        span,
        severity: Severity::Warning,
        help: Some(format!("Use `{replacement}` instead")),
        fix: Some(Fix {
            kind: FixKind::SafeFix,
            message: fix_message,
            edits: vec![Edit { span, replacement }],
            is_snippet: false,
        }),
        labels: vec![],
    });
}

/// Check if an expression is `Promise.resolve(...)` or `Promise.reject(...)`.
/// Returns the method name if it matches.
fn is_promise_resolve_or_reject(expr_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    let Some(AstNode::CallExpression(call)) = ctx.node(expr_id) else {
        return None;
    };

    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
        return None;
    };

    let Some(AstNode::IdentifierReference(obj)) = ctx.node(member.object) else {
        return None;
    };

    if obj.name != "Promise" {
        return None;
    }

    let name = member.property.clone();
    (name == "resolve" || name == "reject").then_some(name)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUselessPromiseResolveReject)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_resolve_in_async() {
        let diags = lint("async function foo() { return Promise.resolve(1); }");
        assert_eq!(diags.len(), 1, "Promise.resolve in async should be flagged");
    }

    #[test]
    fn test_flags_reject_in_async() {
        let diags = lint("async function foo() { return Promise.reject(new Error('x')); }");
        assert_eq!(diags.len(), 1, "Promise.reject in async should be flagged");
    }

    #[test]
    fn test_allows_resolve_in_non_async() {
        let diags = lint("function foo() { return Promise.resolve(1); }");
        assert!(
            diags.is_empty(),
            "Promise.resolve in non-async should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_return_in_async() {
        let diags = lint("async function foo() { return 1; }");
        assert!(
            diags.is_empty(),
            "normal return in async should not be flagged"
        );
    }

    #[test]
    fn test_allows_promise_all() {
        let diags = lint("async function foo() { return Promise.all([a, b]); }");
        assert!(
            diags.is_empty(),
            "Promise.all in async should not be flagged"
        );
    }
}
