//! Rule: `typescript/prefer-promise-reject-errors`
//!
//! Prefer using `Error` objects in `Promise.reject()`. Rejecting with non-Error
//! values (string literals, numbers, booleans, `null`, `undefined`) makes
//! debugging harder because stack traces are lost.
//!
//! Simplified syntax-only version -- full checking requires type information.
//!
//! Flagged patterns:
//! - `Promise.reject("message")`
//! - `Promise.reject(42)`
//! - `Promise.reject(true)`
//! - `Promise.reject(null)`
//! - `Promise.reject(undefined)`

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "typescript/prefer-promise-reject-errors";

/// Flags `Promise.reject()` calls where the argument is a non-Error value.
#[derive(Debug)]
pub struct PreferPromiseRejectErrors;

impl LintRule for PreferPromiseRejectErrors {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer using `Error` objects in `Promise.reject()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check if callee is `Promise.reject`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "reject" {
            return;
        }

        let Some(AstNode::IdentifierReference(obj_id)) = ctx.node(member.object) else {
            return;
        };

        if obj_id.name.as_str() != "Promise" {
            return;
        }

        // Check the first argument -- flag if it is a non-Error literal value
        let Some(&first_arg_id) = call.arguments.first() else {
            return;
        };

        if is_non_error_argument(first_arg_id, ctx) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Expected an `Error` object in `Promise.reject()` — do not reject with a literal value".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Returns `true` if the argument is a literal value that is not an Error:
/// string, number, boolean, null, undefined, or template literal.
fn is_non_error_argument(arg_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(arg_id) {
        Some(
            AstNode::StringLiteral(_)
            | AstNode::NumericLiteral(_)
            | AstNode::BooleanLiteral(_)
            | AstNode::NullLiteral(_)
            | AstNode::TemplateLiteral(_),
        ) => true,
        Some(AstNode::IdentifierReference(ident)) => ident.name == "undefined",
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferPromiseRejectErrors)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_reject_with_string() {
        let diags = lint("Promise.reject(\"error message\");");
        assert_eq!(
            diags.len(),
            1,
            "Promise.reject with string should be flagged"
        );
    }

    #[test]
    fn test_flags_reject_with_number() {
        let diags = lint("Promise.reject(42);");
        assert_eq!(
            diags.len(),
            1,
            "Promise.reject with number should be flagged"
        );
    }

    #[test]
    fn test_flags_reject_with_undefined() {
        let diags = lint("Promise.reject(undefined);");
        assert_eq!(
            diags.len(),
            1,
            "Promise.reject with undefined should be flagged"
        );
    }

    #[test]
    fn test_allows_reject_with_new_error() {
        let diags = lint("Promise.reject(new Error('something failed'));");
        assert!(
            diags.is_empty(),
            "Promise.reject with new Error should not be flagged"
        );
    }

    #[test]
    fn test_allows_reject_with_variable() {
        let diags = lint("Promise.reject(err);");
        assert!(
            diags.is_empty(),
            "Promise.reject with a variable should not be flagged"
        );
    }
}
