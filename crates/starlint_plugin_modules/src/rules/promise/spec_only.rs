//! Rule: `promise/spec-only`
//!
//! Forbid non-standard Promise methods. Flags usage of methods that are
//! not part of the ECMAScript specification (e.g. Bluebird extensions).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Standard ECMAScript `Promise` static methods.
const SPEC_STATIC_METHODS: &[&str] = &[
    "resolve",
    "reject",
    "all",
    "allSettled",
    "any",
    "race",
    "withResolvers",
];

/// Standard ECMAScript `Promise` instance methods.
const SPEC_INSTANCE_METHODS: &[&str] = &["then", "catch", "finally"];

/// Flags non-standard Promise static method calls (e.g. `Promise.map`,
/// `Promise.try`, etc.).
#[derive(Debug)]
pub struct SpecOnly;

impl LintRule for SpecOnly {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/spec-only".to_owned(),
            description: "Forbid non-standard Promise methods".to_owned(),
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

        let (method, is_promise) = {
            let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
                return;
            };
            let method = member.property.clone();
            let is_promise = matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "Promise");
            (method, is_promise)
        };

        // Check for non-standard static methods: Promise.xxx()
        if is_promise && !SPEC_STATIC_METHODS.contains(&method.as_str()) {
            ctx.report(Diagnostic {
                rule_name: "promise/spec-only".to_owned(),
                message: format!("`Promise.{method}` is not a standard ECMAScript method"),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }

        // We do not flag instance methods here since we cannot statically
        // determine whether the callee object is a Promise instance.
        // Only flag static methods on the `Promise` global.
        let _ = SPEC_INSTANCE_METHODS;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(SpecOnly);

    #[test]
    fn test_flags_promise_map() {
        let diags = lint("Promise.map([1, 2], fn);");
        assert_eq!(diags.len(), 1, "should flag non-standard Promise.map");
    }

    #[test]
    fn test_flags_promise_try() {
        let diags = lint("Promise.try(() => 1);");
        assert_eq!(diags.len(), 1, "should flag non-standard Promise.try");
    }

    #[test]
    fn test_allows_promise_all() {
        let diags = lint("Promise.all([p1, p2]);");
        assert!(diags.is_empty(), "Promise.all is a standard method");
    }
}
